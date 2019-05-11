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
use std::rc::Rc;

use cpu::Chip8State;
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 576;
const FONT_SIZE: u16 = 28;
const FONT_SPACING: i32 = 43;

type ContextRef<'a> = Rc<Context<'a>>;

macro_rules! text1 {
    ($canvas:ident { $($face:ident @ $x:expr, $y:expr => $text:expr)* } ) => ({
        $({
            //let text = format!($($arg)*);
            let texture = $face(&$text);
            let query = texture.query();
            let rect = Rect::new($x, $y, query.width, query.height);
            $canvas.copy(&texture, None, rect).unwrap();
        });*

    });
}

macro_rules! panel {
    ($contents:expr) => {
        Panel(Box::new($contents))
    };
}

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

pub struct Screen {}

impl Renderable for Screen {
    fn rect(&self) -> Rect {
        *SCREEN_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &Chip8State) {
        let mut canvas = context.canvas.borrow_mut();

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

pub struct Instructions {
    address: usize,
}

impl Renderable for Instructions {
    fn rect(&self) -> Rect {
        *INSTRUCTION_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &Chip8State) {
        let mut canvas = context.canvas.borrow_mut();
        let rect = self.rect();
        let fonts = &context.fonts;
        let default = fonts.default();
        let address = fonts.address();
        let instruction = fonts.instruction();

        let hl_width = rect.width() - 25;
        let x = rect.left() + 25;
        let mut y = rect.top() + 20;

        if state.pc < self.address || state.pc - self.address > 30 || state.pc - self.address < 4 {
            self.address = state.pc - 4;
        }

        for offset in 0..25 {
            let addr = self.address + offset * 2;
            let highlighted = state.pc == addr;

            if highlighted {
                let rect = Rect::new(x - 4, y - 3, hl_width, FONT_SPACING as u32);
                canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                canvas.fill_rect(rect).unwrap();
            }

            let result = Chip8::disassemble2(addr, state.memory);
            let low = result.instruction as u8;
            let high = (result.instruction >> 8) as u8;

            text1!(canvas {
                address @ x, y => format!("{:04X}", result.address)
                instruction @ x + 90, y => format!("{:02X}", high)
                instruction @ x + 130, y => format!("{:02X}", low)
                default @ x + 190, y => result.operation
                default @ x + 310, y => result.params
            });
            y += FONT_SPACING;
        }
    }
}

pub struct Registers {}

impl Renderable for Registers {
    fn rect(&self) -> Rect {
        *REGISTER_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &Chip8State) {
        let mut canvas = context.canvas.borrow_mut();
        let rect = self.rect();
        let fonts = &context.fonts;
        let default = fonts.default();
        let address = fonts.address();
        let instruction = fonts.instruction();

        let mut x = rect.left() + 20;
        for col in 0..4 {
            let mut y = rect.top() + 20;
            for row in 0..4 {
                let i = col * 4 + row;
                let v = state.v[i];
                text1!(canvas {
                    default     @ x,       y => format!("V{:X}", i)
                    address     @ x + 60,  y => format!("{:02X}", v)
                    instruction @ x + 100, y => format!("({})", v)
                });
                y += FONT_SPACING;
            }
            x += 200;
        }

        let x = rect.left();
        let y = rect.top();

        text1!(canvas {
            default @ x + 20,  y + 200 => format!("PC {:04X}", state.pc)
            default @ x + 220, y + 200 => format!("S {}", state.sp)
            default @ x + 420, y + 200 => format!("DT {:04X}", state.dt)
            default @ x + 620, y + 200 => format!("ST {:04X}", state.st)
            default @ x + 20,  y + 250 => format!("ST {:04X}", state.st)
        });
    }
}

pub struct Log {}

impl Renderable for Log {
    fn rect(&self) -> Rect {
        *LOG_FRAME
    }

    fn render(&mut self, context: ContextRef, _state: &Chip8State) {
        let mut canvas = context.canvas.borrow_mut();
        let font = context.fonts.default();
        let rect = self.rect();

        text1!(canvas {
            font @ rect.left() + 20, rect.top() + 20 => "Log message"
        });
    }
}

trait Renderable {
    fn render(&mut self, context: ContextRef, state: &Chip8State);
    fn rect(&self) -> Rect;
}

struct Panel(Box<Renderable>);

impl Renderable for Panel {
    fn render(&mut self, context: ContextRef, state: &Chip8State) {
        {
            let mut canvas = context.canvas.borrow_mut();
            canvas.set_draw_color(*PANEL_COLOR);
            canvas.fill_rect(self.0.rect()).unwrap();
        }

        self.0.render(context, state);
    }

    fn rect(&self) -> Rect {
        self.0.rect()
    }
}

struct Fonts<'a> {
    texture_creator: Box<TextureCreator<WindowContext>>,
    font: Font<'a, 'static>,
}

impl<'a> Fonts<'a> {
    fn default(&'a self) -> impl Fn(&str) -> Texture<'a> {
        move |s: &str| self.build_texture(s, *TEXT_COLOR)
    }

    fn address(&'a self) -> impl Fn(&str) -> Texture<'a> {
        move |s: &str| self.build_texture(s, *ADDR_COLOR)
    }

    fn instruction(&'a self) -> impl Fn(&str) -> Texture<'a> {
        move |s: &str| self.build_texture(s, *INST_COLOR)
    }

    fn build_texture(&self, text: &str, color: Color) -> Texture {
        let text = if text == "" { " " } else { text };
        let surface = self.font.render(text).blended(color).unwrap();
        let texture = self
            .texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        texture
    }
}

struct Context<'a> {
    fonts: Fonts<'a>,
    canvas: Rc<RefCell<Canvas<Window>>>,
}

pub struct Display<'a> {
    context: ContextRef<'a>,
    panels: Vec<Panel>,
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

        let texture_creator = Box::new(canvas.texture_creator());
        let canvas = Rc::new(RefCell::new(canvas));
        let font = ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap();

        let context = Rc::new(Context {
            canvas: canvas.clone(),
            fonts: Fonts {
                texture_creator: texture_creator,
                font: font,
            },
        });

        let panels = vec![
            panel!(Instructions { address: 512 }),
            panel!(Screen {}),
            panel!(Registers {}),
            panel!(Log {}),
        ];

        Display {
            context: context.clone(),
            panels: panels,
        }
    }

    pub fn update(&mut self, state: &Chip8State) {
        let context = &self.context;

        {
            let mut canvas = context.canvas.borrow_mut();
            canvas.set_draw_color(*BG_COLOR);
            canvas.clear();
        }

        for p in &mut self.panels {
            p.render(context.clone(), state);
        }

        let mut canvas = self.context.canvas.borrow_mut();
        canvas.present();
    }
}
