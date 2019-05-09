extern crate lib;
extern crate sdl2;
extern crate sdl2_sys;
#[macro_use]
extern crate lazy_static;
use lib::cpu::Chip8;
use lib::display::Display;
use lib::rom;
//use sdl2_sys::SDL_WindowFlags;
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
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut display = Display::new(&sdl_context, &ttf_context);

    let mut cpu = Chip8::new();
    cpu.load_rom(&rom::BOOT).unwrap();

    let mut freq = 500;
    let mut paused = false;
    let mut step = false;
    let mut last_time = SystemTime::now();

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

        let now = SystemTime::now();
        let elapsed = now.duration_since(last_time).unwrap().as_millis();
        last_time = now;
        let ms_per_cycle = 1000 / freq;
        let mut cycles = (elapsed as f32 / ms_per_cycle as f32).round() as usize;
        // TODO remaining ms

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

        let state = cpu.state();
        display.update(&state);
        let duration = 16;
        /*
        let flags = display.canvas.window().window_flags() as u32;
        let focused = SDL_WindowFlags::SDL_WINDOW_INPUT_FOCUS as u32;
        let has_focus = (flags & focused) == focused;
        if !has_focus || paused {
            duration = 50;
        }
        */

        thread::sleep(Duration::from_millis(duration));
    }
}
