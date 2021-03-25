//! # CHIP-8 Interpreter
//! ## Description
//! There are 35 different instructions in the most common implementation.
//!
//! Most of them contain values inside:
//! * nnn refers to a hexadecimal address
//! * nn refers to a hexadecimal byte
//! * n refers to a hexadecimal nibble (half-byte)
//! * x and y are 2 different registers in the same op
//! Most of the time we will be incrementing the PC by one, unless when an instruction calls for a skip.
//! ## Organization
//! It is possible to order the instructions by looking at the most and least significant nibbles.
//! This makes using a switch easier.
//! ## Timer woes
//! So, the CHIP-8 is not really an emulator, instead it is more of an interpreted language.
//! What this means is that the clock speed is not a constant (Like a modern console) or set by hardware (Like the 8008, which was set by quartz crystal).
//!
//! Because of this, a bit of trial and error is needed to get a mostly correct timing.
//! The "magic number" is often between 500hz and 600hz.
//! ## The font utility
//! As it is often necessary to print numbers on screen, CHIP-8 comes for an utility for it.
//! The Dxyn instruction allows for quick output of the hex value of n to position (x,y).
//!
//! A list of the font characters can be found in the font constants, to be loaded to memory on constructor of Cpu.
//! ## Binary Coded Decimal: What is bcd()
//! Fx33 takes the binary coded decimal of the value in Vx and places it in I, I+1 and I+2.
//! Taking the binary coded decimal of a number is simply spliting the digits of it into different places.
//!
//! ### For example:
//!
//! The number 114 (0b1110010 in binary) gets split the following way:
//!
//! ```
//! mem[I] = 1
//! mem[I + 1] = 1
//! mem[I + 2] = 4
//! ```
//!
//! This is used by the font utility to be able to display big numbers fast.

use super::memory;
use rand::Rng;

/// Represents the processor, running instructions and sending orders to other modules
pub struct Cpu {
    /// Data registers
    ///
    /// The CHIP-8 interpreter has 16 general purpose data registers, V0 to VF.
    /// Each is 8 bits in length.
    /// Instructions write, read, add, substract or even more to these registers.
    v: [u8; 16],
    /// Subroutine stack
    ///
    /// When 2NNN or 0NNN is called, the current PC should be pushed to it.
    /// When 00EE is called, the PC should be set to a pop of it.
    /// Uses Vec because it already implements push() and pop().
    ///
    /// In reality, the stack would have a limited size based on physical constraints.
    /// Vec is infinite
    stack: Vec<u16>,
    /// Program counter
    ///
    /// It tells us what the current instruction to be executed is.
    /// Always set to 0x200 when execution begins
    /// (Only valid for regular CHIP-8 implementations, others may vary).
    program_counter: u16,
    /// Address register
    ///
    /// Used with read and write operations.
    /// Due to the way op addresses work, only 12 bits can be actually loaded.
    i: u16,
    /// Delay timer
    ///
    /// Counts down at a rate of 1 per second until 0 is reached.
    /// Set by instruction Fx15 and read by using Fx07.
    dt: u8,
    /// Sound timer
    ///
    /// Counts down at 60 hertz just like the Delay timer.
    /// While it is active, a sound will ring.
    ///
    /// The waveform and frequency is unspecified.
    /// Set by instruction Fx18.
    /// Will do nothing if set to 0x01
    st: u8,
    /// Used to generate random numbers for Cxnn
    rng: rand::rngs::ThreadRng,
    /// Used by the Fx0A instruction to be able to compare changes in state
    is_key_pressed_temp: Option<[bool; 16]>,
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
            rng: rand::thread_rng(),
            is_key_pressed_temp: None,
        }
    }
}

/*  ## Instructions
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
*   Dxyn = draw(x: Vx, y: Vy, sprite: sprite(sprite_height: n, sprite_addr: I)); VF = Pixels unset?
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

    /// Used to load the fonts in the default location so that they can be used by Dxyn/draw_sprite()
    pub fn write_fonts_to_mem(mem: &mut memory::Memory) {
        for (idx, sprite) in Cpu::FONT.iter().flatten().enumerate() {
            let res = mem.unbound_write((idx + 0x20) as u16, *sprite);
            match res {
                Err(err) => panic!("{}", err),
                _ => (),
            }
        }
    }
    /// 0nnn - Execute machine language subroutine at nnn
    /// Implemented same as 2nnn in this case
    fn ml_sub(&mut self, addr: u16) {
        self.call_sub(addr)
    }
    /// 0E00 - cls()
    fn cls(&self, state: &mut [[bool; 32]; 64]) {
        *state = [[false; 32]; 64];
    }
    /// 00EE - Return from subroutine
    fn ret_sub(&mut self) {
        let popped_addr = self.stack.pop().unwrap();
        self.program_counter = popped_addr
    }
    /// 1nnn - Jump to nnn
    fn jump(&mut self, addr: u16) {
        self.program_counter = addr - 1;
    }
    /// 2nnn - Execute subroutine at nnn
    fn call_sub(&mut self, addr: u16) {
        self.stack.push(self.program_counter);
        self.program_counter = addr - 1
    }
    /// 3xnn - Skip if Vx == nn
    fn if_reg_equals_nn(&mut self, x: u8, nn: u8) {
        let vx = self.v[x as usize];
        if vx == nn {
            self.program_counter = self.program_counter + 1
        }
    }
    /// 4xnn - Skip if Vx != nn
    fn if_not_reg_equals_nn(&mut self, x: u8, nn: u8) {
        let vx = self.v[x as usize];
        if vx != nn {
            self.program_counter = self.program_counter + 1
        }
    }
    /// 5xy0 - Skip if Vx == Vy
    fn if_reg_equals_reg(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.program_counter = self.program_counter + 1
        }
    }
    /// 6xnn - Vx = nn
    fn reg_store_nn(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = nn
    }
    /// 7xnn - Vx = Vx + nn; CHECK OVERFLOW BEHAVIOR
    fn reg_add_nn(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = self.v[x as usize] + nn
    }
    /// 8xy0 - Vx = Vy
    fn assign_reg_to_reg(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize]
    }
    /// 8xy1 - Vx = Vx | Vy
    fn reg_or_reg(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[x as usize] | self.v[y as usize]
    }
    /// 8xy2 - Vx = Vx & Vy
    fn reg_and_reg(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[x as usize] & self.v[y as usize]
    }
    /// 8xy3 - Vx = Vx ^ Vy
    fn reg_xor_reg(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize]
    }
    /// 8xy4 - Vx = Vx + Vy; VF = Carry?
    fn reg_plus_reg(&mut self, x: u8, y: u8) {
        let result = self.v[x as usize].overflowing_add(self.v[y as usize]);
        self.v[x as usize] = result.0;
        self.v[0xF] = result.1 as u8;
    }
    /// 8xy5 - Vx = Vx - Vy; VF = 0 if borrow else 1
    fn reg_minus_reg(&mut self, x: u8, y: u8) {
        let result = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        self.v[x as usize] = result.0;
        self.v[0xF] = (!result.1) as u8;
    }
    /// 8xy6 - Vx = Vy >> 1; VF = Vy & 1
    fn reg_shift_right(&mut self, x: u8) {
        self.v[0xF] = self.v[x as usize] & 1;
        self.v[x as usize] = self.v[x as usize] >> 1;
    }
    /// 8xy7 - Vx = Vy - Vx; VF = Borrow?
    fn reverse_reg_minus_reg(&mut self, x: u8, y: u8) {
        let result = self.v[y as usize].overflowing_sub(self.v[x as usize]);
        self.v[x as usize] = result.0;
        self.v[0xF] = (!result.1) as u8;
    }
    /// 8xyE - Vx = Vy << 1; VF = Vy >> 7
    fn reg_shift_left(&mut self, x: u8) {
        self.v[0xF] = self.v[x as usize] >> 7;
        self.v[x as usize] = self.v[x as usize] << 1;
    }
    /// 9xy0 - Skip if Vx != Vy
    fn if_not_reg_equals_reg(&mut self, x: u8, y: u8) {
        if self.v[x as usize] != self.v[y as usize] {
            self.program_counter = self.program_counter + 1
        }
    }
    /// Annn - I = nnn
    fn store_addr(&mut self, nnn: u16) {
        self.i = nnn;
    }
    /// Bnnn - Jump to nnn + V0
    fn reg_plus_nnn_jump(&mut self, nnn: u16) {
        self.program_counter = self.v[0] as u16 + nnn - 1
    }
    /// Cxnn = Vx = Rand() & nn
    fn random(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = self.rng.gen::<u8>() & nn;
    }
    /// Dxyn = draw(x: Vx, y: Vy, sprite: sprite(sprite_height: n, sprite_addr: I)); VF = Pixels unset?
    fn draw_sprite(
        &mut self,
        x: u8,
        y: u8,
        n: u8,
        state: &mut [[bool; 32]; 64],
        mem: &mut memory::Memory,
    ) {
        self.v[0xF] = 0;
        for sprite_row in 0..n {
            let row_pos = (self.v[y as usize] + sprite_row) as usize;
            let sprite_value = (mem.read(self.i + sprite_row as u16).unwrap()) as u8;
            println!("{}", sprite_value);
            for sprite_col in 0..8 as u8 {
                let col_pos = (sprite_col + self.v[x as usize]) as usize;
                let bit = (sprite_value >> (7 - sprite_col)) & 1;
                let state_bit = state[col_pos % 64][row_pos % 32] as u8;
                if bit & state_bit > 0 {
                    self.v[0xF] = 1
                }
                state[col_pos % 64][row_pos % 32] = (bit ^ state_bit) > 0;
            }
        }
    }
    /// Ex9E = Skip if key_pressed(hex(Vx)) //keypad is formed by numbers in hex
    fn if_key_pressed(&mut self, keys_pressed: &[bool; 16], x: u8) {
        if keys_pressed[self.v[x as usize] as usize] {
            self.program_counter = self.program_counter + 1
        }
    }
    /// ExA1 = Skip if !key_pressed(hex(Vx))
    fn if_not_key_pressed(&mut self, keys_pressed: &[bool; 16], x: u8) {
        if !keys_pressed[self.v[x as usize] as usize] {
            self.program_counter = self.program_counter + 1
        }
    }
    /// Fx07 = Vx = dt
    fn store_dt(&mut self, x: u8) {
        self.v[x as usize] = self.dt
    }
    /// Fx0A = Vx = block_until_keypress()
    fn wait_for_keypress(&mut self, x: u8, keys_pressed: &[bool; 16]) {
        match self.is_key_pressed_temp {
            Some(is_key_pressed_og) => {
                let mut modded = false; // We must not continue if not key press/release
                for (idx, key) in keys_pressed.iter().enumerate() {
                    if is_key_pressed_og[idx] != *key {
                        self.v[x as usize] = idx as u8;
                        modded = true;
                        self.is_key_pressed_temp = None; //Once the modification's been done correctly, we clean up for the next Fx0A
                        break;
                    }
                }
                if !modded {
                    self.program_counter = self.program_counter - 1
                }
            }
            None => {
                self.is_key_pressed_temp = Some(keys_pressed.clone());
                self.program_counter = self.program_counter - 1
            } //Store state before wait
        }
    }
    /// Fx15 = dt = Vx
    fn dt_from_reg(&mut self, x: u8) {
        self.dt = self.v[x as usize]
    }
    /// Fx18 = st = Vx
    fn st_from_reg(&mut self, x: u8) {
        self.st = self.v[x as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    mod fonts {
        use super::super::memory::Memory;
        use super::Cpu;
        #[test]
        fn load_fonts() {
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            for addr in 0x20..(0x20 + 0xF) {
                let res = mem.read(addr as u16);
                assert!(res.unwrap() > 0, "Value is not empty");
            }
        }
    }
    mod ops {
        use super::super::memory::Memory;
        use super::Cpu;
        #[test]
        fn ml_sub() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let test_addr: u16 = 0x400;
            cpu.ml_sub(test_addr);
            assert!(cpu.stack.len() > 0, "Stack should've been pushed");
            assert_eq!(
                cpu.program_counter,
                test_addr - 1,
                "Address should be set properly"
            );
            assert_eq!(
                cpu.stack[0], 0x200,
                "Check that the original address is in the stack"
            )
        }
        #[test]
        fn cls() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[true; 32]; 64];
            cpu.cls(&mut test_state);
            for item in test_state.iter().flat_map(|sub| sub.iter()) {
                assert_eq!(*item, false, "Array is not empty in a certain position")
            }
        }
        #[test]
        fn ret_sub() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let example_addr1: u16 = 0x400;
            let example_addr2: u16 = 0x500;
            cpu.stack.push(example_addr2); //Push trash so array is not at 0 all the time
            cpu.stack.push(example_addr1);
            cpu.ret_sub();
            assert_eq!(
                cpu.program_counter, example_addr1,
                "Did not pop the correct address the first time"
            );
            cpu.ret_sub();
            assert_eq!(
                cpu.program_counter, example_addr2,
                "Did not pop the correct address the second time"
            )
        }
        #[test]
        fn jump() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let example_addr: u16 = 0x400;
            cpu.jump(example_addr);
            assert_eq!(cpu.program_counter, example_addr - 1, "Wrong address set")
        }
        #[test]
        fn call_sub() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let test_addr: u16 = 0x400;
            cpu.call_sub(test_addr);
            assert!(cpu.stack.len() > 0, "Stack should've been pushed");
            assert_eq!(
                cpu.program_counter,
                test_addr - 1,
                "Address should be set properly"
            );
            assert_eq!(
                cpu.stack[0], 0x200,
                "Check that the original address is in the stack"
            )
        }
        #[test]
        fn if_reg_equals_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let instruction: u16 = 0x3404;
            let nn = (instruction & 0xFF) as u8;
            let reg = ((instruction & 0xF00) >> 8) as u8;
            cpu.v[4] = 4;
            let expected_pc = cpu.program_counter + 1;
            cpu.if_reg_equals_nn(reg, nn);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn if_not_reg_equals_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let instruction: u16 = 0x4405;
            let nn = (instruction & 0xFF) as u8;
            let reg = ((instruction & 0xF00) >> 8) as u8;
            cpu.v[4] = 4;
            let expected_pc = cpu.program_counter + 1;
            cpu.if_not_reg_equals_nn(reg, nn);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn if_reg_equals_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let instruction: u16 = 0x5450;
            let y = ((instruction & 0xF0) >> 4) as u8;
            let x = ((instruction & 0xF00) >> 8) as u8;
            cpu.v[4] = 4;
            cpu.v[5] = 4;
            let expected_pc = cpu.program_counter + 1;
            cpu.if_reg_equals_reg(x, y);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn reg_store_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            let nn: u8 = 10;
            cpu.reg_store_nn(x, nn);
            assert_eq!(
                cpu.v[x as usize], nn,
                "Register should be assigned properly"
            );
        }
        #[test]
        fn reg_add_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 3;
            let nn: u8 = 10;
            cpu.reg_add_nn(x, nn);
            assert_eq!(
                cpu.v[x as usize],
                nn + 3,
                "Register should be assigned properly"
            );
        }
        #[test]
        fn assign_reg_to_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 3;
            let y: u8 = 5;
            cpu.v[y as usize] = 10;
            cpu.assign_reg_to_reg(x, y);
            assert_eq!(
                cpu.v[x as usize], 10,
                "Register should be assigned properly"
            );
        }
        #[test]
        fn reg_or_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 0xF0;
            let y: u8 = 5;
            cpu.v[y as usize] = 0xF;
            cpu.reg_or_reg(x, y);
            assert_eq!(
                cpu.v[x as usize], 0xFF,
                "Register should be assigned properly"
            );
        }
        #[test]
        fn reg_and_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 0xF0;
            let y: u8 = 5;
            cpu.v[y as usize] = 0xF0;
            cpu.reg_and_reg(x, y);
            assert_eq!(
                cpu.v[x as usize], 0xF0,
                "Register should be assigned properly"
            );
        }
        #[test]
        fn reg_xor_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 0b0110;
            let y: u8 = 5;
            cpu.v[y as usize] = 0b1100;
            cpu.reg_xor_reg(x, y);
            assert_eq!(
                cpu.v[x as usize], 0b1010,
                "Register should be assigned properly"
            );
        }
        #[test]
        fn reg_plus_reg_normal() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 0b0010;
            let y: u8 = 5;
            cpu.v[y as usize] = 0b0010;
            cpu.reg_plus_reg(x, y);
            assert_eq!(
                cpu.v[x as usize], 0b0100,
                "Register should be assigned properly"
            );
            assert_eq!(cpu.v[0xF], 0, "Carry should be 0");
        }
        #[test]
        fn reg_plus_reg_carry() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 128;
            let y: u8 = 5;
            cpu.v[y as usize] = 128;
            cpu.reg_plus_reg(x, y);
            assert_eq!(cpu.v[x as usize], 0, "Register should be assigned properly");
            assert_eq!(cpu.v[0xF], 1, "Carry should be 1");
        }
        #[test]
        fn reg_minus_reg_normal() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 10;
            let y: u8 = 5;
            cpu.v[y as usize] = 5;
            cpu.reg_minus_reg(x, y);
            assert_eq!(cpu.v[x as usize], 5, "Register should be assigned properly");
            assert_eq!(cpu.v[0xF], 1, "Borrow should be 1");
        }
        #[test]
        fn reg_minus_reg_borrow() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 5;
            let y: u8 = 5;
            cpu.v[y as usize] = 10;
            cpu.reg_minus_reg(x, y);
            assert_eq!(
                cpu.v[x as usize], 251,
                "Register should be assigned properly"
            );
            assert_eq!(cpu.v[0xF], 0, "Borrow should be 0");
        }
        #[test]
        fn reg_shift_right_normal() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 10;
            cpu.reg_shift_right(x);
            assert_eq!(cpu.v[x as usize], 5, "Register should be assigned properly");
            assert_eq!(cpu.v[0xF], 0, "Overflow should be 0");
        }
        #[test]
        fn reg_shift_right_overflow() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 5;
            cpu.reg_shift_right(x);
            assert_eq!(cpu.v[x as usize], 2, "Register should be assigned properly");
            assert_eq!(cpu.v[0xF], 1, "Overflow should be 1");
        }
        #[test]
        fn reverse_reg_minus_reg_normal() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 5;
            let y: u8 = 5;
            cpu.v[y as usize] = 10;
            cpu.reverse_reg_minus_reg(x, y);
            assert_eq!(cpu.v[x as usize], 5, "Register should be assigned properly");
            assert_eq!(cpu.v[0xF], 1, "Borrow should be 1");
        }
        #[test]
        fn reverse_reg_minus_reg_borrow() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 10;
            let y: u8 = 5;
            cpu.v[y as usize] = 5;
            cpu.reverse_reg_minus_reg(x, y);
            assert_eq!(
                cpu.v[x as usize], 251,
                "Register should be assigned properly"
            );
            assert_eq!(cpu.v[0xF], 0, "Borrow should be 0");
        }
        #[test]
        fn reg_shift_left_normal() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 0x0F;
            cpu.reg_shift_left(x);
            assert_eq!(
                cpu.v[x as usize], 30,
                "Register should be assigned properly"
            );
            assert_eq!(cpu.v[0xF], 0, "Overflow should be 0");
        }
        #[test]
        fn reg_shift_left_overflow() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 4;
            cpu.v[x as usize] = 0xF0;
            cpu.reg_shift_left(x);
            assert_eq!(
                cpu.v[x as usize], 224,
                "Register should be assigned properly"
            );
            assert_eq!(cpu.v[0xF], 1, "Overflow should be 1");
        }
        #[test]
        fn if_not_reg_equals_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let instruction: u16 = 0x5450;
            let y = ((instruction & 0xF0) >> 4) as u8;
            let x = ((instruction & 0xF00) >> 8) as u8;
            cpu.v[4] = 4;
            cpu.v[5] = 5;
            let expected_pc = cpu.program_counter + 1;
            cpu.if_not_reg_equals_reg(x, y);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn store_addr() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let nnn: u16 = 0xF0;
            cpu.store_addr(nnn);
            assert_eq!(cpu.i, nnn, "Address should be incremented properly");
        }
        #[test]
        fn reg_plus_nnn_jump() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let nnn: u16 = 0xF0;
            cpu.v[0] = 4;
            cpu.reg_plus_nnn_jump(nnn);
            assert_eq!(
                cpu.program_counter + 1,
                nnn + 4,
                "Address should be changed"
            );
        }
        /*#[test] Not run because i can't guarantee it doesn't hit 0 occasionallly
        fn random() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let nn: u8 = 0x01;
            let x: u8 = 0x4;
            cpu.random(x, nn);
            assert!(
                cpu.v[x as usize] != 0,
                "Address should not be anything but 0"
            );
        }*/
        #[test]
        fn draw_sprite_no_overflow() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 1;
            let y: u8 = 2;
            cpu.v[x as usize] = 1;
            cpu.v[y as usize] = 3;
            let n: u8 = 5;
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            cpu.i = 0x20;
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            cpu.draw_sprite(x, y, n, &mut test_state, &mut mem);
            /*let mut string: String = "".to_owned();
            let mut table: Vec<String> = Vec::new();
            for y in 0..32 as usize {
                for x in 0..64 as usize {
                    string = string + &((test_state[x][y] as u8).to_string())[..]
                }
                table.push(string.clone());
                string = "".to_owned();
            }
            for row in table {
                println!("{:?}", row);
            }*/
            assert_eq!(test_state[1][3], true, "Top Left Corner should be true");
            assert_eq!(test_state[4][7], true, "Bottom Right Corner should be true");
            assert_eq!(cpu.v[0xF], 0, "Overwrite should be 0");
        }
        #[test]
        fn draw_sprite_overflow() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 1;
            let y: u8 = 2;
            cpu.v[x as usize] = 62;
            cpu.v[y as usize] = 30;
            let n: u8 = 5;
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            cpu.i = 0x20;
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            cpu.draw_sprite(x, y, n, &mut test_state, &mut mem);
            /*let mut string: String = "".to_owned();
            let mut table: Vec<String> = Vec::new();
            for y in 0..32 as usize {
                for x in 0..64 as usize {
                    string = string + &((test_state[x][y] as u8).to_string())[..]
                }
                table.push(string.clone());
                string = "".to_owned();
            }
            for row in table {
                println!("{:?}", row);
            }*/
            assert_eq!(test_state[62][30], true, "Top Left Corner should be true");
            assert_eq!(test_state[1][2], true, "Bottom Right Corner should be true")
        }
        #[test]
        fn draw_sprite_overwrite() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 1;
            let y: u8 = 2;
            cpu.v[x as usize] = 1;
            cpu.v[y as usize] = 3;
            let n: u8 = 5;
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            cpu.i = 0x20;
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            cpu.draw_sprite(x, y, n, &mut test_state, &mut mem);
            cpu.draw_sprite(x, y, n, &mut test_state, &mut mem);
            /*let mut string: String = "".to_owned();
            let mut table: Vec<String> = Vec::new();
            for y in 0..32 as usize {
                for x in 0..64 as usize {
                    string = string + &((test_state[x][y] as u8).to_string())[..]
                }
                table.push(string.clone());
                string = "".to_owned();
            }
            for row in table {
                println!("{:?}", row);
            }*/
            assert_eq!(test_state[1][3], false, "Top Left Corner should be false");
            assert_eq!(
                test_state[4][7], false,
                "Bottom Right Corner should befalse"
            );
            assert_eq!(cpu.v[0xF], 1, "Overwrite should be 1");
        }
        #[test]
        fn if_key_pressed_true() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[x as usize] = 0x3;
            let mut is_key_pressed: [bool; 16] = [false; 16];
            is_key_pressed[x as usize] = true;
            let expected_pc = cpu.program_counter + 1;
            cpu.if_key_pressed(&is_key_pressed, x);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn if_key_pressed_false() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[x as usize] = 0x3;
            let mut is_key_pressed: [bool; 16] = [false; 16];
            is_key_pressed[x as usize] = false;
            let expected_pc = cpu.program_counter;
            cpu.if_key_pressed(&is_key_pressed, x);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn if_not_key_pressed_true() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[x as usize] = 0x3;
            let mut is_key_pressed: [bool; 16] = [false; 16];
            is_key_pressed[x as usize] = false;
            let expected_pc = cpu.program_counter + 1;
            cpu.if_not_key_pressed(&is_key_pressed, x);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn if_not_key_pressed_false() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[x as usize] = 0x3;
            let mut is_key_pressed: [bool; 16] = [false; 16];
            is_key_pressed[x as usize] = true;
            let expected_pc = cpu.program_counter;
            cpu.if_not_key_pressed(&is_key_pressed, x);
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn store_dt() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.dt = 20;
            cpu.store_dt(x);
            assert_eq!(
                cpu.v[x as usize], 20,
                "Address should be incremented properly"
            );
        }
        #[test]
        fn wait_for_keypress_press() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 0x3;
            let mut is_key_pressed: [bool; 16] = [false; 16];
            let expected_pc = cpu.program_counter + 1;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            assert_eq!(
                cpu.program_counter,
                expected_pc - 1,
                "Address shouldn't be incremented yet"
            );
            is_key_pressed[0xE] = true;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
            assert_eq!(
                cpu.v[x as usize], 0xE,
                "Register should have the pressed key saved"
            );
        }
        #[test]
        fn wait_for_keypress_release() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x: u8 = 0x3;
            let mut is_key_pressed: [bool; 16] = [false; 16];
            is_key_pressed[0xE] = true;
            let expected_pc = cpu.program_counter + 1;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            assert_eq!(
                cpu.program_counter,
                expected_pc - 1,
                "Address shouldn't be incremented yet"
            );
            is_key_pressed[0xE] = false;
            cpu.wait_for_keypress(x, &is_key_pressed);
            cpu.program_counter = cpu.program_counter + 1;
            assert_eq!(
                cpu.program_counter, expected_pc,
                "Address should be incremented properly"
            );
            assert_eq!(
                cpu.v[x as usize], 0xE,
                "Register should have the released key saved"
            );
        }
    }
}
