#![feature(mpsc_select)]

extern crate lib;
extern crate sdl2;

use lib::vm::Vm;
use lib::timer::Timer;
use lib::gfx::GfxSubsystem;

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
    let mut vm = Vm::new();
    let timer = Timer::new();
    let receiver = &timer.receiver;

    vm.run();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => match code {
                    Keycode::Escape => break 'running,
                    Keycode::LeftBracket => vm.speed(1.0 / 1.5),
                    Keycode::RightBracket => vm.speed(1.5),
                    Keycode::F5 => vm.pause(),
                    Keycode::F6 => vm.step(),
                    _ => (),
                },
                _ => (),
            }
        }

        select! {
            _ = receiver.recv() => {
                let state = vm.state();
                gfx.redraw(&state);
            }
        }
    }
}
