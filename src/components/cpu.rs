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
    pub v: [u8; 16],
    /// Subroutine stack
    ///
    /// When 2NNN or 0NNN is called, the current PC should be pushed to it.
    /// When 00EE is called, the PC should be set to a pop of it.
    /// Uses Vec because it already implements push() and pop().
    ///
    /// In reality, the stack would have a limited size based on physical constraints.
    /// Vec is infinite
    pub stack: Vec<u16>,
    /// Program counter
    ///
    /// It tells us what the current instruction to be executed is.
    /// Always set to 0x200 when execution begins
    /// (Only valid for regular CHIP-8 implementations, others may vary).
    pub program_counter: u16,
    /// Address register
    ///
    /// Used with read and write operations.
    /// Due to the way op addresses work, only 12 bits can be actually loaded.
    pub i: u16,
    /// Delay timer
    ///
    /// Counts down at a rate of 1 per second until 0 is reached.
    /// Set by instruction Fx15 and read by using Fx07.
    pub dt: u8,
    /// Sound timer
    ///
    /// Counts down at 60 hertz just like the Delay timer.
    /// While it is active, a sound will ring.
    ///
    /// The waveform and frequency is unspecified.
    /// Set by instruction Fx18.
    /// Will do nothing if set to 0x01
    pub st: u8,
    /// Used to generate random numbers for Cxnn
    pub rng: rand::rngs::ThreadRng,
    /// Used by the Fx0A instruction to be able to compare changes in state
    pub is_key_pressed_temp: Option<[bool; 16]>,
    /// In some implementations, Fx55 and Fx65 don't change the value of I
    pub store_load_quirk: bool,
    /// In some implementations x is shifted, in others, y is
    pub shift_y: bool,
    /// Used to notify the drawing code that changes were made to the screen
    pub drawn: bool,
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
            store_load_quirk: false,
            shift_y: false,
            drawn: false,
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
*   Fx65 = [V0, V..., Vx] = [I, I..., I + x]; I = I + x + 1
*/

struct Execution {
    ms: usize,
    function: &'static str,
}

impl Cpu {
    /// Each subarray is a different number, from 0x0 to 0xF
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

    /// Run one instruction on the CPU
    pub fn run_cycle(
        &mut self,
        mem: &mut memory::Memory,
        state: &mut [[bool; 32]; 64],
        keys_pressed: &[bool; 16],
    ) -> Result<&'static str, &'static str> {
        let op_code = mem
            .read(self.program_counter)
            .expect("run_cycle: Failed to read op_code");
        let first_nibble = (op_code >> 12) as u8;
        let nnn = op_code & 0xFFF;
        let nn = (op_code & 0xFF) as u8;
        let x = ((op_code & 0xF00) >> 8) as u8;
        let y = ((op_code & 0xF0) >> 4) as u8;
        let n = (op_code & 0xF) as u8;
        #[cfg(feature = "debug")]
        println!(
            "<< {:04x}: {:04x} >>",
            (self.program_counter) - 0x200,
            op_code
        );
        let result = match first_nibble {
            0x0 => match op_code {
                0x00E0 => Ok(self.cls(state)),
                0x00EE => Ok(self.ret_sub()),
                _ => self.ml_sub(nnn),
            },
            0x1 => Ok(self.jump(nnn)),
            0x2 => Ok(self.call_sub(nnn)),
            0x3 => Ok(self.if_reg_equals_nn(x, nn)),
            0x4 => Ok(self.if_not_reg_equals_nn(x, nn)),
            0x5 => match n {
                0 => Ok(self.if_reg_equals_reg(x, y)),
                _ => Err("5xy0: Tail nibble was not 0x0"),
            },
            0x6 => Ok(self.reg_store_nn(x, nn)),
            0x7 => Ok(self.reg_add_nn(x, nn)),
            0x8 => match n {
                0x0 => Ok(self.assign_reg_to_reg(x, y)),
                0x1 => Ok(self.reg_or_reg(x, y)),
                0x2 => Ok(self.reg_and_reg(x, y)),
                0x3 => Ok(self.reg_xor_reg(x, y)),
                0x4 => Ok(self.reg_plus_reg(x, y)),
                0x5 => Ok(self.reg_minus_reg(x, y)),
                0x6 => Ok(self.reg_shift_right(x, y)),
                0x7 => Ok(self.reverse_reg_minus_reg(x, y)),
                0xE => Ok(self.reg_shift_left(x, y)),
                _ => Err("n was not in the expected values for 0x8... ops"),
            },
            0x9 => match n {
                0 => Ok(self.if_not_reg_equals_reg(x, y)),
                _ => Err("n was not in the expected values for 0x9... ops"),
            },
            0xA => Ok(self.store_addr(nnn)),
            0xB => Ok(self.reg_plus_nnn_jump(nnn)),
            0xC => Ok(self.random(x, nn)),
            0xD => Ok(self.draw_sprite(x, y, n, state, mem)),
            0xE => match nn {
                0x9E => Ok(self.if_key_pressed(keys_pressed, x)),
                0xA1 => Ok(self.if_not_key_pressed(keys_pressed, x)),
                _ => Err("nn was not in the expected values for 0xE... ops"),
            },
            0xF => match nn {
                0x07 => Ok(self.store_dt(x)),
                0x0A => Ok(self.wait_for_keypress(x, keys_pressed)),
                0x15 => Ok(self.dt_from_reg(x)),
                0x18 => Ok(self.st_from_reg(x)),
                0x1E => Ok(self.add_reg_to_i(x)),
                0x29 => Ok(self.get_sprite_address(x)),
                0x33 => Ok(self.get_bcd(x, mem)),
                0x55 => Ok(self.store_regs(x, mem)),
                0x65 => Ok(self.load_regs(x, mem)),
                _ => Err("nn was not in the expected values for 0xF... ops"),
            },
            _ => Err("first_nibble bigger than 0xF"),
        };
        self.program_counter += 2;
        return result;
    }
    /// Used to load the fonts in the default location so that they can be used by Dxyn/draw_sprite()
    pub fn write_fonts_to_mem(mem: &mut memory::Memory) {
        for (idx, sprite) in Cpu::FONT.iter().flatten().enumerate() {
            /*let res = mem.unbound_write((idx + 0x20) as u16, *sprite);
            match res {
                Err(err) => panic!("{}", err),
                _ => (),
            }*/
            mem.space[idx + 0x20] = (sprite >> 8) as u8;
        }
    }
    /// 0nnn - Execute machine language subroutine at nnn
    /// Implemented same as 2nnn in this case
    fn ml_sub(&mut self, addr: u16) -> Result<&'static str, &'static str> {
        match addr {
            0 => return Err("Failed test run"),
            _ => {
                self.call_sub(addr);
                return Ok("0nnn");
            }
        }
    }
    /// 00E0 - cls()
    fn cls(&self, state: &mut [[bool; 32]; 64]) -> &'static str {
        *state = [[false; 32]; 64];
        return "0E00";
    }
    /// 00EE - Return from subroutine
    fn ret_sub(&mut self) -> &'static str {
        let popped_addr = self.stack.pop().expect("00EE: No addresses to pop");
        self.program_counter = popped_addr;
        return "00EE";
    }
    /// 1nnn - Jump to nnn
    fn jump(&mut self, addr: u16) -> &'static str {
        self.program_counter = addr - 2;
        return "1nnn";
    }
    /// 2nnn - Execute subroutine at nnn
    fn call_sub(&mut self, addr: u16) -> &'static str {
        self.stack.push(self.program_counter);
        self.program_counter = addr - 2;
        return "2nnn";
    }
    /// 3xnn - Skip if Vx == nn - OK
    fn if_reg_equals_nn(&mut self, x: u8, nn: u8) -> &'static str {
        let vx = self.v[x as usize];
        if vx == nn {
            self.program_counter += 2
        };
        #[cfg(feature = "debug")]
        println!("{} == {}: {}", vx, nn, vx == nn);
        return "3xnn";
    }
    /// 4xnn - Skip if Vx != nn
    fn if_not_reg_equals_nn(&mut self, x: u8, nn: u8) -> &'static str {
        let vx = self.v[x as usize];
        if vx != nn {
            self.program_counter += 2
        }

        return "4xnn";
    }
    /// 5xy0 - Skip if Vx == Vy
    fn if_reg_equals_reg(&mut self, x: u8, y: u8) -> &'static str {
        if self.v[x as usize] == self.v[y as usize] {
            self.program_counter += 2
        }
        #[cfg(feature = "debug")]
        println!(
            "{} == {}: {}",
            self.v[x as usize],
            self.v[y as usize],
            self.v[x as usize] == self.v[y as usize]
        );
        return "5xy0";
    }
    /// 6xnn - Vx = nn - OK
    fn reg_store_nn(&mut self, x: u8, nn: u8) -> &'static str {
        self.v[x as usize] = nn;
        #[cfg(feature = "debug")]
        println!("V{} = {}", x, nn);
        return "6xnn";
    }
    /// 7xnn - Vx = Vx + nn; Overflows but doesn't set flag
    fn reg_add_nn(&mut self, x: u8, nn: u8) -> &'static str {
        #[cfg(feature = "debug")]
        let old_v = self.v[x as usize];
        self.v[x as usize] = self.v[x as usize].wrapping_add(nn);
        #[cfg(feature = "debug")]
        println!("{} + {} = {}", old_v, nn, self.v[x as usize]);
        return "7xnn";
    }
    /// 8xy0 - Vx = Vy
    fn assign_reg_to_reg(&mut self, x: u8, y: u8) -> &'static str {
        self.v[x as usize] = self.v[y as usize];
        #[cfg(feature = "debug")]
        println!(
            "V{}: {} = V{}: {}",
            x, self.v[x as usize], y, self.v[y as usize]
        );
        return "8xy0";
    }
    /// 8xy1 - Vx = Vx | Vy
    fn reg_or_reg(&mut self, x: u8, y: u8) -> &'static str {
        self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
        return "8xy1";
    }
    /// 8xy2 - Vx = Vx & Vy
    fn reg_and_reg(&mut self, x: u8, y: u8) -> &'static str {
        self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
        return "8xy2";
    }
    /// 8xy3 - Vx = Vx ^ Vy
    fn reg_xor_reg(&mut self, x: u8, y: u8) -> &'static str {
        #[cfg(feature = "debug")]
        let old_x = self.v[x as usize];
        self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
        #[cfg(feature = "debug")]
        println!(
            "{:08b} = V{}: {:08b} ^ v{}: {:08b}",
            self.v[x as usize], x, old_x, y, self.v[y as usize]
        );
        return "8xy3";
    }
    /// 8xy4 - Vx = Vx + Vy; VF = Carry?
    fn reg_plus_reg(&mut self, x: u8, y: u8) -> &'static str {
        let result = self.v[x as usize].overflowing_add(self.v[y as usize]);
        self.v[x as usize] = result.0;
        self.v[0xF] = result.1 as u8;
        return "8xy4";
    }
    /// 8xy5 - Vx = Vx - Vy; VF = 0 if borrow else 1
    fn reg_minus_reg(&mut self, x: u8, y: u8) -> &'static str {
        let result = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        self.v[x as usize] = result.0;
        self.v[0xF] = (!result.1) as u8;
        return "8xy5";
    }
    /// 8xy6 - Vx = Vy >> 1; VF = Vy & 1
    fn reg_shift_right(&mut self, x: u8, y: u8) -> &'static str {
        if self.shift_y {
            self.v[0xF] = self.v[y as usize] & 1;
            self.v[x as usize] = self.v[y as usize] >> 1;
        } else {
            self.v[0xF] = self.v[x as usize] & 1;
            self.v[x as usize] = self.v[x as usize] >> 1;
        }
        return "8xy6";
    }
    /// 8xy7 - Vx = Vy - Vx; VF = Borrow?
    fn reverse_reg_minus_reg(&mut self, x: u8, y: u8) -> &'static str {
        let result = self.v[y as usize].overflowing_sub(self.v[x as usize]);
        self.v[x as usize] = result.0;
        self.v[0xF] = (!result.1) as u8;
        return "8xy7";
    }
    /// 8xyE - Vx = Vy << 1; VF = Vy >> 7
    fn reg_shift_left(&mut self, x: u8, y: u8) -> &'static str {
        if self.shift_y {
            self.v[0xF] = self.v[y as usize] >> 7;
            self.v[x as usize] = self.v[y as usize] << 1;
        } else {
            self.v[0xF] = self.v[x as usize] >> 7;
            self.v[x as usize] = self.v[x as usize] << 1;
        }
        return "8xyE";
    }
    /// 9xy0 - Skip if Vx != Vy - OK
    fn if_not_reg_equals_reg(&mut self, x: u8, y: u8) -> &'static str {
        if self.v[x as usize] != self.v[y as usize] {
            self.program_counter += 2;
        }
        #[cfg(feature = "debug")]
        println!("{:x} != {:x}", self.v[x as usize], self.v[y as usize]);
        return "9xy0";
    }
    /// Annn - I = nnn
    fn store_addr(&mut self, nnn: u16) -> &'static str {
        self.i = nnn;
        #[cfg(feature = "debug")]
        println!("I = {:x}", (nnn - 0x200) * 2);
        return "Annn";
    }
    /// Bnnn - Jump to nnn + V0
    fn reg_plus_nnn_jump(&mut self, nnn: u16) -> &'static str {
        self.program_counter = self.v[0] as u16 + nnn - 2;
        return "Bnnn";
    }
    /// Cxnn = Vx = Rand() & nn
    fn random(&mut self, x: u8, nn: u8) -> &'static str {
        self.v[x as usize] = self.rng.gen::<u8>() & nn;
        return "Cxnn";
    }
    /// Dxyn = draw(x: Vx, y: Vy, sprite: sprite(sprite_height: n, sprite_addr: I)); VF = Pixels unset?
    fn draw_sprite(
        &mut self,
        x: u8,
        y: u8,
        n: u8,
        state: &mut [[bool; 32]; 64],
        mem: &mut memory::Memory,
    ) -> &'static str {
        self.v[0xF] = 0;
        for sprite_row in 0..n {
            let row_pos = (self.v[y as usize] + sprite_row) as usize;
            /*let sprite_value = (mem
            .read(self.i + sprite_row as u16)
            .expect("Dxyn: Failed to read memory")) as u8;
            */
            let sprite_value = mem.space[(self.i + sprite_row as u16) as usize];
            #[cfg(feature = "debug")]
            println!("{:08b}", sprite_value);
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
        #[cfg(feature = "debug")]
        let mut string: String = "".to_owned();
        #[cfg(feature = "debug")]
        let mut table: Vec<String> = Vec::new();
        #[cfg(feature = "debug")]
        for y in 0..32 as usize {
            for x in 0..64 as usize {
                string = string + &((state[x][y] as u8).to_string())[..]
            }
            table.push(string.clone());
            string = "".to_owned();
        }
        #[cfg(feature = "debug")]
        for row in table {
            println!("{:?}", row);
        }
        return "Dxyn";
    }
    /// Ex9E = Skip if key_pressed(hex(Vx)) //keypad is formed by numbers in hex
    fn if_key_pressed(&mut self, keys_pressed: &[bool; 16], x: u8) -> &'static str {
        if keys_pressed[self.v[x as usize] as usize] {
            self.program_counter += 2
        }
        return "Ex9E";
    }
    /// ExA1 = Skip if !key_pressed(hex(Vx))
    fn if_not_key_pressed(&mut self, keys_pressed: &[bool; 16], x: u8) -> &'static str {
        if !keys_pressed[self.v[x as usize] as usize] {
            self.program_counter += 2
        }
        return "ExA1";
    }
    /// Fx07 = Vx = dt
    fn store_dt(&mut self, x: u8) -> &'static str {
        self.v[x as usize] = self.dt;
        return "Fx07";
    }
    /// Fx0A = Vx = block_until_keypress()
    fn wait_for_keypress(&mut self, x: u8, keys_pressed: &[bool; 16]) -> &'static str {
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
                    self.program_counter -= 2
                }
            }
            None => {
                self.is_key_pressed_temp = Some(keys_pressed.clone());
                self.program_counter -= 2
            } //Store state before wait
        }
        return "Fx0A";
    }
    /// Fx15 = dt = Vx - OK
    fn dt_from_reg(&mut self, x: u8) -> &'static str {
        self.dt = self.v[x as usize];
        return "Fx15";
    }
    /// Fx18 = st = Vx
    fn st_from_reg(&mut self, x: u8) -> &'static str {
        self.st = self.v[x as usize];
        return "Fx18";
    }
    /// Fx1E = I = I + Vx
    fn add_reg_to_i(&mut self, x: u8) -> &'static str {
        let old_i = self.i;
        self.i = self.i + self.v[x as usize] as u16;
        #[cfg(feature = "debug")]
        println!("{} + {} = {}", old_i, self.v[x as usize], self.i);
        return "Fx1E";
    }
    /// Fx29 = I = addr(sprite(Vx))
    ///
    /// Sprite address is internal to the interpreter, it'll have to be placed within 0x000 and 0x1FF
    fn get_sprite_address(&mut self, x: u8) -> &'static str {
        self.i = (0x20 + (self.v[x as usize] * 5)) as u16;
        return "Fx29";
    }
    /// Fx33 = [I, I+1, I+2] = bcd(hex(Vx))
    fn get_bcd(&mut self, x: u8, mem: &mut memory::Memory) -> &'static str {
        let mut number = self.v[x as usize];
        let mut stack_of_digits: Vec<u8> = Vec::new();
        while number > 0 {
            stack_of_digits.push(number % 10);
            number = number / 10;
        }
        while stack_of_digits.len() < 3 {
            stack_of_digits.push(0);
        }
        stack_of_digits.reverse();
        for (idx, digit) in stack_of_digits.iter().enumerate() {
            //mem.write(self.i + idx as u16, *digit as u16)
            //    .expect("Fx33: Failed to write to memory");
            mem.space[self.i as usize + idx] = *digit
        }
        return "Fx33";
    }
    /// Fx55 = [I, I..., I + x] = [V0, V..., Vx]; I = I + x + 1
    fn store_regs(&mut self, x: u8, mem: &mut memory::Memory) -> &'static str {
        for reg in 0..=x {
            let reg_addr = self.i + reg as u16;
            //mem.write(reg_addr, self.v[reg as usize] as u16)
            //    .expect("Fx55: Failed to write to memory");
            mem.space[reg_addr as usize] = self.v[reg as usize];
            #[cfg(feature = "debug")]
            println!(
                "I + {}: {:04x} = {}",
                reg,
                (self.i - 0x200 + reg as u16),
                self.v[reg as usize]
            )
        }
        if !self.store_load_quirk {
            self.i = self.i + x as u16 + 1;
        }
        return "Fx55";
    }
    /// Fx65 = [V0, V..., Vx] = [I, I..., I + x]; I = I + x + 1
    fn load_regs(&mut self, x: u8, mem: &mut memory::Memory) -> &'static str {
        for reg in 0..=x {
            let reg_addr = self.i + reg as u16;
            //self.v[reg as usize] = mem.read(reg_addr).expect("Fx65: Failed to read memory") as u8;
            self.v[reg as usize] = mem.space[reg_addr as usize];
            #[cfg(feature = "debug")]
            println!(
                "I + {}: {:04x} = {}",
                reg,
                (self.i - 0x200 + reg as u16),
                self.v[reg as usize]
            )
        }
        if !self.store_load_quirk {
            self.i = self.i + x as u16 + 1;
        }
        return "Fx65";
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
            cpu.reg_shift_right(x, 0);
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
            cpu.reg_shift_right(x, 0);
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
            cpu.reg_shift_left(x, 0);
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
            cpu.reg_shift_left(x, 0);
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
            assert_eq!(
                cpu.is_key_pressed_temp, None,
                "Temp is_key_pressed should be empty"
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
            assert_eq!(
                cpu.is_key_pressed_temp, None,
                "Temp is_key_pressed should be empty"
            );
        }
        #[test]
        fn dt_from_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[x as usize] = 10;
            cpu.dt_from_reg(x);
            assert_eq!(cpu.dt, 10, "Delay timer should be set properly");
        }
        #[test]
        fn st_from_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[x as usize] = 10;
            cpu.st_from_reg(x);
            assert_eq!(cpu.st, 10, "Sound timer should be set properly");
        }
        #[test]
        fn add_reg_to_i() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[x as usize] = 10;
            cpu.i = 10;
            cpu.add_reg_to_i(x);
            assert_eq!(cpu.i, 20, "I should be incremented properly");
        }
        #[test]
        fn get_sprite_address() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            let x = 0x3;
            cpu.v[x as usize] = 1;
            cpu.i = 0;
            cpu.get_sprite_address(x);
            assert_eq!(cpu.i, 0x25, "I should be set properly to 0x25");
            assert_eq!(
                mem.read(cpu.i).expect("Failed to read memory"),
                0x20,
                "First row failed"
            );
            assert_eq!(
                mem.read(cpu.i + 1).expect("Failed to read memory"),
                0x60,
                "Second row failed"
            );
            assert_eq!(
                mem.read(cpu.i + 2).expect("Failed to read memory"),
                0x20,
                "Third row failed"
            );
            assert_eq!(
                mem.read(cpu.i + 3).expect("Failed to read memory"),
                0x20,
                "Fourth row failed"
            );
            assert_eq!(
                mem.read(cpu.i + 4).expect("Failed to read memory"),
                0x70,
                "Fifth row failed"
            );
            assert_eq!(
                mem.read(cpu.i + 5).expect("Failed to read memory"),
                0xF0,
                "First row of next sprite failed"
            );
            cpu.v[x as usize] = 0;
            cpu.i = 0;
            cpu.get_sprite_address(x);
            assert_eq!(cpu.i, 0x20, "I should be set properly to 0x20");
        }
        #[test]
        fn get_bcd() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            let x = 0x3;
            cpu.v[x as usize] = 123;
            cpu.i = 0x400;
            cpu.get_bcd(x, &mut mem);
            assert_eq!(mem.read(cpu.i).unwrap(), 1, "I should be 1");
            assert_eq!(mem.read(cpu.i + 1).unwrap(), 2, "I + 1 should be 2");
            assert_eq!(mem.read(cpu.i + 2).unwrap(), 3, "I + 2 should be 3");
        }
        #[test]
        fn store_regs() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            let x = 0x3;
            cpu.v[0] = 1;
            cpu.v[1] = 2;
            cpu.v[2] = 3;
            cpu.v[x as usize] = 4;
            cpu.i = 0x400;
            let original_i = 0x400;
            cpu.store_regs(x, &mut mem);
            assert_eq!(mem.read(original_i).unwrap(), 1, "I should be 1");
            assert_eq!(mem.read(original_i + 1).unwrap(), 2, "I + 1 should be 2");
            assert_eq!(mem.read(original_i + 2).unwrap(), 3, "I + 2 should be 3");
            assert_eq!(
                mem.read(original_i + x as u16).unwrap(),
                4,
                "I + x should be 3"
            );
            assert_eq!(
                cpu.i,
                original_i + x as u16 + 1,
                "I should be incremented properly"
            )
        }
        #[test]
        fn load_regs() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            let x = 0x3;
            cpu.i = 0x400;
            let original_i = 0x400;
            for reg in 0..=x {
                let reg_addr = original_i + reg as u16;
                mem.write(reg_addr, 55).unwrap();
            }
            cpu.load_regs(x, &mut mem);
            assert_eq!(cpu.v[0], 55, "V0 should be 55");
            assert_eq!(cpu.v[1], 55, "V1 should be 55");
            assert_eq!(cpu.v[2], 55, "V2 should be 55");
            assert_eq!(cpu.v[x as usize], 55, "Vx should be 55");
            assert_eq!(
                cpu.i,
                original_i + x as u16 + 1,
                "I should be incremented properly"
            )
        }
    }
    mod cycle {
        use super::super::memory::Memory;
        use super::Cpu;
        #[test]
        fn ml_sub() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x0100)
                .expect("Example instruction did not write correctly");
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "0nnn");
        }
        #[test]
        fn cls() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x00E0)
                .expect("Example instruction did not write correctly");
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "0E00");
        }
        #[test]
        fn ret_sub() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x00EE)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "00EE");
        }
        #[test]
        fn jump() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x1400)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "1nnn");
        }
        #[test]
        fn call_sub() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x2400)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "2nnn");
        }
        #[test]
        fn if_reg_equals_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x3410)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "3xnn");
        }
        #[test]
        fn if_not_reg_equals_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x4410)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "4xnn");
        }
        #[test]
        fn if_reg_equals_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x5120)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "5xy0");
        }
        #[test]
        fn reg_store_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x64FF)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "6xnn");
        }
        #[test]
        fn reg_add_nn() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x7401)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "7xnn");
        }
        #[test]
        fn assign_reg_to_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8120)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy0");
        }
        #[test]
        fn reg_or_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8121)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy1");
        }
        #[test]
        fn reg_and_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8122)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy2");
        }
        #[test]
        fn reg_xor_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8123)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy3");
        }
        #[test]
        fn reg_plus_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8124)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy4");
        }
        #[test]
        fn reg_minus_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8125)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy5");
        }
        #[test]
        fn reg_shift_right() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8126)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy6");
        }
        #[test]
        fn reverse_reg_minus_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x8127)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xy7");
        }
        #[test]
        fn reg_shift_left() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x812E)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "8xyE");
        }
        #[test]
        fn if_not_reg_equals_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0x9120)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "9xy0");
        }
        #[test]
        fn store_addr() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xA120)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Annn");
        }
        #[test]
        fn reg_plus_nnn_jump() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xB120)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Bnnn");
        }
        #[test]
        fn random() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xC120)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Cxnn");
        }
        #[test]
        fn draw_sprite() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xD121)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Dxyn");
        }
        #[test]
        fn if_key_pressed() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xE09E)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Ex9E");
        }
        #[test]
        fn if_not_key_pressed() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xE0A1)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "ExA1");
        }
        #[test]
        fn store_dt() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xF007)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx07");
        }
        #[test]
        fn wait_for_keypress() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let mut is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xF00A)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx0A");
        }
        #[test]
        fn dt_from_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xF015)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx15");
        }
        #[test]
        fn st_from_reg() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xF018)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx18");
        }
        #[test]
        fn add_reg_to_i() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xF01E)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx1E");
        }
        #[test]
        fn get_sprite_address() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xF029)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx29");
        }
        #[test]
        fn get_bcd() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            Cpu::write_fonts_to_mem(&mut mem);
            let x = 0x3;
            cpu.v[x as usize] = 123;
            cpu.i = 0x400;
            mem.write(0x200, 0xF033)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx33");
        }
        #[test]
        fn store_regs() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            let x = 0x3;
            cpu.v[0] = 1;
            cpu.v[1] = 2;
            cpu.v[2] = 3;
            cpu.v[x as usize] = 4;
            cpu.i = 0x400;
            mem.write(0x200, 0xF055)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx55");
        }
        #[test]
        fn load_regs() {
            let mut cpu = Cpu {
                ..Default::default()
            };
            let mut test_state: [[bool; 32]; 64] = [[false; 32]; 64];
            let is_key_pressed: [bool; 16] = [false; 16];
            let mut mem = Memory {
                ..Default::default()
            };
            mem.write(0x200, 0xF065)
                .expect("Example instruction did not write correctly");
            cpu.stack.push(0x200);
            let result = cpu
                .run_cycle(&mut mem, &mut test_state, &is_key_pressed)
                .expect("Cycle did not run correctly");
            assert_eq!(result, "Fx65");
        }
    }
}
