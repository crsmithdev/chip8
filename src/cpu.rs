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

#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    ClearScreen,
    Return,
    Jump { address: usize },
    Call { address: usize },
    SkipByteEqual { x: usize, byte: u8 },
    SkipByteNotEqual { x: usize, byte: u8 },
    SkipEqual { x: usize, y: usize },
    SkipNotEqual { x: usize, y: usize },
    LoadByte { x: usize, byte: u8 },
    AddByte { x: usize, byte: u8 },
    Load { x: usize, y: usize },
    Or { x: usize, y: usize },
    And { x: usize, y: usize },
    Xor { x: usize, y: usize },
    Add { x: usize, y: usize },
    Sub { x: usize, y: usize },
    ShiftRight { x: usize },
    SubReverse { x: usize, y: usize },
    ShiftLeft { x: usize },
    LoadAddress { address: usize },
    JumpOffset { address: usize },
    Random { x: usize, byte: u8 },
    Draw { x: usize, y: usize, n: u8 },
    SkipKeyPressed { x: usize },
    SkipNotPressed { x: usize },
    LoadFromDelayTimer { x: usize },
    WaitKey { x: usize },
    LoadDelayTimer { x: usize },
    LoadSoundTimer { x: usize },
    AddAddress { x: usize },
    LoadFont { x: usize },
    BCD { x: usize },
    Save { x: usize },
    Restore { x: usize },
    Unknown { instruction: u16 },
}

impl OpCode {
    pub fn decode(instruction: u16) -> OpCode {
        let address = (instruction & 0xFFF) as usize;
        let byte = (instruction & 0xFF) as u8;
        let x = (instruction >> 8 & 0xF) as usize;
        let y = (instruction >> 4 & 0xF) as usize;
        let n = (instruction & 0xF) as u8;

        match instruction {
            0x00E0 => OpCode::ClearScreen,
            0x00EE => OpCode::Return,
            i if i & 0xF000 == 0x1000 => OpCode::Jump { address },
            i if i & 0xF000 == 0x2000 => OpCode::Call { address },
            i if i & 0xF000 == 0x3000 => OpCode::SkipByteEqual { x, byte },
            i if i & 0xF000 == 0x4000 => OpCode::SkipByteNotEqual { x, byte },
            i if i & 0xF000 == 0x5000 => OpCode::SkipEqual { x, y },
            i if i & 0xF000 == 0x6000 => OpCode::LoadByte { x, byte },
            i if i & 0xF000 == 0x7000 => OpCode::AddByte { x, byte },
            i if i & 0xF00F == 0x8000 => OpCode::Load { x, y },
            i if i & 0xF00F == 0x8001 => OpCode::Or { x, y },
            i if i & 0xF00F == 0x8002 => OpCode::And { x, y },
            i if i & 0xF00F == 0x8003 => OpCode::Xor { x, y },
            i if i & 0xF00F == 0x8004 => OpCode::Add { x, y },
            i if i & 0xF00F == 0x8005 => OpCode::Sub { x, y },
            i if i & 0xF00F == 0x8006 => OpCode::ShiftRight { x },
            i if i & 0xF00F == 0x8007 => OpCode::SubReverse { x, y },
            i if i & 0xF00F == 0x800E => OpCode::ShiftLeft { x },
            i if i & 0xF00F == 0x9000 => OpCode::SkipNotEqual { x, y },
            i if i & 0xF000 == 0xA000 => OpCode::LoadAddress { address },
            i if i & 0xF000 == 0xB000 => OpCode::JumpOffset { address },
            i if i & 0xF000 == 0xC000 => OpCode::Random { x, byte },
            i if i & 0xF000 == 0xD000 => OpCode::Draw { x, y, n },
            i if i & 0xF0FF == 0xE09E => OpCode::SkipKeyPressed { x },
            i if i & 0xF0FF == 0xE0A1 => OpCode::SkipNotPressed { x },
            i if i & 0xF0FF == 0xF007 => OpCode::LoadFromDelayTimer { x },
            i if i & 0xF0FF == 0xF00A => OpCode::WaitKey { x },
            i if i & 0xF0FF == 0xF015 => OpCode::LoadDelayTimer { x },
            i if i & 0xF0FF == 0xF018 => OpCode::LoadSoundTimer { x },
            i if i & 0xF0FF == 0xF01E => OpCode::AddAddress { x },
            i if i & 0xF0FF == 0xF029 => OpCode::LoadFont { x },
            i if i & 0xF0FF == 0xF033 => OpCode::BCD { x },
            i if i & 0xF0FF == 0xF055 => OpCode::Save { x },
            i if i & 0xF0FF == 0xF065 => OpCode::Restore { x },
            _ => OpCode::Unknown { instruction },
        }
    }

    pub fn disassemble(instruction: u16) -> (String, String) {
        use cpu::OpCode::*;

        let (op, params) = match Self::decode(instruction) {
            ClearScreen => ("CLS", String::new()),
            Return => ("RET", String::new()),
            Jump { address } => ("JUMP", format!("#{:04X}", address)),
            Call { address } => ("CALL", format!("#{:04X}", address)),
            SkipByteEqual { x, byte } => ("SE", format!("V{:X}, {:02X}", x, byte)),
            SkipByteNotEqual { x, byte } => ("SNE", format!("V{:X}, {:02X}", x, byte)),
            SkipEqual { x, y } => ("SE", format!("V{:X}, V{}", x, y)),
            LoadByte { x, byte } => ("LOAD", format!("V{:X}, {:02X}", x, byte)),
            AddByte { x, byte } => ("ADD", format!("V{:X}, {:02X}", x, byte)),
            Load { x, y } => ("LOAD", format!("V{:X}, V{:X}", x, y)),
            Or { x, y } => ("OR", format!("V{:X}, V{:X}", x, y)),
            And { x, y } => ("AND", format!("V{:X}, V{:X}", x, y)),
            Xor { x, y } => ("XOR", format!("V{:X}, V{:X}", x, y)),
            Add { x, y } => ("ADD", format!("V{:X}, V{:X}", x, y)),
            Sub { x, y } => ("SUB", format!("V{:X}, V{:X}", x, y)),
            ShiftRight { x } => ("SHR", format!("V{:X}", x)),
            SubReverse { x, y } => ("SUBN", format!("V{:X}, V{:X}", x, y)),
            ShiftLeft { x } => ("SHL", format!("V{:X}", x)),
            SkipNotEqual { x, y } => ("SNE", format!("V{:X}, V{:X}", x, y)),
            LoadAddress { address } => ("LOAD", format!("I, #{:04X}", address)),
            JumpOffset { address } => ("JUMP", format!("V0, #{:04X}", address)),
            Random { x, byte } => ("RND", format!("V{:X}, #{:02X}", x, byte)),
            Draw { x, y, n } => ("DRAW", format!("V{:X}, V{:X}, {}", x, y, n)),
            SkipKeyPressed { x } => ("SKP", format!("V{:X}", x)),
            SkipNotPressed { x } => ("SKNP", format!("V{:X}", x)),
            LoadFromDelayTimer { x } => ("LOAD", format!("V{:X}, DT", x)),
            WaitKey { x } => ("LOAD", format!("V{:X}, K", x)),
            LoadDelayTimer { x } => ("LOAD", format!("DT, V{:X}", x)),
            LoadSoundTimer { x } => ("LOAD", format!("ST, V{:X}", x)),
            AddAddress { x } => ("ADD", format!("I, V{:X}", x)),
            LoadFont { x } => ("FONT", format!("I, V{:X}", x)),
            BCD { x } => ("BCD", format!("I, V{:X}", x)),
            Save { x } => ("SAV", format!("[I], V{:X}", x)),
            Restore { x } => ("RST", format!("V{:X}, [I]", x)),
            Unknown { .. } => ("???", String::new()),
        };

        (op.to_owned(), params)
    }
}

pub struct Chip8State {
    video: [u8; Chip8State::VIDEO_SIZE],
    memory: [u8; Chip8State::MEMORY_SIZE],
    v: [u8; Chip8State::N_REGISTERS],
    stack: [usize; Chip8State::STACK_SIZE],
    keys: [bool; Chip8State::N_KEYS],
    pc: usize,
    sp: usize,
    i: usize,
    dt: u8,
    st: u8,
    error: Option<Chip8Error>,
}

impl Default for Chip8State {
    fn default() -> Self {
        let mut state = Chip8State {
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
        };
        state.memory[..rom::ROM.len()].clone_from_slice(&rom::ROM);
        state
    }
}

#[allow(dead_code)]
impl Chip8State {
    const PROGRAM_START: usize = 512;
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

    pub fn from_rom(bytes: &[u8]) -> Chip8State {
        let mut state = Self::default();
        let program_range = Self::PROGRAM_START..Self::PROGRAM_START + bytes.len();
        state.memory[program_range].clone_from_slice(&bytes);
        state
    }

    pub fn from_state(other: &Chip8State) -> Chip8State {
        let mut state = Chip8State::new();
        state.memory[..].clone_from_slice(&other.memory);
        state
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
    pub fn stack(&self) -> &[usize] {
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
    pub fn fetch(&self, address: usize) -> u16 {
        (self.memory[address as usize] as u16) << 8 | (self.memory[address as usize + 1] as u16)
    }
}

pub struct Chip8 {
    state: Chip8State,
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            state: Chip8State::new(),
        }
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Self::default()
    }

    #[inline(always)]
    pub fn state(&self) -> &Chip8State {
        &self.state
    }


    pub fn soft_reset(&mut self) {
        self.state = Chip8State::from_state(&self.state);
    }

    pub fn hard_reset(&mut self) {
        self.state = Chip8State::new();
    }

    pub fn press_key(&mut self, key: usize) {
        self.state.keys[key] = true;
    }

    pub fn release_key(&mut self, key: usize) {
        self.state.keys[key] = false;
    }

    pub fn load_rom(&mut self, bytes: &[u8]) -> Result<usize, Chip8Error> {
        if bytes.len() > Chip8State::MAX_PROGRAM_SIZE {
            return Err(Chip8Error::ProgramLoadError);
        }

        self.state = Chip8State::from_rom(bytes);
        Ok(bytes.len())
    }

    pub fn execute_cycle(&mut self) -> Result<(), Chip8Error> {
        let instruction = self.state.fetch(self.state.pc);

        if self.state.dt > 0 {
            self.state.dt -= 1;
        }

        if self.state.st > 0 {
            self.state.st -= 1;
        }

        self.state.pc += 2;
        self.execute(OpCode::decode(instruction))
    }

    fn execute(&mut self, opcode: OpCode) -> Result<(), Chip8Error> {
        use cpu::OpCode::*;
        match opcode {
            ClearScreen => self.clear_screen(),
            Jump { address } => self.jump(address),
            Call { address } => self.call(address),
            Return => self.return_(),
            SkipByteEqual { x, byte } => self.skip_byte_equal(x, byte),
            SkipByteNotEqual { x, byte } => self.skip_byte_not_equal(x, byte),
            SkipEqual { x, y } => self.skip_equal(x, y),
            LoadByte { x, byte } => self.load_byte(x, byte),
            AddByte { x, byte } => self.add_byte(x, byte),
            Load { x, y } => self.load(x, y),
            Or { x, y } => self.or(x, y),
            And { x, y } => self.and(x, y),
            Xor { x, y } => self.xor(x, y),
            Add { x, y } => self.add(x, y),
            Sub { x, y } => self.sub(x, y),
            ShiftRight { x } => self.shift_right(x),
            SubReverse { x, y } => self.sub_reverse(x, y),
            ShiftLeft { x } => self.shift_left(x),
            SkipNotEqual { x, y } => self.skip_not_equal(x, y),
            LoadAddress { address } => self.load_address(address),
            JumpOffset { address } => self.jump_offset(address),
            Random { x, byte } => self.random(x, byte),
            Draw { x, y, n } => self.draw(x, y, n),
            SkipKeyPressed { x } => self.skip_key_pressed(x),
            SkipNotPressed { x } => self.skip_not_pressed(x),
            LoadFromDelayTimer { x } => self.ld_v_dt(x),
            WaitKey { x } => self.ld_key(x),
            LoadDelayTimer { x } => self.ld_dt_v(x),
            LoadSoundTimer { x } => self.ld_st_v(x),
            AddAddress { x } => self.add_address(x),
            LoadFont { x } => self.load_font(x),
            BCD { x } => self.bcd(x),
            Save { x } => self.save(x),
            Restore { x } => self.restore(x),
            _ => return Err(Chip8Error::UnknownInstructionError),
        };

        Ok(())
    }

    #[inline(always)]
    fn clear_screen(&mut self) {
        self.state.video = [0; Chip8State::VIDEO_SIZE];
    }

    fn return_(&mut self) {
        let state = &mut self.state;
        if state.sp > 0 {
            state.sp -= 1;
            state.pc = state.stack[state.sp];
        } else {
            state.error = Some(Chip8Error::StackUnderflowError);
        }
    }

    #[inline(always)]
    fn jump(&mut self, address: usize) {
        self.state.pc = address;
    }

    #[inline(always)]
    fn jump_offset(&mut self, address: usize) {
        self.state.pc = self.state.v[0] as usize + address;
    }

    fn call(&mut self, address: usize) {
        let state = &mut self.state;

        if address >= Chip8State::MAX_PROGRAM_SIZE {
            state.error = Some(Chip8Error::AddressOutOfRangeError);
        } else if state.sp >= Chip8State::STACK_SIZE {
            state.error = Some(Chip8Error::StackOverflowError);
        } else {
            state.stack[state.sp] = state.pc;
            state.sp += 1;
            state.pc = address as usize;
        }
    }

    #[inline(always)]
    fn skip_byte_equal(&mut self, x: usize, byte: u8) {
        if self.state.v[x] == byte {
            self.state.pc += 2;
        }
    }

    #[inline(always)]
    fn skip_equal(&mut self, x: usize, y: usize) {
        if self.state.v[x] == self.state.v[y] {
            self.state.pc += 2;
        }
    }

    #[inline(always)]
    fn skip_byte_not_equal(&mut self, x: usize, byte: u8) {
        if self.state.v[x] != byte {
            self.state.pc += 2;
        }
    }

    #[inline(always)]
    fn skip_not_equal(&mut self, x: usize, y: usize) {
        if self.state.v[x] != self.state.v[y] {
            self.state.pc += 2;
        }
    }

    fn load_byte(&mut self, x: usize, byte: u8) {
        self.state.v[x] = byte;
    }

    fn load(&mut self, x: usize, y: usize) {
        self.state.v[x] = self.state.v[y];
    }

    fn load_address(&mut self, addr: usize) {
        self.state.i = addr;
    }

    fn add_byte(&mut self, x: usize, byte: u8) {
        self.state.v[x] = self.state.v[x].wrapping_add(byte);
    }

    fn add(&mut self, x: usize, y: usize) {
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

    fn shift_right(&mut self, x: usize) {
        let v = self.state.v[x];
        self.state.v[0xF] = v & 0x1;
        self.state.v[x] = v >> 1;
    }

    fn shift_left(&mut self, x: usize) {
        let v = self.state.v[x];
        self.state.v[0xF] = v >> 7;
        self.state.v[x] = v << 1;
    }

    fn sub_reverse(&mut self, x: usize, y: usize) {
        if self.state.v[y] > self.state.v[x] {
            self.state.v[0xF] = 1;
        } else {
            self.state.v[0xF] = 0;
        }
        self.state.v[x] = self.state.v[y].wrapping_sub(self.state.v[x]);
    }

    fn random(&mut self, x: usize, byte: u8) {
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

    fn skip_key_pressed(&mut self, x: usize) {
        if self.state.keys[x] {
            self.state.pc += 2;
        }
    }

    fn skip_not_pressed(&mut self, x: usize) {
        if !self.state.keys[x] {
            self.state.pc += 2;
        }
    }

    fn draw(&mut self, vx: usize, vy: usize, n: u8) {
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

    fn add_address(&mut self, x: usize) {
        self.state.i += self.state.v[x] as usize;
    }

    fn load_font(&mut self, x: usize) {
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

    macro_rules! execute {
        ($cpu:expr, $opcode:expr) => {
            let result = $cpu.execute($opcode);
            assert!(result.is_ok());
        };
    }

    #[test]
    fn clear_screen() {
        let mut cpu = Chip8::new();
        cpu.state.video[0] = 0xFF;

        execute!(cpu, OpCode::ClearScreen);
        assert_eq!(cpu.state.video[0], 0);
    }

    #[test]
    fn load_address() {
        let mut cpu = Chip8::new();
        execute!(cpu, OpCode::LoadAddress { address: 800 });
        assert_eq!(cpu.state.i, 800);
    }

    #[test]
    fn add_byte() {
        let mut cpu = Chip8::new();

        cpu.state.v[0] = 5;
        execute!(cpu, OpCode::AddByte { x: 0, byte: 5 });
        assert_eq!(cpu.state.v[0], 10);

        cpu.state.v[0] = 255;
        execute!(cpu, OpCode::AddByte { x: 0, byte: 2 });
        assert_eq!(cpu.state.v[0], 1);
    }

    #[test]
    fn add() {
        let mut cpu = Chip8::new();
        let op = OpCode::Add { x: 0, y: 1 };

        cpu.state.v[0] = 5;
        cpu.state.v[1] = 5;
        execute!(cpu, op);
        assert_eq!(cpu.state.v[0], 10);
        assert_eq!(cpu.state.v[0xF], 0);

        cpu.state.v[0] = 255;
        cpu.state.v[1] = 2;
        execute!(cpu, op);
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
