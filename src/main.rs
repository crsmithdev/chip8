extern crate lib;
extern crate sdl2;
#[macro_use]
extern crate lazy_static;

use lib::cpu::Chip8;
use lib::display::{Context, Display};
use lib::rom;
use sdl2::pixels::PixelFormatEnum;
use std::cmp::{max, min};
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const FONT_SIZE: u16 = 28;
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 610;

lazy_static! {
    static ref FONT_PATH: &'static Path = Path::new("../../resources/SourceCodePro-Semibold.ttf");
}

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
    let screen = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
        .unwrap();

    let mut font = ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap();
    font.set_hinting(sdl2::ttf::Hinting::Mono);

    let mut display = Display::new();
    let mut context = Context {
        canvas: canvas,
        screen: screen,
        font: font,
    };

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
        display.redraw(&mut context, &state);

        thread::sleep(Duration::from_millis(5));
    }
}
