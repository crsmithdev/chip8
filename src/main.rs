extern crate lib;
extern crate sdl2;

use lib::gfx::GfxSubsystem;
use lib::cpu::{Chip8};
use std::time::{SystemTime, Duration};
use std::thread;
use lib::rom;

use sdl2::keyboard::Keycode;
use sdl2::event::Event;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 610;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let window = sdl_context
        .video()
        .unwrap()
        .window("CHIP-8", WINDOW_WIDTH, WINDOW_HEIGHT)
        .allow_highdpi()
        .build()
        .unwrap();

    let canvas = window.into_canvas().present_vsync().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let mut gfx = GfxSubsystem::new(canvas, &ttf_context, &texture_creator);
    let mut vm = Chip8::new();
    vm.load_rom(&rom::BOOT).unwrap();
    let freq = 500;

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
                    /*
                    Keycode::LeftBracket => vm.speed(1.0 / 1.5),
                    Keycode::RightBracket => vm.speed(1.5),
                    Keycode::F5 => vm.pause(),
                    Keycode::F6 => vm.step(),
                    */
                    _ => (),
                },
                _ => (),
            }
        }

        let now = SystemTime::now();
        let elapsed = now.duration_since(last_time).unwrap().as_millis();
        last_time = now;
        let cycles = elapsed * (1000 / freq);
        for _ in 0..cycles {
            if let Err(err) = vm.step() {
                println!("error: {:?}", err);
                break;
            }
        }

        let state = vm.state();
        gfx.redraw(&state);

        thread::sleep(Duration::from_millis(5));
    }
}
