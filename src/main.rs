// #![warn(clippy)]
mod cpu;
mod display;
mod logger;
mod rom;
mod util;
mod vm;
mod audio;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nfd;
extern crate rand;
extern crate sdl2;
extern crate sdl2_sys;

use vm::{VMArgs, VM};

fn main() {
    let context = &sdl2::init().unwrap();
    let args = VMArgs {
        sdl: context,
        ttf: &sdl2::ttf::init().unwrap(),
        audio: &context.audio().unwrap(),
        log: &logger::init(),
        cache: &display::TextureCache::new(),
    };

    VM::new(args).start();
}
