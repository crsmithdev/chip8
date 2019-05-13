use std::cmp::{max, min};
use std::thread;
use std::time::{Duration, SystemTime};

use logger::Logger;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::{EventPump, Sdl};

use cpu::Chip8;
use display::{Display, VMState};
use rom;

const HZ_MAX: u32 = 2000;
const HZ_DEFAULT: u32 = 500;

pub struct VMArgs<'a> {
    pub sdl: &'a Sdl,
    pub ttf: &'a Sdl2TtfContext,
    pub log: &'static Logger,
}

struct RunState {
    cpu_state: CPUState,
    last_step: SystemTime,
    hz: u32,
}

#[derive(Copy, Clone, PartialEq)]
enum CPUState {
    Paused,
    Running,
    OneStep,
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

pub struct VM<'a> {
    cpu: Chip8,
    display: Display<'a>,
    events: EventPump,
}

impl<'a> VM<'a> {
    pub fn new(args: VMArgs<'a>) -> VM<'a> {
        VM {
            cpu: Chip8::new(),
            display: Display::new(args.sdl, args.ttf, args.log),
            events: args.sdl.event_pump().unwrap(),
        }
    }

    pub fn start(&mut self) {
        let bytes = self.cpu.load_bytes(&rom::BOOT).unwrap();
        info!("Loaded {} bytes", bytes);

        let mut fps = FPSCounter::new(30);

        // TODO reset
        let mut state = RunState {
            cpu_state: CPUState::Running,
            last_step: SystemTime::now(),
            hz: HZ_DEFAULT,
        };

        'running: loop {
            for event in self.events.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(code),
                        ..
                    } => match code {
                        Keycode::Escape => break 'running,
                        Keycode::LeftBracket => state.hz = Self::dec_hz(state.hz),
                        Keycode::RightBracket => state.hz = Self::inc_hz(state.hz),
                        Keycode::F5 => state.cpu_state = Self::toggle_pause(state.cpu_state),
                        Keycode::F6 => state.cpu_state = CPUState::OneStep,
                        _ => (),
                    },
                    _ => (),
                }
            }

            let now = SystemTime::now();
            let cycles = match state.cpu_state {
                CPUState::Paused => 0,
                CPUState::OneStep => {
                    state.cpu_state = CPUState::Paused;
                    1
                }
                CPUState::Running => Self::cycles_since(state.last_step, state.hz),
            };

            for _ in 0..cycles {
                if let Err(err) = self.cpu.step() {
                    error!("CPU Error: {}", err);
                    state.cpu_state = CPUState::Paused;
                }
            }

            let state2 = VMState {
                vm: &self.cpu,
                hz: state.hz as i32,
                fps: fps.fps() as i32,
            };
            self.display.update(&state2);

            let mut delay = fps.frame();
            if state.cpu_state == CPUState::Paused || !self.display.focused() {
                delay = 50;
            }

            state.last_step = now;
            thread::sleep(Duration::from_millis(delay as u64));
        }
    }

    fn dec_hz(hz: u32) -> u32 {
        let inc = match hz {
            n if n <= 10 => n - 1,
            n if n <= 100 => n - 10,
            n => n - 100,
        };

        min(max(inc, 0), HZ_MAX)
    }

    fn inc_hz(hz: u32) -> u32 {
        let dec = match hz {
            n if n <= 10 => n + 1,
            n if n <= 100 => n + 10,
            n => n + 100,
        };

        min(max(dec, 0), HZ_MAX)
    }

    fn toggle_pause(state: CPUState) -> CPUState {
        match state {
            CPUState::Running => CPUState::Paused,
            CPUState::Paused => CPUState::Running,
            _ => state,
        }
    }

    fn cycles_since(time: SystemTime, hz: u32) -> u32 {
        let now = SystemTime::now();
        let elapsed = now.duration_since(time).unwrap().as_millis() as u32;
        let ms_per_cycle = 1000 / hz;
        elapsed / ms_per_cycle
    }
}
