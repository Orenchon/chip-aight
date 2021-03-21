pub struct Cpu {
    v: [u8; 16], /* Data registers
                  * The CHIP-8 interpreter has 16 general purpose data registers, V0 to VF
                  * Each is 8 bits in length
                  * Instructions write, read, add, substract or even more to these registers
                  */
    stack: Vec<u16>, /* Subroutine stack
                      * When 2NNN or 0NNN is called, the current PC should be pushed to it
                      * When 00EE is called, the PC should be set to a pop of it
                      * Use Vec because it already implements push() and pop()
                      * In reality, the stack would have a limited size based on physical constraints
                      * Vec is infinite
                      */
    program_counter: u16, /* Program counter
                           * It tells us what the current instruction to be executed is
                           * Always set to 0x200 when execution begins
                           * (Only valid for regular CHIP-8 implementations, others may vary)
                           */
    i: u16, /*  Address register
             *  Used with read and write operations
             *  Due to the way op addresses work, only 12 bits can be actually loaded
             */
    dt: u8, /*  Delay timer
             *  Counts down at a rate of 1 per second until 0 is reached
             *  Set by instruction Fx15 and read by using Fx07
             */
    st: u8, /*  Sound timer
             *  Counts down at 60 hertz just like the Delay timer
             *  While it is active, a sound will ring
             *  The waveform and frequency is unspecified
             *  Set by instruction Fx18
             *  Will do nothing if set to 0x01
             */
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu {
            v: [0; 16],
            stack: Vec::new(),
            program_counter: 0x200,
            i: 0,
            dt: 0,
            st: 0,
        }
    }
}

/*  # CHIP-8 Instruction Set
*   ## Description
*   There are 35 different instructions in the most common implementation
*   Most of them contain values inside:
*   * nnn refers to a hexadecimal address
*   * nn refers to a hexadecimal byte
*   * n refers to a hexadecimal nibble (half-byte)
*   * x and y are 2 different registers in the same op
*   Most of the time we will be incrementing the PC by one, unless when an instruction calls for a skip
*   ## Organization
*   It is possible to order the instructions by looking at the most and least significant nibbles
*   This makes using a switch easier
*   ## Timer woes
*   So, the CHIP-8 is not really an emulator, instead it is more of an interpreted language
*   What this means is that the clock speed is not a constant (Like a modern console) or set by hardware (Like the 8008, which was set by quartz crystal)
*   Because of this, a bit of trial and error is needed to get a mostly correct timing
*   The "magic number" is often between 500hz and 600hz
*   ## The font utility
*   As it is often necessary to print numbers on screen, CHIP-8 comes for an utility for it
*   The Dxyn instruction allows for quick output of the hex value of n to position (x,y)
    A list of the font characters can be found in the font constants, to be loaded to memory on constructor of Cpu
*   ## Binary Coded Decimal: WTF is bcd()
*   Fx33 takes the binary coded decimal of the value in Vx and places it in I, I+1 and I+2
*   Taking the binary coded decimal of a number is simply spliting the digits of it into different places
*   For example:
*   The number 114 (0b1110010 in binary)
*   Gets split the following way:
*   [1, 1, 4]
*   This is used by the font utility to be able to display big numbers
*   ## Instructions
*   0nnn - Execute machine language subroutine at nnn
*   00E0 - cls()
*   00EE - Return from subroutine
*   1nnn - Jump to nnn
*   2nnn - Execute subroutine at nnn
*   3xnn - Skip if Vx == nn
*   4xnn - Skip if Vx != nn
*   5xy0 - Skip if Vx == Vy
*   6xnn - Vx = nn
*   7xnn - Vx = Vx + nn
*   8xy0 - Vx = Vy
*   8xy1 - Vx = Vx | Vy
*   8xy2 - Vx = Vx & Vy
*   8xy3 - Vx = Vx ^ Vy
*   8xy4 - Vx = Vx + Vy; VF = Carry?
*   8xy5 - Vx = Vx - Vy; VF = Borrow?
*   8xy6 - Vx = Vy >> 1; VF = Vy & 1 //Least significant bit prior to the shift. In S-CHIP Vx is shifted
*   8xy7 - Vx = Vy - Vx; VF = Borrow?
*   8xyE - Vx = Vy << 1; VF = Vy >> 7
*   9xy0 - Skip if Vx != Vy
*   Annn - I = nnn
*   Bnnn - Jump to nnn + V0
*   Cxnn = Vx = Rand() & nn
*   Dxyn = draw(x: Vx, y: Vy, sprite: sprite(amount_of_data: n, sprite_addr: I)); VF = Pixels unset?
*   Ex9E = Skip if key_pressed(hex(Vx)) //keypad is formed by numbers in hex
*   ExA1 = Skip if !key_pressed(hex(Vx))
*   Fx07 = Vx = dt
*   Fx0A = Vx = block_until_keypress()
*   Fx15 = dt = Vx
*   Fx18 = st = Vx
*   Fx1E = I = I + Vx; Unconfirmed: VF = Carry?
*   Fx29 = I = addr(sprite(Vx)) // Sprite address is internal to the interpreter, it'll have to be placed within 0x000 and 0x1FF
*   Fx33 = [I, I+1, I+2] = bcd(hex(Vx))
*   Fx55 = [I, I..., I + x] = [V0, V..., Vx]; I = I + x + 1
*   Fx55 = [V0, V..., Vx] = [I, I..., I + x]; I = I + x + 1
*/

impl Cpu {
    pub const FONT: [[u16; 5]; 16] = [
        [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0 Done
        [0x20, 0x60, 0x20, 0x20, 0x70], // 1 Done
        [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2 Done
        [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3 Done
        [0x90, 0x90, 0xF0, 0x10, 0x10], // 4 Done
        [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5 Done
        [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6 Done
        [0xF0, 0x10, 0x20, 0x40, 0x40], // 7 Done
        [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8 Done
        [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9 Done
        [0xF0, 0x90, 0xF0, 0x90, 0x90], // A Done
        [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B Done
        [0xF0, 0x80, 0x80, 0x80, 0xF0], // C Done
        [0xE0, 0x90, 0x90, 0x90, 0xE0], // D Done
        [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E Done
        [0xF0, 0x80, 0xF0, 0x80, 0x80], // F Done
    ];
    // 0nnn - Execute machine language subroutine at nnn
    // Implemented same as 2nnn in this case
    fn ml_sub(&mut self, addr: u16) {
        self.call_sub(addr)
    }
    // 0E00 - cls()
    fn cls(&self, state: &mut [[bool; 32]; 64]) {
        *state = [[false; 32]; 64];
    }
    // 00EE - Return from subroutine
    fn ret_sub(&mut self) {
        let popped_addr = self.stack.pop().unwrap();
        self.program_counter = popped_addr
    }
    // 1nnn - Jump to nnn
    fn jump(&mut self, addr: u16) {
        self.program_counter = addr - 1;
    }
    // 2nnn - Execute subroutine at nnn
    fn call_sub(&mut self, addr: u16) {
        self.stack.push(self.program_counter);
        self.program_counter = addr - 1
    }
    //3xnn - Skip if Vx == nn
    fn if_reg_equals_nn(&mut self, reg: u8, nn: u8) {
        let vx = self.v[reg as usize];
        if vx == nn {
            self.program_counter = self.program_counter + 1
        }
    }
    //4xnn - Skip if Vx != nn
    fn if_not_reg_equals_nn(&mut self, reg: u8, nn: u8) {
        let vx = self.v[reg as usize];
        if vx != nn {
            self.program_counter = self.program_counter + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    mod ops {
        use super::Cpu;
        #[test]
        fn ml_sub() {
            let mut mem = Cpu {
                ..Default::default()
            };
            let test_addr: u16 = 0x400;
            mem.ml_sub(test_addr);
            assert!(mem.stack.len() > 0, "Stack should've been pushed");
            assert_eq!(
                mem.program_counter,
                test_addr - 1,
                "Address should be set properly"
            );
            assert_eq!(
                mem.stack[0], 0x200,
                "Check that the original address is in the stack"
            )
        }
        #[test]
        fn cls() {
            let mut mem = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[true; 32]; 64];
            mem.cls(&mut test_state);
            for item in test_state.iter().flat_map(|sub| sub.iter()) {
                assert_eq!(*item, false, "Array is not empty in a certain position")
            }
        }
        #[test]
        fn ret_sub() {
            let mut mem = Cpu {
                ..Default::default()
            };
            let example_addr1: u16 = 0x400;
            let example_addr2: u16 = 0x500;
            mem.stack.push(example_addr2); //Push trash so array is not at 0 all the time
            mem.stack.push(example_addr1);
            mem.ret_sub();
            assert_eq!(
                mem.program_counter, example_addr1,
                "Did not pop the correct address the first time"
            );
            mem.ret_sub();
            assert_eq!(
                mem.program_counter, example_addr2,
                "Did not pop the correct address the second time"
            )
        }
        #[test]
        fn jump() {
            let mut mem = Cpu {
                ..Default::default()
            };
            let example_addr: u16 = 0x400;
            mem.jump(example_addr);
            assert_eq!(mem.program_counter, example_addr - 1, "Wrong address set")
        }
        #[test]
        fn call_sub() {
            let mut mem = Cpu {
                ..Default::default()
            };
            let test_addr: u16 = 0x400;
            mem.call_sub(test_addr);
            assert!(mem.stack.len() > 0, "Stack should've been pushed");
            assert_eq!(
                mem.program_counter,
                test_addr - 1,
                "Address should be set properly"
            );
            assert_eq!(
                mem.stack[0], 0x200,
                "Check that the original address is in the stack"
            )
        }
        #[test]
        fn if_reg_equals_nn() {
            let mut mem = Cpu {
                ..Default::default()
            };
            let instruction: u16 = 0x3404;
            let nn = (instruction & 0xFF) as u8;
            let reg = ((instruction & 0xF00) >> 8) as u8;
            mem.v[4] = 4;
            let expected_pc = mem.program_counter + 1;
            mem.if_reg_equals_nn(reg, nn);
            assert_eq!(
                mem.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn if_not_reg_equals_nn() {
            let mut mem = Cpu {
                ..Default::default()
            };
            let instruction: u16 = 0x3405;
            let nn = (instruction & 0xFF) as u8;
            let reg = ((instruction & 0xF00) >> 8) as u8;
            mem.v[4] = 4;
            let expected_pc = mem.program_counter + 1;
            mem.if_not_reg_equals_nn(reg, nn);
            assert_eq!(
                mem.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
    }
}
