// #![warn(clippy)]
pub mod audio;
pub mod display;
pub mod logger;
pub mod rom;
pub mod util;
pub mod vm;
pub mod cpu;


extern crate sdl2;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nfd;
extern crate sdl2_sys;
