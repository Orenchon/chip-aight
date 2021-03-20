struct Memory {
    space: [u8; 4096],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            space: [0; 0xFFF + 1],
        }
    }
}

impl Memory {
    const START: usize = 0x200; // CHIP-8 programs are loaded at address 0x200, 0x000 to 0x1FF is reserved for the interpreter
    const MAX: usize = 0xFFF; // The biggest memory size used with the CHIP-8 is 4k on the COSMAC VIP
    const USABLE_SPACE: usize = Memory::MAX as usize - Memory::START as usize + 1;
    fn write(&mut self, pos: u16, data: u8) -> Result<&'static str, &'static str> {
        let pos_u: usize = pos as usize;
        if pos_u >= Memory::START && pos_u <= Memory::MAX {
            self.space[pos_u] = data;
            return Ok("Ok");
        } else {
            return Err("Out of bounds exception");
        }
    }
    fn read(&mut self, pos: u16) -> Result<u8, &'static str> {
        let pos_u: usize = pos as usize;
        if pos_u >= Memory::START && pos_u <= Memory::MAX {
            return Ok(self.space[pos_u]);
        } else {
            return Err("Out of bounds exception");
        }
    }
    fn load(&mut self, program: &[u8]) -> Result<&'static str, &'static str> {
        let mut pos: usize = Memory::START;
        // println!("{}", (Memory::MAX - Memory::START + 1) as usize);
        if program.len() <= Memory::USABLE_SPACE {
            for byte in program {
                println!("{}", byte);
                self.space[pos] = byte.clone();
                pos = pos + 1;
            }
            return Ok("Ok");
        } else {
            return Err("Program bigger than memory space");
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
            let values: [u8; 0xFFF] = [1; 0xFFF];
            assert!(mem.load(&values).is_err(), "Didn't fail")
        }
        #[test]
        fn correct_case() {
            let mut mem = Memory {
                ..Default::default()
            };
            let values: [u8; Memory::USABLE_SPACE] = [1; Memory::USABLE_SPACE];
            println!("{}", values.len());
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
            let data: u8 = 0xFF;
            assert!(mem.write(pos, data).is_err(), "Upper bounds didn't work")
        }
        #[test]
        fn out_of_bounds_lower() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0x200 - 1;
            let data: u8 = 0xFF;
            assert!(mem.write(pos, data).is_err(), "Lower bounds didn't work")
        }
        #[test]
        fn correct_case() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0x400;
            let data: u8 = 0xFF;
            let result: Result<&'static str, &'static str> = mem.write(pos, data);
            assert!(!result.is_err(), "Failed to write to memory");
            assert_eq!(mem.space[pos as usize], data, "Wrong value written");
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
        fn out_of_bounds_lower() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0x200 - 1;
            assert!(mem.read(pos).is_err(), "Lower bounds didn't work")
        }
        #[test]
        fn correct_case() {
            let mut mem = Memory {
                ..Default::default()
            };
            let pos: u16 = 0x400;
            let data: u8 = 0xFF;
            mem.space[pos as usize] = data;
            let result: Result<u8, &'static str> = mem.read(pos);
            assert!(!result.is_err(), "Failed to read memory");
            assert_eq!(mem.read(pos).unwrap(), data, "Wrong value received");
        }
    }
}
