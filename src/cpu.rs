extern crate rand;

use std::error::Error;
use std::fmt;
use std::string::String;

const PROGRAM_START: usize = 0x200;
pub const MAX_PROGRAM_SIZE: usize = 0xE00;
const VIDEO_SIZE: usize = 256;
const PITCH: usize = 8;

#[derive(Clone, Copy, Debug)]
pub enum Chip8Error {
    UnknownInstructionError,
    /*
    AddressOutOfRangeError,
    InstructionOffsetError,
    ProgramLoadError,
    NotImplementedError,
    StackOverflowError,
    StackUnderflowError,
    */
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
            /*
            Chip8Error::AddressOutOfRangeError => "memory address out of range",
            Chip8Error::InstructionOffsetError => "instruction byte not aligned",
            Chip8Error::ProgramLoadError => "error loading program rom",
            Chip8Error::NotImplementedError => "instruction or function not implemented",
            Chip8Error::StackOverflowError => "stack overflow",
            Chip8Error::StackUnderflowError => "stack underflow",
            */
        }
    }
}

pub struct Chip8 {
    pub video: [u8; VIDEO_SIZE],
    pub memory: [u8; MAX_PROGRAM_SIZE],
    pub v: [u8; 16],
    pub stack: [u16; 16],
    pub pc: usize,
    pub sp: usize,
    pub i: u16,
    pub dt: u8,
    pub st: u8,
    pub pitch: u32,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            pc: PROGRAM_START,
            sp: 0,
            memory: [0; MAX_PROGRAM_SIZE],
            stack: [0; 16],
            video: [0; VIDEO_SIZE],
            v: [0; 16],
            i: 512,
            dt: 0,
            st: 0,
            pitch: 8,
        }
    }

    fn cls(&mut self) {
        for i in 0..self.video.len() {
            self.video[i] = 0;
        }
    }

    fn ret(&mut self) {
        // TODO underflow
        self.sp -= 1;
        self.pc = self.stack[self.sp] as usize;
    }

    fn jp(&mut self, address: u16) {
        self.pc = address as usize;
    }

    fn jp_v0(&mut self, address: u16) {
        self.pc = address as usize + (self.v[0] as usize);
    }

    fn call(&mut self, address: u16) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = address as usize;
    }

    fn se_byte(&mut self, x: usize, byte: u8) {
        if self.v[x] == byte {
            self.pc += 2;
        }
    }

    fn se_reg(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    fn sne_byte(&mut self, x: usize, byte: u8) {
        if self.v[x] != byte {
            self.pc += 2;
        }
    }

    fn sne_reg(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    fn ld_byte(&mut self, x: usize, byte: u8) {
        self.v[x] = byte;
    }

    fn ld_reg(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn ld_i(&mut self, addr: u16) {
        self.i = addr;
    }

    fn add_byte(&mut self, x: usize, byte: u8) {
        // TODO verify
        let value = self.v[x] as u16;
        self.v[x] = (value + byte as u16) as u8;
    }

    fn add_reg(&mut self, x: usize, y: usize) {
        let sum = (self.v[x] as u16) + (self.v[y] as u16);
        self.v[x] = sum as u8;

        if sum > 0xFF {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
    }

    fn or(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] | self.v[y];
    }

    fn and(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] & self.v[y];
    }

    fn xor(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[x] ^ self.v[y];
    }

    fn sub(&mut self, x: usize, y: usize) {
        if self.v[x] > self.v[y] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x] = ((self.v[x] as i32) - (self.v[y] as i32)) as u8;
    }

    fn shr(&mut self, _x: usize, _y: usize) {
        // TODO implement
    }

    fn shl(&mut self, _x: usize, _y: usize) {
        // TODO implement
    }

    fn subn(&mut self, _x: usize, _y: usize) {
        // TODO implement
    }

    fn rnd(&mut self, x: usize, byte: u8) {
        let r = rand::random::<u8>();
        self.v[x] = r & byte;
    }

    fn ld_dt_v(&mut self, x: usize) {
        self.dt = self.v[x];
    }

    fn ld_v_dt(&mut self, x: usize) {
        self.v[x] = self.dt;
    }

    fn ld_key(&mut self, _x: usize) {
        // TODO implement
    }

    fn ld_st_v(&mut self, x: usize) {
        self.st = self.v[x];
    }

    fn skp(&mut self, _x: usize) {
        // TODO implement
    }

    fn sknp(&mut self, _x: usize) {
        // TODO implement
    }

    fn drw(&mut self, vx: usize, vy: usize, n: u8) {
        let mut carry: u8 = 0;
        let x = self.v[vx];
        let y = self.v[vy];

        for i in 0..n {
            let x_offset = x >> 3;
            let x_bit = x & 7;
            let y_offset = ((y + i) as usize) * PITCH;
            let mem_addr = self.i + (i as u16);
            let mem_byte = self.memory[mem_addr as usize];

            let video_addr = y_offset + (x_offset as usize);

            if video_addr >= 255 {
                // TODO fix
                break;
            }

            let byte_0 = self.video[video_addr];
            let byte_1 = self.video[video_addr + 1];
            self.video[video_addr] ^= mem_byte >> x_bit;

            if x_bit > 0 {
                self.video[video_addr + 1] ^= mem_byte << (8 - x_bit);
            }

            carry |= byte_0 & !self.video[video_addr];
            carry |= byte_1 & !self.video[video_addr + 1];
        }

        self.v[0xF] = match carry {
            0 => 0 as u8,
            _ => 1 as u8,
        };
    }

    fn add_i(&mut self, x: usize) {
        self.i += self.v[x] as u16;
    }

    fn fnt(&mut self, _x: usize) {
        // TODO implement
    }

    fn bcd(&mut self, x: usize) {
        let value = self.v[x];
        let addr = self.i as usize;

        self.memory[addr] = ((value as u16 % 1000) / 100) as u8;
        self.memory[addr + 1] = (value % 100) / 10;
        self.memory[addr + 2] = value % 10;
    }

    fn save(&mut self, x: usize) {
        let addr = self.i as usize;

        for i in 0..x {
            self.memory[addr + i] = self.v[i];
        }
    }

    fn restore(&mut self, x: usize) {
        let addr = self.i as usize;

        for i in 0..x {
            self.v[i] = self.memory[addr + i];
        }
    }

    pub fn step(&mut self) -> Result<(), Chip8Error> {
        let pc = self.pc;

        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }

        let instruction: u16 = self.fetch(pc);
        self.pc += 2;

        let result = self._execute(instruction);

        result
    }

    fn _execute(&mut self, instruction: u16) -> Result<(), Chip8Error> {
        let addr = instruction & 0x0FFF;
        let byte: u8 = (instruction & 0x00FF) as u8;
        let nibble = (instruction & 0x000F) as u8;
        let x: usize = (instruction >> 8 & 0xF) as usize;
        let y: usize = (instruction >> 4 & 0xF) as usize;

        match instruction {
            0x00E0 => Ok(self.cls()),
            0x00EE => Ok(self.ret()),
            i if i & 0xF000 == 0x1000 => Ok(self.jp(addr)),
            i if i & 0xF000 == 0x2000 => Ok(self.call(addr)),
            i if i & 0xF000 == 0x3000 => Ok(self.se_byte(x, byte)),
            i if i & 0xF000 == 0x4000 => Ok(self.sne_byte(x, byte)),
            i if i & 0xF000 == 0x5000 => Ok(self.se_reg(x, y)),
            i if i & 0xF000 == 0x6000 => Ok(self.ld_byte(x, byte)),
            i if i & 0xF000 == 0x7000 => Ok(self.add_byte(x, byte)),
            i if i & 0xF00F == 0x8000 => Ok(self.ld_reg(x, y)),
            i if i & 0xF00F == 0x8001 => Ok(self.or(x, y)),
            i if i & 0xF00F == 0x8002 => Ok(self.and(x, y)),
            i if i & 0xF00F == 0x8003 => Ok(self.xor(x, y)),
            i if i & 0xF00F == 0x8004 => Ok(self.add_reg(x, y)),
            i if i & 0xF00F == 0x8005 => Ok(self.sub(x, y)),
            i if i & 0xF00F == 0x8006 => Ok(self.shr(x, y)),
            i if i & 0xF00F == 0x8007 => Ok(self.subn(x, y)),
            i if i & 0xF00F == 0x800E => Ok(self.shl(x, y)),
            i if i & 0xF00F == 0x9000 => Ok(self.sne_reg(x, y)),
            i if i & 0xF000 == 0xA000 => Ok(self.ld_i(addr)),
            i if i & 0xF000 == 0xB000 => Ok(self.jp_v0(addr)),
            i if i & 0xF000 == 0xC000 => Ok(self.rnd(x, byte)),
            i if i & 0xF000 == 0xD000 => Ok(self.drw(x, y, nibble)),
            i if i & 0xF0FF == 0xE09E => Ok(self.skp(x)),
            i if i & 0xF0FF == 0xE0A1 => Ok(self.sknp(x)),
            i if i & 0xF0FF == 0xF007 => Ok(self.ld_v_dt(x)),
            i if i & 0xF0FF == 0xF00A => Ok(self.ld_key(x)),
            i if i & 0xF0FF == 0xF015 => Ok(self.ld_dt_v(x)),
            i if i & 0xF0FF == 0xF018 => Ok(self.ld_st_v(x)),
            i if i & 0xF0FF == 0xF01E => Ok(self.add_i(x)),
            i if i & 0xF0FF == 0xF029 => Ok(self.fnt(x)),
            i if i & 0xF0FF == 0xF033 => Ok(self.bcd(x)),
            i if i & 0xF0FF == 0xF055 => Ok(self.save(x)),
            i if i & 0xF0FF == 0xF065 => Ok(self.restore(x)),
            _ => Err(Chip8Error::UnknownInstructionError),
        }
    }

    pub fn fetch(&self, address: usize) -> u16 {
        // TODO out of range
        let result = (self.memory[address] as u16) << 8 | (self.memory[address + 1] as u16);
        result
    }

    pub fn execute(&mut self, instructions: &[u16]) -> Result<(), Chip8Error> {
        for instruction in instructions.iter() {
            if let Err(err) = self._execute(*instruction) {
                return Err(err);
            }
        }

        Ok(())
    }

    pub fn disassemble2(address: usize, memory: &[u8; MAX_PROGRAM_SIZE]) -> DecodedInstruction {
        let instruction = (memory[address] as u16) << 8 | (memory[address + 1] as u16);
        let addr = instruction & 0xFFF;
        let byte: u8 = (instruction & 0xFF) as u8;
        //let nibble = (instruction & 0xF) as u8;
        let x: u8 = (instruction >> 8 & 0xF) as u8;
        let y: u8 = (instruction >> 4 & 0xF) as u8;

        let (op, params) = match instruction {
            0x00E0 => ("CLS", String::new()),
            0x00EE => ("RET", String::new()),
            i if i & 0xF000 == 0x1000 => ("JP", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x2000 => ("CALL", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x1000 => ("JP", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x2000 => ("CALL", format!("#{:04X}", addr)),
            i if i & 0xF000 == 0x3000 => ("SE", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF000 == 0x4000 => ("SNE", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF000 == 0x5000 => ("SE", format!("V{:X}, V{}", x, y)),
            i if i & 0xF000 == 0x6000 => ("LD", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF000 == 0x7000 => ("ADD", format!("V{:X}, {:02X}", x, byte)),
            i if i & 0xF00F == 0x8000 => ("LD", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8001 => ("OR", format!("V{:X}, V{:X}", x, y)),
            i if i & 0xF00F == 0x8002 => ("AND", format!("V{:X}, V{:X}", x, y)),
            /*
            i if i & 0xF00F == 0x8003 => format!("XOR    V{:X}, V{:X}", x, y),
            i if i & 0xF00F == 0x8004 => format!("ADD    V{:X}, V{:X}", x, y),
            i if i & 0xF00F == 0x8005 => format!("SUB    V{:X}, V{:X}", x, y),
            i if i & 0xF00F == 0x8006 => format!("SHR    V{:X}, V{:X}", x, y),
            i if i & 0xF00F == 0x8007 => format!("SUBN   V{:X}, V{:X}", x, y),
            i if i & 0xF00F == 0x800E => format!("SHL    V{:X}, V{:X}", x, y),
            i if i & 0xF00F == 0x9000 => format!("SNE    V{:X}, V{:X}", x, y),
            i if i & 0xF000 == 0xA000 => format!("LD     I, #{:04X}", addr),
            i if i & 0xF000 == 0xB000 => format!("JP     V0, #{:04X}", addr),
            i if i & 0xF000 == 0xC000 => format!("RND    V{:X}, #{:02X}", x, byte),
            i if i & 0xF000 == 0xD000 => format!("DRW    V{:X}, V{:X}, {}", x, y, nibble),
            i if i & 0xF0FF == 0xE09E => format!("SKP    V{:X}", x),
            i if i & 0xF0FF == 0xE0A1 => format!("SKNP   V{:X}", x),
            i if i & 0xF0FF == 0xF007 => format!("LD     V{:X}, DT", x),
            i if i & 0xF0FF == 0xF00A => format!("LD     V{:X}, K", x),
            i if i & 0xF0FF == 0xF015 => format!("LD     DT, V{:X}", x),
            i if i & 0xF0FF == 0xF018 => format!("LD     ST, V{:X}", x),
            i if i & 0xF0FF == 0xF01E => format!("ADD    I, V{:X}", x),
            i if i & 0xF0FF == 0xF029 => format!("FNT    I, V{:X}", x),
            i if i & 0xF0FF == 0xF033 => format!("BCD    I, V{:X}", x),
            i if i & 0xF0FF == 0xF055 => format!("LD     [I], V0-V{:X}", x),
            i if i & 0xF0FF == 0xF065 => format!("LD     V0-V{:X}, [I]", x),
            */
            _ => ("???", String::new()),
        };

        DecodedInstruction {
            address: address,
            instruction: instruction,
            operation: String::from(op),
            params: params,
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) -> Result<usize, &Chip8Error> {
        /*
        let mut buffer = [0; MAX_PROGRAM_SIZE];
        let bytes_read: usize;

        match reader.read(&mut buffer) {
            Err(_) => return Err(&Chip8Error::ProgramLoadError), // TODO cause
            Ok(bytes) => bytes_read = bytes,
        }

        self.memory = [0; MAX_PROGRAM_SIZE];
        */

        for i in 512..self.memory.len() {
            // TODO
            if i - 512 < bytes.len() {
                self.memory[i] = bytes[i - 512];
            }
        }

        // TODO too big

        return Ok(bytes.len());
    }
}

pub struct DecodedInstruction {
    pub address: usize,
    pub instruction: u16,
    pub operation: String,
    pub params: String,
}
