extern crate sdl2;
extern crate sdl2_sys;

use vm::{Chip8, Chip8Error};
use gfx::GfxSubsystem;

use std;
use ui::sdl2::EventPump;
use ui::sdl2::event::Event;
use ui::sdl2::keyboard::Keycode;
use ui::sdl2::render::{Canvas, TextureCreator};
use ui::sdl2::video::{Window, WindowContext};
use ui::sdl2::ttf::Sdl2TtfContext;

use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

const DEFAULT_SPEED: f32 = 100f32;

fn vm_loop(speed_in: Receiver<i32>) -> Receiver<()> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        let mut speed = DEFAULT_SPEED;

        loop {
            for msg in speed_in.try_iter() {
                match msg {
                    i if i > 0 => speed *= 1.5,
                    i if i < 0 => speed /= 1.5,
                    _ => (),
                }
            }

            let ms = std::cmp::max(1, (1.0f32 / speed * 1000f32) as u64); // TODO improve
            thread::sleep(Duration::from_millis(ms));

            sender.send(()).unwrap();
        }
    });

    return receiver;
}

fn ui_loop() -> Receiver<()> {
    let (sender, receiver) = channel();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(16));
            sender.send(()).unwrap();
        }
    });

    return receiver;
}

pub struct System<'ttf, 'b> {
    vm: Chip8,
    gfx: GfxSubsystem<'ttf, 'b>,
    events: EventPump,
    paused: bool,
    vm_error: Option<Chip8Error>,
}

pub struct SystemSdl2Context<'ttf, 'b> {
    pub canvas: Canvas<Window>,
    pub ttf: &'ttf Sdl2TtfContext,
    pub texture_creator: &'b TextureCreator<WindowContext>,
    pub event_pump: EventPump,
}

impl<'ttf, 'b> System<'ttf, 'b> {
    pub fn new(ctx: SystemSdl2Context<'ttf, 'b>, vm: Chip8) -> System<'ttf, 'b> {
        let gfx = GfxSubsystem::new(ctx.canvas, ctx.ttf, &ctx.texture_creator);

        System {
            events: ctx.event_pump,
            vm: vm,
            gfx: gfx,
            paused: false,
            vm_error: None,
        }
    }

    pub fn run(&mut self) {
        let (speed_tx, speed_rx) = channel();
        let render_rx = ui_loop();
        let vm_rx = vm_loop(speed_rx);

        'running2: loop {
            for event in self.events.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running2,
                    Event::KeyDown {
                        keycode: Some(code),
                        ..
                    } => match code {
                        Keycode::Escape => break 'running2,
                        Keycode::LeftBracket => speed_tx.send(-1).unwrap(),
                        Keycode::RightBracket => speed_tx.send(1).unwrap(),
                        Keycode::F5 => self.paused = !self.paused,
                        Keycode::F6 => if let Err(err) = self.vm.step() {
                            self.vm_error = Some(err);
                            self.paused = true;
                        },
                        _ => (),
                    },
                    _ => (),
                }
            }

            select! {
                _ = vm_rx.recv() => {
                    if !self.paused {
                        if let Err(err) = self.vm.step() {
                            self.vm_error = Some(err);
                            self.paused = true;
                        }
                    }
                },
                _ = render_rx.recv() => {
                    let state = self.vm.state();
                    self.gfx.redraw(&state);
                }
            }
        }
    }
}
