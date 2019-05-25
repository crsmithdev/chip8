#![deny(clippy::cognitive_complexity)]

extern crate rand;

use rom;
use std::error::Error;
use std::fmt;
use std::string::String;

#[derive(Clone, Copy, Debug)]
pub enum Chip8Error {
    UnknownInstructionError,
    AddressOutOfRangeError,
    ProgramLoadError,
    StackOverflowError,
    StackUnderflowError,
}

pub struct DecodedInstruction {
    pub operation: String,
    pub params: String,
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for Chip8Error {
    fn description(&self) -> &str {
        match *self {
            Chip8Error::UnknownInstructionError => "instruction unknown",
            Chip8Error::AddressOutOfRangeError => "memory address out of range",
            Chip8Error::ProgramLoadError => "error loading program rom",
            Chip8Error::StackOverflowError => "stack overflow",
            Chip8Error::StackUnderflowError => "stack underflow",
        }
    }
}

pub struct Chip8State {
    pub video: [u8; Chip8::VIDEO_SIZE],
    pub memory: [u8; Chip8::MEMORY_SIZE],
    pub v: [u8; 16],
    pub stack: [u16; 16],
    pub keys: [bool; 16],
    pub pc: usize,
    pub sp: usize,
    pub i: usize,
    pub dt: u8,
    pub st: u8,
    pub error: Option<Chip8Error>,
}

impl Default for Chip8State {
    fn default() -> Self {
        Chip8State {
            pc: Self::PROGRAM_START,
            sp: 0,
            memory: [0; Self::MEMORY_SIZE],
            stack: [0; Self::STACK_SIZE],
            video: [0; Self::VIDEO_SIZE],
            keys: [false; 16],
            v: [0; 16],
            i: Self::PROGRAM_START,
            dt: 0,
            st: 0,
            error: None,
        }
    }
}
#[allow(dead_code)]
impl Chip8State {
    pub const PROGRAM_START: usize = 512;

    const MEMORY_SIZE: usize = 4096;
    const VIDEO_SIZE: usize = 256;
    const MAX_PROGRAM_SIZE: usize = 3584;
    const STACK_SIZE: usize = 16;
    const N_REGISTERS: usize = 16;
    const N_KEYS: usize = 16;
    const PITCH: usize = 8;

    pub fn new() -> Chip8State {
        Self::default()
    }

    #[inline(always)]
    pub fn video(&self) -> &[u8] {
        &self.video
    }

    #[inline(always)]
    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    #[inline(always)]
    pub fn registers(&self) -> &[u8] {
        &self.v
    }

    #[inline(always)]
    pub fn stack(&self) -> &[u16] {
        &self.stack
    }

    #[inline(always)]
    pub fn i(&self) -> usize {
        self.i
    }

    #[inline(always)]
    pub fn sp(&self) -> usize {
        self.sp
    }

    #[inline(always)]
    pub fn dt(&self) -> u8 {
        self.dt
    }

    #[inline(always)]
    pub fn st(&self) -> u8 {
        self.st
    }

    #[inline(always)]
    pub fn pc(&self) -> usize {
        self.pc
    }

    #[inline(always)]
    pub fn error(&self) -> Option<Chip8Error> {
        None
    }

    pub fn fetch(&self, address: u16) -> u16 {
        (self.memory[address as usize] as u16) << 8 | (self.memory[address as usize + 1] as u16)
    }
}

pub struct Chip8 {
    state: Chip8State,
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            state: Chip8State {
                pc: Self::PROGRAM_START,
                sp: 0,
                memory: [0; Self::MEMORY_SIZE],
                stack: [0; Self::STACK_SIZE],
                video: [0; Self::VIDEO_SIZE],
                keys: [false; 16],
                v: [0; 16],
                i: Self::PROGRAM_START,
                dt: 0,
                st: 0,
                error: None,
            },
        }
    }
}

impl Chip8 {
    pub const PROGRAM_START: usize = 512;
    const MEMORY_SIZE: usize = 4096;
    const VIDEO_SIZE: usize = 256;
    const MAX_PROGRAM_SIZE: usize = 3584;
    const STACK_SIZE: usize = 16;
    const N_REGISTERS: usize = 16;
    const N_KEYS: usize = 16;

    pub fn new() -> Chip8 {
        Chip8 {
            state: Chip8State {
                pc: Self::PROGRAM_START,
                sp: 0,
                memory: [0; Self::MEMORY_SIZE],
                stack: [0; Self::STACK_SIZE],
                video: [0; Self::VIDEO_SIZE],
                keys: [false; Self::N_KEYS],
                v: [0; Self::N_REGISTERS],
                i: Self::PROGRAM_START,
                dt: 0,
                st: 0,
                error: None,
            },
        }
    }

    #[inline(always)]
    pub fn state(&self) -> &Chip8State {
        &self.state
    }

    pub fn disassemble(instruction: u16) -> DecodedInstruction {
        let addr = instruction & 0xFFF;
        let byte = (instruction & 0xFF) as u8;
        let nibble = (instruction & 0xF) as u8;
        let x = (instruction >> 8 & 0xF) as u8;
        let y = (instruction >> 4 & 0xF) as u8;

        let (op, params) = match instruction {
            0x00E0 => ("CLS", String::new()),
            0x00EE => ("RET", String::new()),
            i if i & 0xF000 == 0x1000 => ("JUMP", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x2000 => ("CALL", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x1000 => ("JUMP", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x2000 => ("CALL", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x3000 => ("SE", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF000 == 0x4000 => ("SNE", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF000 == 0x5000 => ("SE", format!("V{:X}, V{}", x, y)),
            i if i & 0xF000 == 0x6000 => ("LOAD", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF000 == 0x7000 => ("ADD", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF00F == 0x8000 => ("LOAD", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8001 => ("OR", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8002 => ("AND", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8003 => ("XOR", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8004 => ("ADD", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8005 => ("SUB", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8006 => ("SHR", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8007 => ("SUBN", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x800E => ("SHL", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x9000 => ("SNE", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF000 == 0xA000 => ("LOAD", format!("I, #{:04X}", addr)),
            i if i & 0xF000 == 0xB000 => ("JUMP", format!("V0, #{:04X}", addr)),
            i if i & 0xF000 == 0xC000 => ("RND", format!("V{:X}, #{:02X}", x, byte)),
            i if i & 0xF000 == 0xD000 => ("DRAW", format!("V{:X}, V{:X}, {}", x, y, nibble)),
            i if i & 0xF0FF == 0xE09E => ("SKP", format!("V{:X}", x)),
            i if i & 0xF0FF == 0xE0A1 => ("SKNP", format!("V{:X}", x)),
            i if i & 0xF0FF == 0xF007 => ("LOAD", format!("V{:X}, DT", x)),
            i if i & 0xF0FF == 0xF00A => ("LOAD", format!("V{:X}, K", x)),
            i if i & 0xF0FF == 0xF015 => ("LOAD", format!("DT, V{:X}", x)),
            i if i & 0xF0FF == 0xF018 => ("LOAD", format!("ST, V{:X}", x)),
            i if i & 0xF0FF == 0xF01E => ("ADD", format!("I, V{:X}", x)),
            i if i & 0xF0FF == 0xF029 => ("FONT", format!("I, V{:X}", x)),
            i if i & 0xF0FF == 0xF033 => ("BCD", format!("I, V{:X}", x)),
            i if i & 0xF0FF == 0xF055 => ("SAV", format!("[I], V{:X}", x)),
            i if i & 0xF0FF == 0xF065 => ("RST", format!("V{:X}, [I]", x)),
            _ => ("???", String::new()),
        };

        DecodedInstruction {
            operation: String::from(op),
            params,
        }
    }

    pub fn soft_reset(&mut self) {
        let state = &mut self.state;

        state.pc = Self::PROGRAM_START;
        state.sp = 0;
        state.i = Self::PROGRAM_START;
        state.dt = 0;
        state.st = 0;
        state.error = None;
        state.stack = [0; Self::STACK_SIZE];
        state.video = [0; Self::VIDEO_SIZE];
        state.keys = [false; 16];
        state.v = [0; 16];
    }

    pub fn hard_reset(&mut self) {
        self.soft_reset();
        self.state.memory = [0; Self::MEMORY_SIZE];
        self.state.memory[..rom::ROM.len()].clone_from_slice(&rom::ROM);
    }

    pub fn press_key(&mut self, key: usize) {
        self.state.keys[key] = true;
    }

    pub fn release_key(&mut self, key: usize) {
        self.state.keys[key] = false;
    }

    pub fn load_bytes(&mut self, bytes: &[u8]) -> Result<usize, Chip8Error> {
        let cpu = &mut self.state;
        let n_bytes = bytes.len();

        if n_bytes > Self::MAX_PROGRAM_SIZE {
            return Err(Chip8Error::ProgramLoadError);
        }

        let i = Self::PROGRAM_START;
        cpu.memory[i..i + n_bytes].clone_from_slice(&bytes);

        Ok(n_bytes)
    }

    pub fn execute_cycle(&mut self) -> Result<(), Chip8Error> {
        //let pc = self.state.pc;
        //self.state.v[0xF] = 0;

        if self.state.dt > 0 {
            self.state.dt -= 1;
        }

        if self.state.st > 0 {
            self.state.st -= 1;
        }

        let instruction: u16 = self.state.fetch(self.state.pc as u16);
        self.state.pc += 2;

        self.execute(instruction)
    }

    fn execute(&mut self, instruction: u16) -> Result<(), Chip8Error> {
        let addr: usize = (instruction & 0x0FFF) as usize;
        let byte: u8 = (instruction & 0x00FF) as u8;
        let nibble = (instruction & 0x000F) as u8;
        let x: usize = (instruction >> 8 & 0xF) as usize;
        let y: usize = (instruction >> 4 & 0xF) as usize;

        match instruction {
            0x00E0 => self.cls(),
            0x00EE => self.ret(),
            i if i & 0xF000 == 0x1000 => self.jp(addr),
            i if i & 0xF000 == 0x2000 => self.call(addr),
            i if i & 0xF000 == 0x3000 => self.se_byte(x, byte),
            i if i & 0xF000 == 0x4000 => self.sne_byte(x, byte),
            i if i & 0xF000 == 0x5000 => self.se_reg(x, y),
            i if i & 0xF000 == 0x6000 => self.ld_byte(x, byte),
            i if i & 0xF000 == 0x7000 => self.add_byte(x, byte),
            i if i & 0xF00F == 0x8000 => self.ld_reg(x, y),
            i if i & 0xF00F == 0x8001 => self.or(x, y),
            i if i & 0xF00F == 0x8002 => self.and(x, y),
            i if i & 0xF00F == 0x8003 => self.xor(x, y),
            i if i & 0xF00F == 0x8004 => self.add_reg(x, y),
            i if i & 0xF00F == 0x8005 => self.sub(x, y),
            i if i & 0xF00F == 0x8006 => self.shr(x),
            i if i & 0xF00F == 0x8007 => self.subn(x, y),
            i if i & 0xF00F == 0x800E => self.shl(x),
            i if i & 0xF00F == 0x9000 => self.sne_reg(x, y),
            i if i & 0xF000 == 0xA000 => self.load_i(addr),
            i if i & 0xF000 == 0xB000 => self.jp_v0(addr),
            i if i & 0xF000 == 0xC000 => self.rnd(x, byte),
            i if i & 0xF000 == 0xD000 => self.drw(x, y, nibble),
            i if i & 0xF0FF == 0xE09E => self.skp(x),
            i if i & 0xF0FF == 0xE0A1 => self.sknp(x),
            i if i & 0xF0FF == 0xF007 => self.ld_v_dt(x),
            i if i & 0xF0FF == 0xF00A => self.ld_key(x),
            i if i & 0xF0FF == 0xF015 => self.ld_dt_v(x),
            i if i & 0xF0FF == 0xF018 => self.ld_st_v(x),
            i if i & 0xF0FF == 0xF01E => self.add_i(x),
            i if i & 0xF0FF == 0xF029 => self.fnt(x),
            i if i & 0xF0FF == 0xF033 => self.bcd(x),
            i if i & 0xF0FF == 0xF055 => self.save(x),
            i if i & 0xF0FF == 0xF065 => self.restore(x),
            _ => return Err(Chip8Error::UnknownInstructionError),
        };

        Ok(())
    }

    fn cls(&mut self) {
        for i in 0..self.state.video.len() {
            self.state.video[i] = 0;
        }
    }

    fn ret(&mut self) {
        self.state.error = match self.state.sp {
            p if p == 0 => Some(Chip8Error::StackUnderflowError),
            _ => {
                self.state.sp -= 1;
                self.state.pc = self.state.stack[self.state.sp] as usize;
                None
            }
        }
    }

    fn jp(&mut self, address: usize) {
        self.state.pc = address;
    }

    fn jp_v0(&mut self, address: usize) {
        self.state.pc = address + (self.state.v[0] as usize);
    }

    fn call(&mut self, address: usize) {
        match address {
            a if a >= Self::MAX_PROGRAM_SIZE => {
                self.state.error = Some(Chip8Error::AddressOutOfRangeError);
                return;
            }
            _ => (),
        }
        self.state.stack[self.state.sp] = self.state.pc as u16;
        self.state.sp += 1;
        self.state.pc = address as usize;

        self.state.error = match self.state.sp {
            sp if sp >= Self::STACK_SIZE => Some(Chip8Error::StackOverflowError),
            _ => None,
        };
    }

    fn se_byte(&mut self, x: usize, byte: u8) {
        if self.state.v[x] == byte {
            self.state.pc += 2;
        }
    }

    fn se_reg(&mut self, x: usize, y: usize) {
        if self.state.v[x] == self.state.v[y] {
            self.state.pc += 2;
        }
    }

    fn sne_byte(&mut self, x: usize, byte: u8) {
        if self.state.v[x] != byte {
            self.state.pc += 2;
        }
    }

    fn sne_reg(&mut self, x: usize, y: usize) {
        if self.state.v[x] != self.state.v[y] {
            self.state.pc += 2;
        }
    }

    fn ld_byte(&mut self, x: usize, byte: u8) {
        self.state.v[x] = byte;
    }

    fn ld_reg(&mut self, x: usize, y: usize) {
        self.state.v[x] = self.state.v[y];
    }

    fn load_i(&mut self, addr: usize) {
        self.state.i = addr;
    }

    fn add_byte(&mut self, x: usize, byte: u8) {
        self.state.v[x] = self.state.v[x].wrapping_add(byte);
    }

    fn add_reg(&mut self, x: usize, y: usize) {
        let sum = u16::from(self.state.v[x]) + u16::from(self.state.v[y]);
        self.state.v[0xF] = if sum > 0xFF { 1 } else { 0 };
        self.state.v[x] = sum as u8;
    }

    fn or(&mut self, x: usize, y: usize) {
        self.state.v[x] |= self.state.v[y];
    }

    fn and(&mut self, x: usize, y: usize) {
        self.state.v[x] &= self.state.v[y];
    }

    fn xor(&mut self, x: usize, y: usize) {
        self.state.v[x] ^= self.state.v[y];
    }

    fn sub(&mut self, x: usize, y: usize) {
        if self.state.v[x] > self.state.v[y] {
            self.state.v[0xF] = 1;
        } else {
            self.state.v[0xF] = 0;
        }

        self.state.v[x] = ((self.state.v[x] as i32) - (self.state.v[y] as i32)) as u8;
    }

    fn shr(&mut self, x: usize) {
        let v = self.state.v[x];
        self.state.v[0xF] = v & 0x1;
        self.state.v[x] = v >> 1;
    }

    fn shl(&mut self, x: usize) {
        let v = self.state.v[x];
        self.state.v[0xF] = v >> 7;
        self.state.v[x] = v << 1;
    }

    fn subn(&mut self, x: usize, y: usize) {
        if self.state.v[y] > self.state.v[x] {
            self.state.v[0xF] = 1;
        } else {
            self.state.v[0xF] = 0;
        }
        self.state.v[x] = self.state.v[y].wrapping_sub(self.state.v[x]);
    }

    fn rnd(&mut self, x: usize, byte: u8) {
        let r = rand::random::<u8>();
        self.state.v[x] = r & byte;
    }

    fn ld_dt_v(&mut self, x: usize) {
        self.state.dt = self.state.v[x];
    }

    fn ld_v_dt(&mut self, x: usize) {
        self.state.v[x] = self.state.dt;
    }

    fn ld_key(&mut self, x: usize) {
        for i in 0..self.state.keys.len() {
            if self.state.keys[i] {
                self.state.v[x] = i as u8;
                return;
            }
        }

        self.state.pc -= 2;
    }

    fn ld_st_v(&mut self, x: usize) {
        self.state.st = self.state.v[x];
    }

    fn skp(&mut self, x: usize) {
        if self.state.keys[x] {
            self.state.pc += 2;
        }
    }

    fn sknp(&mut self, x: usize) {
        if !self.state.keys[x] {
            self.state.pc += 2;
        }
    }

    fn drw(&mut self, vx: usize, vy: usize, n: u8) {
        let mut carry: u8 = 0;
        let x = self.state.v[vx];
        let y = self.state.v[vy];

        for i in 0..n {
            let x_offset = x >> 3;
            let x_bit = x & 7;
            let y_offset = ((y + i) as usize) * Chip8State::PITCH;
            let mem_addr = i as usize + self.state.i;
            let mem_byte = self.state.memory[mem_addr as usize];

            let video_addr = y_offset + (x_offset as usize);

            if video_addr >= 255 {
                // TODO fix
                break;
            }

            let byte_0 = self.state.video[video_addr];
            let byte_1 = self.state.video[video_addr + 1];
            self.state.video[video_addr] ^= mem_byte >> x_bit;

            if x_bit > 0 {
                self.state.video[video_addr + 1] ^= mem_byte << (8 - x_bit);
            }

            carry |= byte_0 & !self.state.video[video_addr];
            carry |= byte_1 & !self.state.video[video_addr + 1];
        }

        self.state.v[0xF] = match carry {
            0 => 0 as u8,
            _ => 1 as u8,
        };
    }

    fn add_i(&mut self, x: usize) {
        self.state.i += self.state.v[x] as usize;
    }

    fn fnt(&mut self, x: usize) {
        let addr = 0 + self.state.v[x] * 5;
        self.state.i = addr as usize;
    }

    fn bcd(&mut self, x: usize) {
        let value = self.state.v[x];
        let addr = self.state.i;

        self.state.memory[addr] = ((value as u16 % 1000) / 100) as u8;
        self.state.memory[addr + 1] = (value % 100) / 10;
        self.state.memory[addr + 2] = value % 10;
    }

    fn save(&mut self, x: usize) {
        let addr = self.state.i;

        for i in 0..=x {
            self.state.memory[addr + i] = self.state.v[i];
        }
    }

    fn restore(&mut self, x: usize) {
        for i in 0..=x {
            self.state.v[i] = self.state.memory[self.state.i + i];
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_i() {
        let mut cpu = Chip8::new();
        cpu.state.load_i(800);
        assert_eq!(cpu.state.i, 800);
    }

    #[test]
    fn test_add_byte() {
        let mut cpu = Chip8::new();

        cpu.state.v[0] = 5;
        cpu.state.add_byte(0, 5);
        assert_eq!(cpu.state.v[0], 10);

        cpu.state.v[0] = 255;
        cpu.state.add_byte(0, 2);
        assert_eq!(cpu.state.v[0], 1);
    }

    #[test]
    fn test_add_reg() {
        let mut cpu = Chip8::new();

        cpu.state.v[0] = 5;
        cpu.state.v[1] = 5;
        cpu.state.add_reg(0, 1);
        assert_eq!(cpu.state.v[0], 10);
        assert_eq!(cpu.state.v[0xF], 0);

        cpu.state.v[0] = 255;
        cpu.state.v[1] = 2;
        cpu.state.add_reg(0, 1);
        assert_eq!(cpu.state.v[0], 1);
        assert_eq!(cpu.state.v[0xF], 1);
    }

    #[test]
    fn test_key_press_release() {
        let mut cpu = Chip8::new();
        assert_eq!(cpu.state().keys[1], false);
        cpu.press_key(1);
        assert_eq!(cpu.state().keys[1], true);;
        cpu.release_key(1);
        assert_eq!(cpu.state().keys[1], false);;
    }
}
