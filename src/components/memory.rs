struct Memory {
    space: [i8; 4096],
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            space: [0; 0xFFF + 1],
        }
    }
}

impl Memory {
    const START: u16 = 0x200;
    const MAX: u16 = 0xFFF;
    fn write() {}
    fn load(&self, program: &[u8]) -> Result<&'static str, &'static str> {
        let pos: u16 = Memory::START;
        println!("{}", (Memory::MAX - Memory::START + 1) as usize);
        if program.len() < (Memory::MAX - Memory::START + 1) as usize {
            for byte in program {
                println!("{}", byte);
            }
            return Ok("Ok");
        } else {
            return Err("Program bigger than memory space");
        }
    }
}

#[cfg(test)]
mod load {
    use super::Memory;
    #[test]
    fn wrong_case() {
        let mut mem = Memory {
            ..Default::default()
        };
        let values: [u8; 0xFFF] = [0; 0xFFF];
        assert!(mem.load(&values).is_err(), "Failed correctly")
    }
    #[test]
    fn correct_case() {
        let mut mem = Memory {
            ..Default::default()
        };
        let values: [u8; 0xFFF] = [0; 0xFFF];
        assert!(mem.load(&values).is_err(), "Failed correctly")
    }
}
