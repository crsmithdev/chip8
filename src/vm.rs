use cpu::{Chip8, Chip8State};
use display::{Display, TextureCache};
use logger::Logger;
use nfd::Response;
use rom;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::{EventPump, Sdl};
use std::cmp::{max, min};
use std::env::current_dir;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::{Duration, SystemTime};
use util::FPSCounter;

const HZ_MAX: u32 = 2000;
const HZ_MIN: u32 = 1;
const HZ_DEFAULT: u32 = 500;
const FPS_DEFAULT: u32 = 60;
const DELAY_BG: u64 = 50;

macro_rules! round {
    ($num:expr, $nearest:expr) => {
        ($num / $nearest) * $nearest
    };
}

pub struct VMArgs<'a> {
    pub sdl: &'a Sdl,
    pub ttf: &'a Sdl2TtfContext,
    pub log: &'static Logger,
    pub cache: &'a TextureCache,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CPUState {
    Stopped,
    Paused,
    Running,
    OneStep,
}

pub struct RunState {
    pub cpu_state: CPUState,
    pub last_step: SystemTime,
    pub fps: i32,
    pub hz: u32,
}

pub struct UpdateState<'a> {
    pub cpu: &'a Chip8State,
    pub run: &'a RunState,
}

pub struct VM<'a> {
    pub cpu: Chip8,
    display: Display<'a>,
    events: EventPump,
    state: RunState,
}

impl<'a> VM<'a> {
    pub fn new(args: VMArgs<'a>) -> VM<'a> {
        let chip8 = Chip8::new();

        VM {
            display: Display::new(args.sdl, args.ttf, args.log, args.cache),
            events: args.sdl.event_pump().unwrap(),
            cpu: chip8,
            state: RunState {
                cpu_state: CPUState::Stopped,
                last_step: SystemTime::UNIX_EPOCH,
                hz: HZ_DEFAULT,
                fps: 0,
            },
        }
    }

    pub fn start(&mut self) {
        self.state.cpu_state = CPUState::Running;
        self.state.last_step = SystemTime::now();
        let mut fps = FPSCounter::new(FPS_DEFAULT);

        self.cpu.load_bytes(&rom::BOOT).unwrap();
        info!("Started");

        'runloop: loop {
            if self.state.cpu_state == CPUState::Stopped {
                break 'runloop;
            }

            self.handle_events();
            let cycles = self.cycles_since();

            if cycles > 0 {
                for _ in 0..cycles {
                    if let Err(err) = self.cpu.execute_cycle() {
                        error!("CPU Error: {}", err);
                        self.state.cpu_state = CPUState::Paused;
                    }
                }
                self.state.last_step = SystemTime::now();
            }

            if self.state.cpu_state == CPUState::OneStep {
                self.state.cpu_state = CPUState::Paused;
            }

            self.state.fps = fps.fps() as i32;
            self.display.update(&UpdateState {
                cpu: self.cpu.state(),
                run: &self.state,
            });

            let delay = fps.frame() as u64;
            let paused = self.state.cpu_state == CPUState::Paused;
            let focused = self.display.focused();
            let ms = delay * if paused || !focused { DELAY_BG } else { 1 };

            thread::sleep(Duration::from_millis(ms));
        }
    }

    fn handle_events(&mut self) {
        while let Some(event) = self.events.poll_event() {
            match event {
                Event::Quit { .. } => self.state.cpu_state = CPUState::Stopped,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => self.state.cpu_state = CPUState::Stopped,
                    Keycode::LeftBracket => self.dec_hz(),
                    Keycode::RightBracket => self.inc_hz(),
                    Keycode::F1 => self.load_file(),
                    Keycode::F2 => self.reload(),
                    Keycode::F3 => self.restart(),
                    Keycode::F5 => self.toggle_pause(),
                    Keycode::F6 => self.state.cpu_state = CPUState::OneStep,
                    Keycode::Num1 => self.key_down(0x1),
                    Keycode::Num2 => self.key_down(0x2),
                    Keycode::Num3 => self.key_down(0x3),
                    Keycode::Num4 => self.key_down(0xC),
                    Keycode::Q => self.key_down(0x4),
                    Keycode::W => self.key_down(0x5),
                    Keycode::E => self.key_down(0x6),
                    Keycode::R => self.key_down(0xD),
                    Keycode::A => self.key_down(0x7),
                    Keycode::S => self.key_down(0x8),
                    Keycode::D => self.key_down(0x9),
                    Keycode::F => self.key_down(0xE),
                    Keycode::Z => self.key_down(0xA),
                    Keycode::X => self.key_down(0x0),
                    Keycode::C => self.key_down(0xB),
                    Keycode::V => self.key_down(0xF),
                    _ => (),
                },
                Event::KeyUp {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Num1 => self.key_up(0x1),
                    Keycode::Num2 => self.key_up(0x2),
                    Keycode::Num3 => self.key_up(0x3),
                    Keycode::Num4 => self.key_up(0xC),
                    Keycode::Q => self.key_up(0x4),
                    Keycode::W => self.key_up(0x5),
                    Keycode::E => self.key_up(0x6),
                    Keycode::R => self.key_up(0xD),
                    Keycode::A => self.key_up(0x7),
                    Keycode::S => self.key_up(0x8),
                    Keycode::D => self.key_up(0x9),
                    Keycode::F => self.key_up(0xE),
                    Keycode::Z => self.key_up(0xA),
                    Keycode::X => self.key_up(0x0),
                    Keycode::C => self.key_up(0xB),
                    Keycode::V => self.key_up(0xF),
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn load_file(&mut self) {
        let current = format!("{}", current_dir().unwrap().display());
        let mut bytes = Vec::new();

        match nfd::open_file_dialog(None, Some(&current)) {
            Err(error) => error!("Error loading file: {}", error),
            Ok(Response::OkayMultiple(_)) => error!("Multiple files selected"),
            Ok(Response::Okay(path)) => {
                let mut file = File::open(path).unwrap();
                file.read_to_end(&mut bytes).unwrap();
                self.cpu.hard_reset();
                self.cpu.load_bytes(&bytes).unwrap();
            }
            _ => (),
        };
    }

    fn reload(&mut self) {
        self.cpu.soft_reset();
        self.state = RunState {
            cpu_state: CPUState::Running,
            last_step: SystemTime::now(),
            hz: HZ_DEFAULT,
            fps: 0,
        };
        info!("Reloaded");
    }

    fn restart(&mut self) {
        self.cpu.load_bytes(&rom::BOOT).unwrap();
        self.cpu.hard_reset();
        self.state = RunState {
            cpu_state: CPUState::Running,
            last_step: SystemTime::now(),
            hz: HZ_DEFAULT,
            fps: 0,
        };
        info!("Restarted");
    }

    fn key_down(&mut self, k: usize) {
        self.cpu.press_key(k);
    }

    fn key_up(&mut self, k: usize) {
        self.cpu.release_key(k);
    }

    fn dec_hz(&mut self) {
        let inc = match self.state.hz {
            n if n < 20 => n - 1,
            n if n <= 100 => round!(n, 10) - 10,
            n => round!(n, 100) - 100,
        };

        self.state.hz = min(max(inc, HZ_MIN), HZ_MAX);
    }

    fn inc_hz(&mut self) {
        let dec = match self.state.hz {
            n if n < 20 => n + 1,
            n if n <= 100 => round!(n, 10) + 10,
            n => round!(n, 100) + 100,
        };

        self.state.hz = min(max(dec, HZ_MAX), HZ_MAX);
    }

    fn toggle_pause(&mut self) {
        self.state.cpu_state = match self.state.cpu_state {
            CPUState::Running => {
                info!("Paused");
                CPUState::Paused
            }
            CPUState::Paused => {
                info!("Resumed");
                CPUState::Running
            }
            state => state,
        };
    }

    fn cycles_since(&self) -> u32 {
        match self.state.cpu_state {
            CPUState::Stopped => 0,
            CPUState::Paused => 0,
            CPUState::OneStep => 1,
            CPUState::Running => {
                let now = SystemTime::now();
                let elapsed = now
                    .duration_since(self.state.last_step)
                    .unwrap()
                    .as_millis() as u32;
                let ms_per_cycle = 1000 / self.state.hz;
                elapsed / ms_per_cycle
            }
        }
    }
}
