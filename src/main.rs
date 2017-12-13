#![feature(mpsc_select)]

#[macro_use]
extern crate lazy_static;

extern crate rand;
extern crate sdl2;
extern crate sdl2_sys;
extern crate libc;

extern crate lib;
use lib::chip8::chip8::{Chip8, Chip8Error};

use std::path::Path;
// use std::fs::File;
use std::error::Error;

use sdl2::pixels::PixelFormatEnum;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::rect::Rect;
use sdl2::rect::Point;

use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

const FONT_SIZE: u16 = 28;
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 695;

type TextureCreator2 = TextureCreator<WindowContext>;


lazy_static! {
    static ref FONT_PATH: &'static Path = Path::new("../../resources/SourceCodePro-Semibold.ttf");

    static ref TEXT_COLOR: Color = Color::RGBA(0xFD, 0xF6, 0xE3, 0xFF);
    static ref BG_COLOR: Color = Color::RGBA(0x00, 0x2B, 0x36, 0xFF);
    static ref PANEL_HL_COLOR: Color = Color::RGBA(0x58, 0x6E, 0x75, 0xFF);
    static ref PANEL_SH_COLOR: Color = Color::RGBA(0x00, 0x20, 0x20, 0xFF);
    static ref INSTRUCTION_HL_COLOR: Color = Color::RGBA(0x2a, 0xa1, 0x98, 0xFF);

    static ref SCREEN_TARGET: Rect = Rect::new(24, 24, 1536, 768);
    static ref SCREEN_FRAME: Rect = Rect::new(20, 20, 1540, 772);
    static ref LOG_FRAME: Rect = Rect::new(20, 808, 1540, 563);
    static ref INSTRUCTION_FRAME: Rect = Rect::new(1582, 20, 446, 580);
    static ref REGISTER_FRAME: Rect = Rect::new(1582, 620, 446, 580);
    static ref STATUS_FRAME: Rect = Rect::new(1582, 1220, 446, 150);
}


pub struct GfxSubsystem<'ttf, 'b> {
    font: Font<'ttf, 'b>,
    canvas: Canvas<Window>,
    address: usize,
    screen: Texture<'b>,
}

impl <'ttf, 'b> GfxSubsystem<'ttf, 'b> {

    pub fn new(canvas: Canvas<Window>, ttf_context: &'ttf mut Sdl2TtfContext, texture_creator: &'b TextureCreator2) -> GfxSubsystem<'ttf, 'b> {

        let mut font = ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap();
        font.set_hinting(sdl2::ttf::Hinting::Mono);

        let screen = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
            .unwrap();

        GfxSubsystem {
            font: font,
            canvas: canvas,
            address: 512,
            screen: screen,
        }
    }

    fn redraw(&mut self, vm: &mut Chip8) {

        self.canvas.set_draw_color(*BG_COLOR);
        self.canvas.clear();

        self.draw_screen(vm);
        self.draw_log();
        self.draw_instructions(vm);
        self.draw_registers(vm);
        self.draw_status(vm);

        self.canvas.present();
    }

    fn draw_panel(&mut self, rect: Rect) {

        let ul = Point::new(rect.left(), rect.top());
        let ur = Point::new(rect.right(), rect.top());
        let ll = Point::new(rect.left(), rect.bottom());
        let lr = Point::new(rect.right(), rect.bottom());

        self.canvas.set_draw_color(*PANEL_HL_COLOR);
        self.canvas.draw_line(ll, lr).unwrap();
        self.canvas.draw_line(ur, lr).unwrap();

        self.canvas.set_draw_color(*PANEL_SH_COLOR);
        self.canvas.draw_line(ul, ur).unwrap();
        self.canvas.draw_line(ul, ll).unwrap();
    }

    fn draw_screen(&mut self, vm: &mut Chip8) {
        self.draw_panel(*SCREEN_FRAME);

        let vm_pitch = vm.video.len() / 32;

        self.screen.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for i in 0..vm.video.len() {
                let vm_byte = vm.video[i];
                let y = i / vm_pitch;
                let x_byte = i % vm_pitch;

                for bit_position in 0..8 {
                    let x = x_byte * 8 + bit_position;
                    let val = vm_byte & (1 << bit_position);

                    let offset = y*pitch + x*3;

                    if val == 0 {
                        buffer[offset] = 0x8F;
                        buffer[offset + 1] = 0x91;
                        buffer[offset + 2] = 0x85;
                    } else {
                        buffer[offset] = 0x11;
                        buffer[offset + 1] = 0x13;
                        buffer[offset + 2] = 0x2B;

                    }
                }
            }
        }).unwrap();

        self.canvas.copy(&self.screen, None, Some(*SCREEN_TARGET)).unwrap();
    }

    fn draw_log(&mut self) {
        let panel = *LOG_FRAME;
        self.draw_panel(panel);

        let x = panel.left() + 5;
        let y = panel.top() + 3;

        let text = "HERE IS A CONSOLE WITH SOME TEXT IN IT";
        self.draw_text(&text, x, y, *TEXT_COLOR);
    }

    fn draw_status(&mut self, vm: &mut Chip8) {
        let panel = *STATUS_FRAME;
        let spacing = self.font.recommended_line_spacing();
        self.draw_panel(panel);

        let x = panel.left() + 5;
        let mut y = panel.top() + 3;

        let text = " VM HZ = ??";
        self.draw_text(&text, x, y, *TEXT_COLOR);
        y += spacing;

        let text = "VID HZ = ??";
        self.draw_text(&text, x, y, *TEXT_COLOR);
        y += spacing;

        let text = format!("  STEP = {}", vm.step);
        self.draw_text(&text, x, y, *TEXT_COLOR);
        y += spacing;

        let text = "    ?? = ??";
        self.draw_text(&text, x, y, *TEXT_COLOR);

    }

    fn draw_instructions(&mut self, vm: &mut Chip8) {
        let panel = *INSTRUCTION_FRAME;
        let spacing = self.font.recommended_line_spacing();
        let hihglight_width = panel.width() - 10;
        let x = panel.left() + 5;
        let mut y = panel.top();

        self.draw_panel(panel);

        if vm.pc < self.address || vm.pc - self.address > 30 || vm.pc - self.address < 4 {
            self.address = vm.pc - 4;
        }

        for offset in 0..16 {
            let addr = self.address + offset * 2;

            if vm.pc == addr {
                self.canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                self.canvas.fill_rect(Rect::new(x - 1, y + 1, hihglight_width, spacing as u32)).unwrap();
            }

            match vm.fetch_instruction(addr) {
                Ok(instruction) => {
                    match vm.disassemble(instruction) {
                        Ok(text) => {
                            let text = format!("{:04X}: {}", addr, text);
                            self.draw_text(&text, x, y, *TEXT_COLOR);
                        },
                        Err(err) => self.draw_text(err.description(), x, y, *TEXT_COLOR),
                    }
                },
                Err(err) => {
                    self.draw_text(err.description(), x, y, *TEXT_COLOR)
                }
            };

            y += spacing;
        }
    }

    fn draw_registers(&mut self, vm: &mut Chip8) {

        let panel = *REGISTER_FRAME;
        let spacing = self.font.recommended_line_spacing();
        self.draw_panel(panel);

        let mut x = panel.left() + 5;
        let mut y = panel.top() + 3;

        for i in 0..0x10 {
            let line = format!("V{:X} = {:2X}", i, vm.v[i]);
            self.draw_text(&line, x, y, *TEXT_COLOR);
            y += spacing;
        }

        let pc = vm.pc;
        let sp = vm.sp;
        let dt = vm.dt;
        let st = vm.st;
        let i = vm.i;

        x = panel.left() + 240; // TODO hidpi
        y = panel.top() + 3;

        self.draw_text(&format!("PC = {:04X}", pc), x, y, *TEXT_COLOR);
        y += spacing;

        self.draw_text(&format!("SP = {}", sp), x, y, *TEXT_COLOR);
        y += spacing;

        self.draw_text(&format!("DT = {:04X}", dt), x, y, *TEXT_COLOR);
        y += spacing;

        self.draw_text(&format!("ST = {:04X}", st), x, y, *TEXT_COLOR);

        y += spacing * 2;
        self.draw_text(&format!(" I = {:04X}", i), x, y, *TEXT_COLOR);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: Color) {

        let surface = self.font.render(text).blended(color).unwrap();
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
        let target = Rect::new(x, y, surface.width(), surface.height());

        self.canvas.copy(&texture, None, Some(target)).unwrap();
    }
}

struct System<'a, 'b> {
    event_pump: sdl2::EventPump,
    paused: bool,
    vm_error: Option<Chip8Error>,
    vm: Chip8,
    gfx: GfxSubsystem<'a, 'b>,
}

impl <'a, 'b> System<'a, 'b> {

    fn new(gfx: GfxSubsystem<'a, 'b>, event_pump: sdl2::EventPump) -> System<'a, 'b> {

        /*
        let path = Path::new("roms/BRIX");
        let mut file = match File::open(&path) {
            Err(err) => panic!("couldn't open {}: {}", path.display(), err.description()),
            Ok(file) => file,
        };
        */
        let mut boot: &[u8] = &[
            0xA2, 0x5B, 0x60, 0x0B, 0x61, 0x03, 0x62, 0x07,
            0xD0, 0x17, 0x70, 0x07, 0xF2, 0x1E, 0xD0, 0x17,
            0x70, 0x07, 0xF2, 0x1E, 0xD0, 0x17, 0x70, 0x07,
            0xF2, 0x1E, 0xD0, 0x17, 0x70, 0x07, 0xF2, 0x1E,
            0xD0, 0x17, 0x70, 0x05, 0xF2, 0x1E, 0xD0, 0x17,
            0xF2, 0x1E, 0xA2, 0x5A, 0xC0, 0x3F, 0xC1, 0x1F,
            0x62, 0x01, 0x63, 0x01, 0xD0, 0x11, 0x64, 0x02,
            0xF4, 0x15, 0xF4, 0x07, 0x34, 0x00, 0x12, 0x3A,
            0xD0, 0x11, 0x80, 0x24, 0x81, 0x34, 0xD0, 0x11,
            0x41, 0x00, 0x63, 0x01, 0x41, 0x1F, 0x63, 0xFF,
            0x40, 0x00, 0x62, 0x01, 0x40, 0x3F, 0x62, 0xFF,
            0x12, 0x36, 0x80, 0x78, 0xCC, 0xC0, 0xC0, 0xC0,
            0xCC, 0x78, 0xCC, 0xCC, 0xCC, 0xFC, 0xCC, 0xCC,
            0xCC, 0xFC, 0x30, 0x30, 0x30, 0x30, 0x30, 0xFC,
            0xF8, 0xCC, 0xCC, 0xF8, 0xC0, 0xC0, 0xC0, 0x00,
            0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x78, 0xCC,
            0xCC, 0x78, 0xCC, 0xCC, 0x78,
        ];
        let mut vm = Chip8::new();
        //let mut buf = File::create("DEMO").unwrap();
        //buf.write_all(boot);

        //match vm.load_rom(&mut file) {
        vm.load_rom(&mut boot).unwrap();
        /*
        match vm.load_rom(&mut boot) {
            Err(err) => panic!("couldn't read {}: {}", path.display(), err.description()),
            Ok(_) => (),
        }
        */

        System {
            event_pump: event_pump,
            paused: false,
            vm_error: None,
            vm: vm,
            gfx: gfx,
        }
    }

    fn run(&mut self) {

        let (render_tx, render_rx) = channel();
        let (vm_tx, vm_rx) = channel();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(16));
                render_tx.send("x").unwrap();
            }
        });

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(2));
                vm_tx.send("x").unwrap();
            }
        });

        'running: loop {

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                    Event::KeyDown { keycode: Some(Keycode::F5), .. } => self.paused = !self.paused,
                    Event::KeyDown { keycode: Some(Keycode::F6), .. } => {
                        match self.vm.step() {
                            Ok(_) => (),
                            Err(err) => {
                                self.vm_error = Some(err);
                                self.paused = true;
                            }
                        }
                    },
                    _ => ()
                }
            }

            select! {
                _ = vm_rx.recv() => {

                    if !self.paused {
                        if let Err(_) = self.vm.step() {
                            self.paused = true;
                        }
                    }
                },
                _ = render_rx.recv() => {
                    self.gfx.redraw(&mut self.vm);
                }
            }
        }
    }
}

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
