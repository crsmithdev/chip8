extern crate sdl2;

use cpu::Chip8;
use sdl2::pixels::PixelFormatEnum;
use std::path::Path;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::{Texture, TextureCreator};
use sdl2::ttf::Font;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use cpu::Chip8State;
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 576;
const FONT_SIZE: u16 = 28;

type ContextRef<'a> = Rc<Context<'a>>;

lazy_static! {
    static ref FONT_PATH: &'static Path = Path::new("../../resources/SourceCodePro-Semibold.ttf");
    static ref TEXT_COLOR: Color = Color::RGBA(166, 172, 205, 0xFF);
    static ref ADDR_COLOR: Color = Color::RGBA(130, 170, 255, 255);
    static ref INST_COLOR: Color = Color::RGBA(198, 146, 233, 255);
    static ref BG_COLOR: Color = Color::RGBA(27, 31, 43, 0xFF);
    static ref PANEL_COLOR: Color = Color::RGBA(42, 46, 62, 0xFF);
    static ref INSTRUCTION_HL_COLOR: Color = Color::RGBA(27, 31, 43, 255);
    static ref SCREEN_PX_OFF_COLOR: Color = Color::RGBA(0x8F, 0x91, 0x85, 0xFF);
    static ref SCREEN_PX_ON_COLOR: Color = Color::RGBA(0x11, 0x13, 0x2B, 0xFF);
    static ref SCREEN_TARGET: Rect = Rect::new(40, 40, 1470, 735);
    static ref SCREEN_FRAME: Rect = Rect::new(20, 20, 1510, 775);
    static ref LOG_FRAME: Rect = Rect::new(20, 815, 700, 317);
    static ref INSTRUCTION_FRAME: Rect = Rect::new(1550, 20, 478, 1112);
    static ref REGISTER_FRAME: Rect = Rect::new(740, 815, 790, 317);
}

pub struct Screen<'a> {
    context: ContextRef<'a>,
}

impl<'a> Renderable for Screen<'a> {
    type State = Chip8State;

    fn render(&mut self, state: &Chip8State) {
        let frame = *SCREEN_FRAME;
        //let ctx = self.context;//.borrow_mut();
        let mut canvas = self.context.canvas.borrow_mut();
        //{
        //let mut canvas = self.canvas.borrow_mut();
        canvas.set_draw_color(*PANEL_COLOR);
        canvas.fill_rect(frame).unwrap();
        //}
        //let mut canvas = self.canvas.borrow_mut();
        let texture_creator = canvas.texture_creator();
        let mut screen = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
            .unwrap();

        screen
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                for byte_offset in 0..state.video.len() {
                    let vm_byte = state.video[byte_offset];

                    for bit_offset in 0..8 {
                        let buf_idx = (byte_offset * 8 * 3) + (bit_offset * 3);

                        match vm_byte & (1 << (7 - bit_offset)) {
                            0 => {
                                buffer[buf_idx] = SCREEN_PX_OFF_COLOR.r;
                                buffer[buf_idx + 1] = SCREEN_PX_OFF_COLOR.g;
                                buffer[buf_idx + 2] = SCREEN_PX_OFF_COLOR.b;
                            }
                            _ => {
                                buffer[buf_idx] = SCREEN_PX_ON_COLOR.r;
                                buffer[buf_idx + 1] = SCREEN_PX_ON_COLOR.g;
                                buffer[buf_idx + 2] = SCREEN_PX_ON_COLOR.b;
                            }
                        }
                    }
                }
            })
            .unwrap();

        canvas.copy(&screen, None, Some(*SCREEN_TARGET)).unwrap();
    }
}

pub struct Instructions<'a> {
    address: usize,
    context: ContextRef<'a>,
}

impl<'a> Renderable for Instructions<'a> {
    type State = Chip8State;

    fn render(&mut self, state: &Chip8State) {
        let frame = *INSTRUCTION_FRAME;
        //let ctx = self.context;//.borrow_mut();
        let mut canvas = self.context.canvas.borrow_mut();

        canvas.set_draw_color(*PANEL_COLOR);
        canvas.fill_rect(frame).unwrap();

        let hihglight_width = frame.width() - 25;
        let x = frame.left() + 15;
        let mut y = frame.top() + 10;

        if state.pc < self.address || state.pc - self.address > 30 || state.pc - self.address < 4 {
            self.address = state.pc - 4;
        }

        let spacing = ((self.context.font_cache.spacing() as f32) * 1.19) as i32;
        for offset in 0..26 {
            let addr = self.address + offset * 2;
            let highlighted = state.pc == addr;

            if highlighted {
                canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                canvas
                    .fill_rect(Rect::new(x - 4, y - 3, hihglight_width, spacing as u32))
                    .unwrap();
            }
            let mut c = canvas.deref_mut();
            let result = Chip8::disassemble2(addr, state.memory);
            let mut text = self.context.font_cache.build(
                &format!("{:04X}", result.address),
                x,
                y,
                *ADDR_COLOR,
            );
            self.context.font_cache.copy(text, c);

            let b2 = result.instruction as u8;
            let b1 = (result.instruction >> 8) as u8;
            let b1_text = format!("{:02X}", b1);
            let b2_text = format!("{:02X}", b2);
            let text = self
                .context
                .font_cache
                .build(&b1_text, x + 90, y, *INST_COLOR);
            self.context.font_cache.copy(text, c);

            let text = self
                .context
                .font_cache
                .build(&b2_text, x + 130, y, *INST_COLOR);
            self.context.font_cache.copy(text, c);

            let text = self
                .context
                .font_cache
                .build(&result.operation, x + 190, y, *TEXT_COLOR);
            self.context.font_cache.copy(text, c);

            if result.params != "" {
                //self.writer.write(&result.params, x + 310, y, *TEXT_COLOR);
                let text = self
                    .context
                    .font_cache
                    .build(&result.params, x + 310, y, *TEXT_COLOR);
                self.context.font_cache.copy(text, c);
            }

            y += spacing;
        }
    }
}

pub struct Registers<'a> {
    context: ContextRef<'a>,
}

impl<'a> Renderable for Registers<'a> {
    type State = Chip8State;

    fn render(&mut self, state: &Chip8State) {
        let frame = *REGISTER_FRAME;
        //let context = self.context;//.borrow_mut();
        let mut canvas = self.context.canvas.borrow_mut();
        {
            canvas.set_draw_color(*PANEL_COLOR);
            canvas.fill_rect(frame).unwrap();
        }

        let spacing = ((self.context.font_cache.spacing() as f32) * 1.2) as i32;
        let mut x = frame.left() + 15;

        let c = canvas.deref_mut();
        for col in 0..4 {
            let mut y = frame.top() + 10;
            for row in 0..4 {
                let i = col * 4 + row;
                let v = state.v[i];
                let mut text =
                    self.context
                        .font_cache
                        .build(&format!("V{:X}", i), x, y, *TEXT_COLOR);
                self.context.font_cache.copy(text, c);

                let text =
                    self.context
                        .font_cache
                        .build(&format!("{:02X}", v), x + 60, y, *ADDR_COLOR);
                self.context.font_cache.copy(text, c);

                let text =
                    self.context
                        .font_cache
                        .build(&format!("({})", v), x + 100, y, *INST_COLOR);
                self.context.font_cache.copy(text, c);
                y += spacing;
            }
            x += 200;
        }

        x = frame.left() + 15;
        let mut y = frame.top() + 200;

        let mut text =
            self.context
                .font_cache
                .build(&format!("PC {:04X}", state.pc), x, y, *TEXT_COLOR);
        self.context.font_cache.copy(text, c);
        x += 200;

        text = self
            .context
            .font_cache
            .build(&format!("S {}", state.sp), x, y, *TEXT_COLOR);
        self.context.font_cache.copy(text, c);
        x += 200;

        text = self
            .context
            .font_cache
            .build(&format!("DT {:04X}", state.dt), x, y, *TEXT_COLOR);
        self.context.font_cache.copy(text, c);
        x += 200;

        text = self
            .context
            .font_cache
            .build(&format!("ST {:04X}", state.st), x, y, *TEXT_COLOR);
        self.context.font_cache.copy(text, c);

        x = frame.left() + 15;
        y += 50;

        text = self
            .context
            .font_cache
            .build(&format!(" I {:04X}", state.i), x, y, *TEXT_COLOR);
        self.context.font_cache.copy(text, c);
    }
}

pub struct Log<'a> {
    context: ContextRef<'a>,
}

impl<'a> Renderable for Log<'a> {
    type State = Chip8State;

    fn render(&mut self, _state: &Chip8State) {
        let frame = *LOG_FRAME;
        //let mut context = self.context;//.borrow_mut();
        let mut canvas = self.context.canvas.borrow_mut();
        {
            canvas.set_draw_color(*PANEL_COLOR);
            canvas.fill_rect(frame).unwrap();
        }

        let x = frame.left() + 5;
        let y = frame.top() + 3;

        let text =
            self.context
                .font_cache
                .build(&"This is a log message with data", x, y, *TEXT_COLOR);
        let c = canvas.deref_mut();
        self.context.font_cache.copy(text, c);
    }
}

pub trait Renderable {
    type State;

    fn render(&mut self, state: &Self::State);
}

struct Text<'a> {
    texture: Texture<'a>,
    rect: Rect,
}

struct FontCache<'a> {
    texture_creator: Box<TextureCreator<WindowContext>>,
    font: Font<'a, 'static>,
}

impl<'a> FontCache<'a> {
    fn build(&self, text: &str, x: i32, y: i32, color: Color) -> Text {
        let surface = self.font.render(text).blended(color).unwrap();
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        Text {
            texture: texture,
            rect: Rect::new(x, y, surface.width(), surface.height()),
        }
    }

    fn copy(&self, text: Text, canvas: &mut Canvas<Window>) {
        canvas.copy(&text.texture, None, Some(text.rect)).unwrap();
    }

    fn spacing(&self) -> i32 {
        self.font.recommended_line_spacing()
    }
}

struct Context<'a> {
    font_cache: FontCache<'a>,
    canvas: Rc<RefCell<Canvas<Window>>>,
}

pub struct Display<'a> {
    context: ContextRef<'a>,
    screen: Screen<'a>,
    instructions: Instructions<'a>,
    registers: Registers<'a>,
    log: Log<'a>,
}

impl<'a> Renderable for Display<'a> {
    type State = Chip8State;

    fn render(&mut self, state: &Chip8State) {
        {
            //let mut context = self.context;//.borrow_mut();
            let mut canvas = self.context.canvas.borrow_mut();
            canvas.set_draw_color(*BG_COLOR);
            canvas.clear();
        }

        self.instructions.render(state);
        self.registers.render(state);
        self.log.render(state);
        self.screen.render(state);

        {
            //let mut context = self.context;//.borrow_mut();
            let mut canvas = self.context.canvas.borrow_mut();
            canvas.present();
        }
    }
}

impl<'a> Display<'a> {
    pub fn new(sdl_context: &'a Sdl, ttf_context: &'a Sdl2TtfContext) -> Display<'a> {
        let window = sdl_context
            .video()
            .unwrap()
            .window("CHIP-8", WINDOW_WIDTH, WINDOW_HEIGHT)
            .allow_highdpi()
            .build()
            .unwrap();

        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();
        let tc = Box::new(canvas.texture_creator());
        let canvas = Rc::new(RefCell::new(canvas));

        let context = Rc::new(Context {
            canvas: canvas.clone(),
            font_cache: FontCache {
                texture_creator: tc,
                font: ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap(),
            },
        });

        let screen = Screen {
            context: context.clone(),
        };

        let instructions = Instructions {
            address: 512,
            context: context.clone(),
        };

        let registers = Registers {
            context: context.clone(),
        };

        let log = Log {
            context: context.clone(),
        };

        Display {
            context: context.clone(),
            screen: screen,
            instructions: instructions,
            registers: registers,
            log: log,
        }
    }
}
