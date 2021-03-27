//! # CHIP-8 Memory Module
//! ## Description
//! Represents the RAM of the virtual computer.
//! There are 0xFFF (4096) addresses available, and each is 16 bits in size.
//! ## Operation
//! The functions serve to abstract away the following operations:
//! * Loading a new program to memory
//! * Reading a specific address
//! * Writing to a specific address
//! Inside the struct, the memory is represented as an array of u8, and the functions join or split the inputs and outputs when necessary.

/// Represents the memory of the virtual computer.
///
/// Remember to load the fonts so they can be used by the programs.
pub struct Memory {
    pub space: [u8; Memory::BYTE_MAX],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            space: [0; Memory::BYTE_MAX],
        }
    }
}

impl Memory {
    /// CHIP-8 programs are loaded at address 0x200, 0x000 to 0x1FF is reserved for the interpreter.
    const START: u16 = 0x200;
    /// It is technically impossible to access more than 0xFFF due to how the I register is loaded.
    const MAX: u16 = 0xFFF;
    /// The biggest memory size used with the CHIP-8 is 8k on the COSMAC VIP.
    const BYTE_MAX: usize = 8192;
    /// Maximun size a program can be.
    const USABLE_SPACE: usize = (Memory::MAX as usize - Memory::START as usize + 1) * 2;
    /// Write to a memory address.
    pub fn write(&mut self, pos: u16, data: u16) -> Result<&'static str, &'static str> {
        let pos_u: usize = (pos * 2) as usize;
        if pos >= Memory::START && pos <= Memory::MAX {
            let data_head: u8 = (data >> 8) as u8;
            let data_tail: u8 = (data & 0xFF) as u8;
            self.space[pos_u] = data_head;
            self.space[pos_u + 1] = data_tail;
            return Ok("Ok");
        } else {
            return Err("Out of bounds exception");
        }
    }
    /// Write to a memory address without checking for lower bounds.
    /// Used for loading fonts to the interpreter reserved space.
    pub fn unbound_write(&mut self, pos: u16, data: u16) -> Result<&'static str, &'static str> {
        let pos_u: usize = (pos * 2) as usize;
        if pos <= Memory::MAX {
            let data_head: u8 = (data >> 8) as u8;
            let data_tail: u8 = (data & 0xFF) as u8;
            self.space[pos_u] = data_head;
            self.space[pos_u + 1] = data_tail;
            return Ok("Ok");
        } else {
            return Err("Out of bounds exception");
        }
    }
    /// Read the value from a memory address.
    pub fn read(&mut self, pos: u16) -> Result<u16, &'static str> {
        let pos_u: usize = (pos * 2) as usize;
        if pos <= Memory::MAX {
            let data_head: u16 = ((self.space[pos_u]) as u16) << 8;
            let data_tail: u16 = (self.space[pos_u + 1]) as u16;
            //println!("{:x} {:x}", self.space[pos_u], self.space[pos_u + 1]);
            return Ok(data_head | data_tail);
        } else {
            return Err("Out of bounds exception");
        }
    }
    /// Load a program to memory, it starts at 0x200.
    pub fn load(&mut self, program: &[u8]) -> Result<&'static str, &'static str> {
        let pos: usize = (Memory::START * 2) as usize;
        if program.len() <= Memory::USABLE_SPACE {
            let mut idx: usize = 0;
            while idx < program.len() {
                self.space[pos + idx] = program[idx].clone();
                idx = idx + 1
            }
            return Ok("Ok");
        } else {
            return Err("Program bigger than memory space");
        }
    }
    pub fn print_memory(&mut self) {
        for idx in 0x200..=Memory::MAX {
            println!(
                "{:4x}: {:4x}",
                idx - 0x200,
                self.read(idx).expect("Couldn't print a valid memory addr")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Memory;
    mod load {
        use super::Memory;
        #[test]
        fn wrong_case() {
            let mut mem = Memory {
                ..Default::default()
            };
            let values: [u8; 0xFFFF] = [1; 0xFFFF];
            assert!(mem.load(&values).is_err(), "Didn't fail")
        }
        #[test]
        fn correct_case() {
            let mut mem = Memory {
                ..Default::default()
            };
            let values: [u8; Memory::USABLE_SPACE] = [1; Memory::USABLE_SPACE];
            let result: Result<&'static str, &'static str> = mem.load(&values);
            assert_eq!(*mem.space.last().unwrap(), 1 as u8);
            assert!(!result.is_err(), "Couldn't load the memory")
        }
    }

    mod write {
        use super::Memory;
        #[test]
        fn out_of_bounds_upper() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0xFFF + 1;
            let data: u16 = 0xFFFF;
            assert!(mem.write(pos, data).is_err(), "Upper bounds didn't work")
        }
        #[test]
        fn out_of_bounds_lower() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0x200 - 1;
            let data: u16 = 0xFFFF;
            assert!(mem.write(pos, data).is_err(), "Lower bounds didn't work")
        }
        #[test]
        fn correct_case() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0x400;
            let data: u16 = 0xFFFF;
            let result: Result<&'static str, &'static str> = mem.write(pos, data);
            assert!(!result.is_err(), "Failed to write to memory");
            assert_eq!(
                mem.space[(pos * 2) as usize],
                0xFF,
                "Wrong value written on head"
            );
            assert_eq!(
                mem.space[(pos * 2 + 1) as usize],
                0xFF,
                "Wrong value written on tail"
            );
        }
    }
    mod read {
        use super::Memory;
        #[test]
        fn out_of_bounds_upper() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0xFFF + 1;
            assert!(mem.read(pos).is_err(), "Upper bounds didn't work")
        }
        #[test]
        fn correct_case() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0x400;
            let data: u8 = 0xFF;
            mem.space[(pos * 2) as usize] = data;
            let result: Result<u16, &'static str> = mem.read(pos);
            assert!(!result.is_err(), "Failed to read memory");
            assert_eq!(mem.read(pos).unwrap(), 0xFF00, "Wrong value received");
        }
        #[test]
        fn test_every_addr() {
            let mut mem = Memory {
                ..Default::default()
            };
            for idx in 0x200..=Memory::MAX {
                println!(
                    "{:4x}: {:4x}",
                    idx - 0x200,
                    mem.read(idx).expect("Couldn't print a valid memory addr")
                )
            }
        }
    }
}
