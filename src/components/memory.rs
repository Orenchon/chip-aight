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
    const START: usize = 0x200;
    const MAX: usize = 0xFFF;
    const USABLE_SPACE: usize = Memory::MAX as usize - Memory::START as usize + 1;
    fn write() {}
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
mod load {
    use super::Memory;
    #[test]
    fn wrong_case() {
        let mut mem = Memory {
            ..Default::default()
        };
        let values: [u8; 0xFFF] = [1; 0xFFF];
        assert!(mem.load(&values).is_err(), "Failed correctly")
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
        assert!(!result.is_err(), "Succesfully overwrote the memory")
    }
}
