extern crate rand;

use std::error::Error;
use std::fmt;

const MEMORY_SIZE: usize = 0x1000;
const PROGRAM_START: usize = 0x200;
const MAX_PROGRAM_SIZE: usize = 0xE00;
const VIDEO_SIZE: usize = 256;

#[derive(Debug)]
pub enum Chip8Error {
    AddressOutOfRangeError,
    InstructionOffsetError,
    ProgramLoadError,
    NotImplementedError,
    UnknownInstructionError,
    StackOverflowError,
    StackUnderflowError,
}

impl fmt::Display for Chip8Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for Chip8Error {
    fn description(&self) -> &str {
        match *self {
            Chip8Error::AddressOutOfRangeError => "memory address out of range",
            Chip8Error::InstructionOffsetError => "instruction byte not aligned",
            Chip8Error::UnknownInstructionError => "instruction unknown",
            Chip8Error::ProgramLoadError => "error loading program rom",
            Chip8Error::NotImplementedError => "instruction or function not implemented",
            Chip8Error::StackOverflowError => "stack overflow",
            Chip8Error::StackUnderflowError => "stack underflow",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Chip8State {
    pub video: [u8; VIDEO_SIZE],
    pub memory: [u8; MAX_PROGRAM_SIZE],
    pub v: [u8; 16],
    pub stack: [u16; 16],
    pub dt: u8,
    pub st: u8,
    pub pc: usize,
    pub i: usize,
    pub sp: usize,
}

pub struct Chip8 {
    pub pc: usize,
    pub sp: usize,
    pub memory: [u8; MAX_PROGRAM_SIZE],
    pub v: [u8; 16],
    pub dt: u8,
    pub st: u8,
    pub i: u16,
    pub video: [u8; 256],
    stack: [u16; 16],
    pub step: u64,
    pub pitch: u32,
    pub speed: u32,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            pc: PROGRAM_START,
            sp: 0,
            memory: [0; MAX_PROGRAM_SIZE],
            stack: [0; 16],
            video: [0; 256],
            v: [0; 16],
            i: 512,
            dt: 0,
            st: 0,
            step: 0,
            pitch: 8,
            speed: 1000,
        }
    }

    fn cls(&mut self) -> Result<(), Chip8Error> {
        for i in 0..self.video.len() {
            self.video[i] = 0;
        }
        Ok(())
    }

    fn ret(&mut self) -> Result<(), Chip8Error> {
        match self.sp {
            i if i < 1 => Err(Chip8Error::StackUnderflowError),
            _ => {
                self.sp -= 1;
                self.pc = self.stack[self.sp] as usize;
                Ok(())
            }
        }
    }

    fn jp(&mut self, address: u16) -> Result<(), Chip8Error> {
        self.pc = address as usize;
        Ok(())
    }

    fn jp_v0(&mut self, address: u16) -> Result<(), Chip8Error> {
        self.pc = address as usize + (self.v[0] as usize);
        Ok(())
    }

    fn call(&mut self, address: u16) -> Result<(), Chip8Error> {

        match self.sp {
            i if i > 14 => Err(Chip8Error::StackOverflowError),
            _ => {
                self.stack[self.sp] = self.pc as u16;
                self.sp += 1;
                self.pc = address as usize;
                Ok(())
            }
        }
    }

    fn se_byte(&mut self, x: usize, byte: u8) -> Result<(), Chip8Error> {
        if self.v[x] == byte {
            self.pc += 2;
        }

        Ok(())
    }

    fn se_reg(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }

        Ok(())
    }

    fn sne_byte(&mut self, x: usize, byte: u8) -> Result<(), Chip8Error> {
        if self.v[x] != byte {
            self.pc += 2;
        }

        Ok(())
    }

    fn sne_reg(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }

        Ok(())
    }

    fn ld_byte(&mut self, x: usize, byte: u8) -> Result<(), Chip8Error> {
        self.v[x] = byte;
        Ok(())
    }

    fn ld_reg(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        self.v[x] = self.v[y];
        Ok(())
    }

    fn ld_i(&mut self, addr: u16) -> Result<(), Chip8Error> {
        self.i = addr;
        Ok(())
    }

    fn add_byte(&mut self, x: usize, byte: u8) -> Result<(), Chip8Error> {
        // TODO verify
        let value = self.v[x] as u16;
        self.v[x] = (value + byte as u16) as u8;
        Ok(())
    }

    fn add_reg(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        let sum = (self.v[x] as u16) + (self.v[y] as u16);
        self.v[x] = sum as u8;

        if sum > 0xFF {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        Ok(())
    }

    fn or(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        self.v[x] = self.v[x] | self.v[y];
        Ok(())
    }

    fn and(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        self.v[x] = self.v[x] & self.v[y];
        Ok(())
    }

    fn xor(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {
        self.v[x] = self.v[x] ^ self.v[y];
        Ok(())
    }

    fn sub(&mut self, x: usize, y: usize) -> Result<(), Chip8Error> {

        if self.v[x] > self.v[y] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x] = ((self.v[x] as i32) - (self.v[y] as i32)) as u8;

        Ok(())
    }

    fn shr(&mut self, _x: usize, _y: usize) -> Result<(), Chip8Error> {
        Err(Chip8Error::NotImplementedError)
    }

    fn shl(&mut self, _x: usize, _y: usize) -> Result<(), Chip8Error> {
        Err(Chip8Error::NotImplementedError)
    }

    fn subn(&mut self, _x: usize, _y: usize) -> Result<(), Chip8Error> {
        Err(Chip8Error::NotImplementedError)
    }

    fn rnd(&mut self, x: usize, byte: u8) -> Result<(), Chip8Error> {

        let r = rand::random::<u8>();
        self.v[x] = r & byte;
        Ok(())
    }

    fn ld_dt_v(&mut self, x: usize) -> Result<(), Chip8Error> {
        self.dt = self.v[x];
        Ok(())
    }

    fn ld_v_dt(&mut self, x: usize) -> Result<(), Chip8Error> {
        self.v[x] = self.dt;
        Ok(())
    }

    fn ld_key(&mut self, _x: usize) -> Result<(), Chip8Error> {
        // TODO keypress
        Ok(())
    }

    fn ld_st_v(&mut self, x: usize) -> Result<(), Chip8Error> {
        self.st = self.v[x];
        Ok(())
    }

    fn skp(&mut self, _x: usize) -> Result<(), Chip8Error> {
        // TODO keypress
        Ok(())
    }

    fn sknp(&mut self, _x: usize) -> Result<(), Chip8Error> {
        // TODO keypress
        Ok(())
    }

    fn drw(&mut self, vx: usize, vy: usize, n: u8) -> Result<(), Chip8Error> {
        let mut carry: u8 = 0;
        let x = self.v[vx];
        let y = self.v[vy];

        for i in 0..n {
            let x_offset = x >> 3;
            let x_bit = x & 7;
            let y_offset = ((y+i) as usize) * 8;
            let mem_addr = self.i + (i as u16);
            let mem_byte = self.memory[mem_addr as usize];

            let video_addr = y_offset + (x_offset as usize);

            if video_addr >= 255 { // TODO fix
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

        Ok(())
    }

    fn add_i(&mut self, x: usize,) -> Result<(), Chip8Error> {
        self.i += self.v[x] as u16;
        Ok(())
    }

    fn fnt(&mut self, _x: usize,) -> Result<(), Chip8Error> {
        Err(Chip8Error::NotImplementedError)
    }

    fn bcd(&mut self, x: usize,) -> Result<(), Chip8Error> {
        let value = self.v[x];
        let addr = self.i as usize;

        self.memory[addr] = ((value as u16 % 1000) / 100) as u8;
        self.memory[addr + 1] = (value % 100) / 10;
        self.memory[addr + 2] = value % 10;

        Ok(())
    }

    fn save(&mut self, x: usize) -> Result<(), Chip8Error> {
        let addr = self.i as usize;

        for i in 0..x {
            self.memory[addr + i] = self.v[i];
        }

        Ok(())
    }

    fn restore(&mut self, x: usize) -> Result<(), Chip8Error> {
        let addr = self.i as usize;

        for i in 0..x {
            self.v[i] = self.memory[addr + i];
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Chip8Error> {
        let pc = self.pc;

        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }

        let instruction: u16 = self.fetch(pc)?;
        self.pc += 2;

        let result = self._execute(instruction);

        self.step += 1;

        result
    }

    pub fn state(&self) -> Chip8State {
        Chip8State {
            video: self.video.clone(),
            memory: self.memory.clone(),
            v: self.v.clone(),
            dt: self.dt,
            st: self.st,
            pc: self.pc,
            i: self.i as usize,
            stack: self.stack.clone(),
            sp: self.sp,
        }
    }

    pub fn execute(&mut self, instructions: &[u16]) -> Result<(), Chip8Error> {
        for instruction in instructions.iter() {
            if let Err(err) = self._execute(*instruction) {
                return Err(err);
            }
        }

        Ok(())
    }

    fn _execute(&mut self, instruction: u16) -> Result<(), Chip8Error> {

        let addr = instruction & 0x0FFF;
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
            i if i & 0xF00F == 0x8006 => self.shr(x, y),
            i if i & 0xF00F == 0x8007 => self.subn(x, y),
            i if i & 0xF00F == 0x800E => self.shl(x, y),
            i if i & 0xF00F == 0x9000 => self.sne_reg(x, y),
            i if i & 0xF000 == 0xA000 => self.ld_i(addr),
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
            _ => Err(Chip8Error::UnknownInstructionError)
        }
    }


    pub fn fetch(&mut self, address: usize) -> Result<u16, Chip8Error> {
        match address {
            i if i >= MEMORY_SIZE - 1 => Err(Chip8Error::AddressOutOfRangeError),
            i if i % 2 > 0 => Err(Chip8Error::InstructionOffsetError),
            _ => {
                let result =
                    Ok((self.memory[address] as u16) << 8 | (self.memory[address + 1] as u16));
                result
            }
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

pub fn disassemble(instruction: u16) -> Result<String, Chip8Error> {

    let addr = instruction & 0xFFF;
    let byte: u8 = (instruction & 0xFF) as u8;
    let nibble = (instruction & 0xF) as u8;
    let x: u8 = (instruction >> 8 & 0xF) as u8;
    let y: u8 = (instruction >> 4 & 0xF) as u8;

    let mut string = String::new();

    string.push_str(&match instruction {
        0x00E0 => "CLS".to_owned(),
        0x00EE => "RET".to_owned(),
        i if i & 0xF000 == 0x1000 => format!("JP     #{:04X}", addr),
        i if i & 0xF000 == 0x2000 => format!("CALL   #{:04X}", addr),
        i if i & 0xF000 == 0x3000 => format!("SE     V{:X}, {:02X}", x, byte),
        i if i & 0xF000 == 0x4000 => format!("SNE    V{:X}, {:02X}", x, byte),
        i if i & 0xF000 == 0x5000 => format!("SE     V{:X}, V{}", x, y),
        i if i & 0xF000 == 0x6000 => format!("LD     V{:X}, {:02X}", x, byte),
        i if i & 0xF000 == 0x7000 => format!("ADD    V{:X}, {:02X}", x, byte),
        i if i & 0xF00F == 0x8000 => format!("LD     V{:X}, V{:X}", x, y),
        i if i & 0xF00F == 0x8001 => format!("OR     V{:X}, V{:X}", x, y),
        i if i & 0xF00F == 0x8002 => format!("AND    V{:X}, V{:X}", x, y),
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
        _ => return Err(Chip8Error::UnknownInstructionError),
    });

    Ok(string)
}
