extern crate sdl2;

extern crate lib;
use lib::ui::{System, GfxSubsystem};

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 695;

fn main() {

    let sdl_context = sdl2::init().unwrap();
    let mut ttf_context = sdl2::ttf::init().unwrap();
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

    let gfx = GfxSubsystem::new(canvas, &mut ttf_context, &texture_creator);
    let mut system = System::new(gfx, event_pump);

    system.run();
}
