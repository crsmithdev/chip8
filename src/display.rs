extern crate sdl2_sys;

use cache::TextureCache;
use cpu::Chip8;
use vm::VMState2;

use logger::Logger;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use sdl2::Sdl;
use sdl2_sys::SDL_WindowFlags;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 576;
const FONT_SIZE: u16 = 28;
const FONT_SPACING: i32 = 43;
const N_MESSAGES: usize = 7;
const N_INSTRUCTIONS: usize = 25;

type ContextRef<'a> = Rc<Context<'a>>;

macro_rules! text {
    ($canvas:ident, $writer:ident { $($style:tt @ $x:expr, $y:expr => $text:expr)* } ) => ({
        $({
            let texture = $writer.write(&$text, $style);
            let query = texture.query();
            let rect = Rect::new($x, $y, query.width, query.height);
            $canvas.copy(&texture, None, rect).unwrap();
        });*

    });
}

macro_rules! text2 {
    ($context:ident { $($style:expr => $x:expr, $y:expr => $text:expr)* } ) => ({
        $({
            let mut canvas = $context.canvas.borrow_mut();
            let writer = $context.writer.clone();
            let texture = writer.write(&$text, $style);
            let query = texture.query();
            let rect = Rect::new($x, $y, query.width, query.height);
            canvas.copy(&texture, None, rect).unwrap();
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
    static ref HIGHLIGHT_COLOR: Color = Color::RGBA(27, 31, 43, 255);
    static ref SCREEN_OFF_COLOR: Color = Color::RGBA(0x8F, 0x91, 0x85, 0xFF);
    static ref SCREEN_ON_COLOR: Color = Color::RGBA(0x11, 0x13, 0x2B, 0xFF);
    static ref SCREEN_TARGET: Rect = Rect::new(40, 40, 1470, 735);
    static ref SCREEN_FRAME: Rect = Rect::new(20, 20, 1510, 775);
    static ref LOG_FRAME: Rect = Rect::new(20, 815, 700, 317);
    static ref INSTRUCTION_FRAME: Rect = Rect::new(1550, 20, 478, 1112);
    static ref REGISTER_FRAME: Rect = Rect::new(740, 815, 790, 317);
}

pub struct Screen {}

impl Screen {
    fn new() -> Screen {
        Screen {}
    }
}

impl Renderable for Screen {
    fn rect(&self) -> Rect {
        *SCREEN_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &VMState2) {
        let mut canvas = context.canvas.borrow_mut();
        let screen = context
            .cache
            .get_mut_or_else("screen", || {
                canvas
                    .texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
                    .unwrap()
            })
            .unwrap();

        screen
            .with_lock(None, |buffer: &mut [u8], _: usize| {
                let video = state.cpu.video();
                for byte_offset in 0..video.len() {
                    let byte = video[byte_offset];

                    for bit_offset in 0..8 {
                        let i = (byte_offset * 8 * 3) + (bit_offset * 3);
                        let color = match byte & (1 << (7 - bit_offset)) {
                            0 => *SCREEN_OFF_COLOR,
                            _ => *SCREEN_ON_COLOR,
                        };

                        buffer[i] = color.r;
                        buffer[i + 1] = color.g;
                        buffer[i + 2] = color.b;
                    }
                }
            })
            .unwrap();

        canvas.copy(&screen, None, Some(*SCREEN_TARGET)).unwrap();
    }
}

pub struct Instructions {
    offset: usize,
}

impl Instructions {
    fn new() -> Instructions {
        Instructions { offset: 512 }
    }
}

impl Renderable for Instructions {
    fn rect(&self) -> Rect {
        *INSTRUCTION_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &VMState2) {
        let rect = self.rect();
        let cpu = &state.cpu;
        let x = rect.left() + 20;
        let mut y = rect.top() + 20;

        if cpu.pc < self.offset + 4 || cpu.pc > self.offset + N_INSTRUCTIONS * 2 - 4 {
            self.offset = cpu.pc - 4;
        }

        for i in 0..N_INSTRUCTIONS {
            let address = self.offset + i * 2;

            if cpu.pc == address {
                let width = rect.width() - 20;
                let rect = Rect::new(x - 10, y - 3, width, FONT_SPACING as u32);
                let mut canvas = context.canvas.borrow_mut();
                canvas.set_draw_color(*HIGHLIGHT_COLOR);
                canvas.fill_rect(rect).unwrap();
            }

            let inst = cpu.fetch(address as u16);
            let result = Chip8::disassemble(inst);

            text2!(context {
                Style::Address     => x,       y => format!("{:04X}", address)
                Style::Instruction => x + 85,  y => format!("{:04X}", inst)
                Style::Default     => x + 170, y => result.operation
                Style::Default     => x + 280, y => result.params
            });

            y += FONT_SPACING;
        }
    }
}

pub struct Registers {}

impl Registers {
    fn new() -> Registers {
        Registers {}
    }
}

impl Renderable for Registers {
    fn rect(&self) -> Rect {
        *REGISTER_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &VMState2) {
        let mut canvas = context.canvas.borrow_mut();
        let writer = context.writer.clone();
        let rect = self.rect();
        let default = Style::Default;
        let address = Style::Address;
        let instruction = Style::Instruction;
        let spacing = FONT_SPACING;
        let vm = &state.cpu;

        let mut x = rect.left() + 20;
        let separator = Rect::new(rect.left() + 20, rect.top() + 110, rect.width() - 40, 5);
        canvas.set_draw_color(*BG_COLOR);
        canvas.fill_rect(separator).unwrap();

        for col in 0..4 {
            let mut y = rect.top() + 135;
            for row in 0..4 {
                let i = col * 4 + row;
                let v = vm.v[i];
                text!(canvas, writer {
                    default     @ x,       y => format!("V{:X}", i)
                    address     @ x + 60,  y => format!("{:02X}", v)
                    instruction @ x + 100, y => format!("({})", v)
                });
                y += spacing;
            }
            x += 200;
        }

        let x = rect.left() + 20;
        let y = rect.top() + 10;

        text!(canvas, writer {
            default @ x,       y      => "PC"
            address @ x + 60,  y      => format!("{:04X}", vm.pc)
            default @ x + 200, y      => "ST"
            address @ x + 260, y      => format!("{:02X}", vm.st)
            default @ x + 400, y      => "DT"
            address @ x + 460, y      => format!("{:02X}", vm.dt)
            default @ x + 600, y      => "SP"
            address @ x + 660, y      => format!("{:02X}", vm.sp)
            default @ x,       y + 40 => "I"
            address @ x + 60,  y + 40 => format!("{:04X }", vm.i)
            default @ x + 200, y + 40 => "HZ"
            address @ x + 260, y + 40 => format!("{:04}", state.hz)
            default @ x + 400, y + 40 => "FPS"
            address @ x + 460, y + 40 => format!("{:02}", state.fps)
        });
    }
}

pub struct Log {
    messages: VecDeque<String>,
}

impl Log {
    fn new() -> Log {
        Log {
            messages: VecDeque::new(),
        }
    }
}

impl Renderable for Log {
    fn rect(&self) -> Rect {
        *LOG_FRAME
    }

    fn render(&mut self, context: ContextRef, _state: &VMState2) {
        let mut canvas = context.canvas.borrow_mut();
        let writer = context.writer.clone();
        //let font = context.fonts.default();
        let font = Style::Default;
        let rect = self.rect();
        if context.log.unread() > 0 {
            let read = context.log.read();
            self.messages.extend(read);
            while self.messages.len() > N_MESSAGES {
                self.messages.pop_front();
            }
        }

        let mut y = rect.top() + 10;
        for message in &self.messages {
            text!(canvas, writer {
                font @ rect.left() + 20, y => message
            });
            y += FONT_SPACING;
        }
    }
}

trait Renderable {
    fn render(&mut self, context: ContextRef, state: &VMState2);
    fn rect(&self) -> Rect;
}

struct Panel(Box<Renderable>);

impl Renderable for Panel {
    fn render(&mut self, context: ContextRef, state: &VMState2) {
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

struct FontWriter<'a> {
    creator: TextureCreator<WindowContext>,
    cache: &'a TextureCache,
    font: Font<'a, 'static>,
}

impl<'a> FontWriter<'a> {
    fn write(&self, text: &str, style: Style) -> &'a Texture {
        let key = format!("{}|{}", text, style);
        match self.cache.get(&key) {
            Some(ref t) => t,
            None => {
                let color = self.color(style);
                let texture = self.build(&text, &self.font, color);
                self.cache.put(&key, texture);
                self.cache.get(&key).unwrap()
            }
        }
    }

    fn build(&self, text: &str, font: &Font<'_, '_>, color: Color) -> Texture {
        let text = if text == "" { " " } else { text };
        let surface = font.render(text).blended(color).unwrap();
        let texture = self.creator.create_texture_from_surface(&surface).unwrap();
        texture
    }
    fn color(&self, style: Style) -> Color {
        match style {
            Style::Default => *TEXT_COLOR,
            Style::Address => *ADDR_COLOR,
            Style::Instruction => *INST_COLOR,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Style {
    Default,
    Address,
    Instruction,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Context<'a> {
    cache: &'a TextureCache,
    writer: Rc<FontWriter<'a>>,
    canvas: Rc<RefCell<Canvas<Window>>>,
    log: &'static Logger,
}

pub struct Display<'a> {
    context: ContextRef<'a>,
    panels: Vec<Panel>,
}

impl<'a> Display<'a> {
    pub fn new(
        sdl_context: &'a Sdl,
        ttf_context: &'a Sdl2TtfContext,
        log: &'static Logger,
        cache: &'a TextureCache,
    ) -> Display<'a> {
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

        let texture_creator = canvas.texture_creator();
        let canvas = Rc::new(RefCell::new(canvas));

        let panels = vec![
            panel!(Instructions::new()),
            panel!(Screen::new()),
            panel!(Registers::new()),
            panel!(Log::new()),
        ];

        let font = ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap();

        let writer2 = Rc::new(FontWriter {
            creator: texture_creator,
            font: font,
            cache: cache,
        });

        let context = Rc::new(Context {
            cache: cache,
            log: log,
            canvas: canvas.clone(),
            writer: writer2,
        });

        Display {
            context: context.clone(),
            panels: panels,
        }
    }

    pub fn update(&mut self, state: &VMState2) {
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

    pub fn focused(&self) -> bool {
        let canvas = self.context.canvas.borrow();
        let flags = canvas.window().window_flags() as u32;
        let focused = SDL_WindowFlags::SDL_WINDOW_INPUT_FOCUS as u32;
        (flags & focused) == focused
    }
}
