mod cpu;
mod display;
mod logger;
mod rom;

extern crate sdl2;
extern crate sdl2_sys;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rand;
use rand::Rng;

use cpu::Chip8;
use display::{Display, VMState};
use std::cmp::{max, min};
use std::thread;
use std::time::{Duration, SystemTime};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::path::Path;

lazy_static! {
    static ref FONT_PATH: &'static Path = Path::new("../../resources/SourceCodePro-Semibold.ttf");
}

fn main() {
    let log = logger::init();
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut display = Display::new(&sdl_context, &ttf_context, &log);

    let mut cpu = Chip8::new();
    cpu.load_bytes(&rom::BOOT).unwrap();

    let mut freq = 500;
    let mut paused = false;
    let mut step = false;
    let mut last_time = SystemTime::now();
    let mut fps_time = SystemTime::now();
    let mut fps = 0;

    info!("Started");
    let mut counter = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => break 'running,
                    Keycode::LeftBracket => freq = max(freq - 50, 0),
                    Keycode::RightBracket => freq = min(freq + 50, 1000),
                    Keycode::F5 => paused = !paused,
                    Keycode::F6 => step = true,
                    _ => (),
                },
                _ => (),
            }
        }
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(0, 100);
        if n == 1 {
            info!("Message {:04}", rng.gen_range(0, 1000));
        }

        let now = SystemTime::now();
        let elapsed = now.duration_since(last_time).unwrap().as_millis();
        last_time = now;
        let ms_per_cycle = 1000 / freq;
        let mut cycles = (elapsed as f32 / ms_per_cycle as f32).round() as usize;
        counter += 1;

        if !paused || step {
            if step {
                cycles = 1;
                step = false;
            }
            for _ in 0..cycles {
                if let Err(err) = cpu.step() {
                    println!("error: {:?}", err);
                    break;
                }
            }
        }

        let fps_secs = now.duration_since(fps_time).unwrap().as_secs();
        if fps_secs > 5 {
            let n = counter / fps_secs;
            fps_time = now;
            counter = 0;
            fps = n;
        }

        let state = VMState {
            vm: &cpu,
            hz: freq,
            fps: fps as i32,
        };
        display.update(&state);
        let mut duration = 1;
        if !display.focused() || paused {
            duration = 50;
        }

        thread::sleep(Duration::from_millis(duration));
    }
}
