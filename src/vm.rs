use std::cmp::{max, min};
use std::env::current_dir;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::{Duration, SystemTime};

use logger::Logger;
use nfd::Response;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::{EventPump, Sdl};

use cpu::Chip8;
use display::Display;
use rom;

const HZ_MAX: u32 = 2000;
const HZ_DEFAULT: u32 = 500;

pub struct VMArgs<'a> {
    pub sdl: &'a Sdl,
    pub ttf: &'a Sdl2TtfContext,
    pub log: &'static Logger,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CPUState {
    Stopped,
    Paused,
    Running,
    OneStep,
}

pub struct VMState {
    pub cpu: Chip8,
    pub cpu_state: CPUState,
    pub last_step: SystemTime,
    pub fps: i32,
    pub hz: u32,
}

pub struct VM<'a> {
    display: Display<'a>,
    events: EventPump,
    state: VMState,
}

impl<'a> VM<'a> {
    pub fn new(args: VMArgs<'a>) -> VM<'a> {
        VM {
            display: Display::new(args.sdl, args.ttf, args.log),
            events: args.sdl.event_pump().unwrap(),
            state: VMState {
                cpu: Chip8::new(),
                cpu_state: CPUState::Stopped,
                last_step: SystemTime::UNIX_EPOCH,
                hz: HZ_DEFAULT,
                fps: 0,
            },
        }
    }

    pub fn start(&mut self) {
        let bytes = self.state.cpu.load_bytes(&rom::BOOT).unwrap();
        info!("Loaded {} bytes", bytes);

        let mut fps = FPSCounter::new(30);
        self.state.cpu_state = CPUState::Running;
        self.state.last_step = SystemTime::now();

        'running: loop {
            if self.state.cpu_state == CPUState::Stopped {
                break 'running;
            }
            self.handle_events();
            let cycles = self.cycles_since();

            if cycles > 0 {
                for _ in 0..cycles {
                    if let Err(err) = self.state.cpu.step() {
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
            self.display.update(&self.state);

            let mut delay = fps.frame();
            if self.state.cpu_state == CPUState::Paused || !self.display.focused() {
                delay = 50;
            }

            thread::sleep(Duration::from_millis(delay as u64));
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
        let result = nfd::open_file_dialog(None, Some(&current)).unwrap_or_else(|e| {
            panic!(e);
        });

        match result {
            Response::Okay(file_path) => {
                // TODO errors
                let mut file = File::open(file_path).unwrap();
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes).unwrap();
                self.state.cpu.reset();
                self.state.cpu.load_bytes(&bytes).unwrap();
            }
            Response::OkayMultiple(files) => println!("Files {:?}", files),
            Response::Cancel => println!("User canceled"),
        }
    }

    fn key_down(&mut self, k: usize) {
        self.state.cpu.keys[k] = true;
    }

    fn key_up(&mut self, k: usize) {
        self.state.cpu.keys[k] = false;
    }

    fn dec_hz(&mut self) {
        let inc = match self.state.hz {
            n if n < 20 => n - 1,
            n if n >= 20 && n <= 100 => ((n / 10) * 10) - 10,
            n => ((n / 100) * 100) - 100,
        };

        self.state.hz = min(max(inc, 10), HZ_MAX);
    }

    fn inc_hz(&mut self) {
        let dec = match self.state.hz {
            n if n <= 100 => ((n / 10) * 10) + 10,
            n => ((n / 100) * 100) + 100,
        };

        self.state.hz = min(max(dec, 1), HZ_MAX);
    }

    fn toggle_pause(&mut self) {
        self.state.cpu_state = match self.state.cpu_state {
            CPUState::Running => CPUState::Paused,
            CPUState::Paused => CPUState::Running,
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

struct FPSCounter {
    last_frame: SystemTime,
    last_fps: SystemTime,
    fps_actual: f32,
    frames: u32,
    ms_per_frame: f32,
}

impl FPSCounter {
    fn new(fps: u32) -> FPSCounter {
        FPSCounter {
            last_frame: SystemTime::now(),
            last_fps: SystemTime::now(),
            fps_actual: 0.0,
            frames: 0,
            ms_per_frame: 1000.0 / fps as f32,
        }
    }

    fn frame(&mut self) -> u32 {
        let now = SystemTime::now();
        let delta = now.duration_since(self.last_frame).unwrap().as_millis() as f32;
        self.frames += 1;
        self.last_frame = now;
        (self.ms_per_frame - delta).max(0.0) as u32
    }

    fn fps(&mut self) -> f32 {
        let now = SystemTime::now();
        let delta = now.duration_since(self.last_fps).unwrap().as_millis();

        if delta > 5000 {
            self.fps_actual = self.frames as f32 / (delta as f32 / 1000.0);
            self.frames = 0;
            self.last_fps = now;
        }

        self.fps_actual
    }
}
