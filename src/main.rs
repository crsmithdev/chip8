mod cpu;
mod display;
mod logger;
mod rom;
mod vm;
mod cache;

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
    let log = logger::init();
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let args = VMArgs {
        sdl: &sdl_context,
        ttf: &ttf_context,
        log: &log,
    };

    let mut vm = VM::new(args);
    vm.start();
}
