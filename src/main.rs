extern crate sdl2;
extern crate lib;

use lib::ui::{System, SystemSdl2Context};
use lib::vm::Chip8;
use lib::rom;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 695;

fn main() {

    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let window = sdl_context
        .video().unwrap()
        .window("CHIP-8", WINDOW_WIDTH, WINDOW_HEIGHT)
        .allow_highdpi()
        .build().unwrap();

    let canvas = window
        .into_canvas()
        .present_vsync()
        .build().unwrap();

    let texture_creator = canvas.texture_creator();

    let system_context = SystemSdl2Context {
        canvas: canvas,
        ttf: &ttf_context,
        texture_creator: &texture_creator,
        event_pump: event_pump,
    };

    let mut vm = Chip8::new();
    vm.load_rom(&rom::BOOT).unwrap();
    let mut system = System::new(system_context, vm);

    system.run();
}
