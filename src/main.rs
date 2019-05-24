mod cache;
mod cpu;
mod display;
mod logger;
mod rom;
mod util;
mod vm;

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
    let args = VMArgs {
        sdl: &sdl2::init().unwrap(),
        ttf: &sdl2::ttf::init().unwrap(),
        log: &logger::init(),
        cache: &display::TextureCache::new(),
    };

    VM::new(args).start();
}
