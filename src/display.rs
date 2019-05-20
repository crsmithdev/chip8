extern crate sdl2_sys;

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
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 576;
const FONT_SIZE: u16 = 28;
const FONT_SPACING: i32 = 43;
const N_MESSAGES: usize = 7;

type ContextRef<'a> = Rc<Context<'a>>;

macro_rules! text {
    ($canvas:ident, $writer:ident { $($style:ident @ $x:expr, $y:expr => $text:expr)* } ) => ({
        $({
            let texture = $writer.write(&$text, $style);
            //let texture = $face(&$text);
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

pub struct Screen {
    screen: Texture,
}

impl Screen {
    fn new(screen: Texture) -> Screen {
        Screen { screen: screen }
    }
}

impl Renderable for Screen {
    fn rect(&self) -> Rect {
        *SCREEN_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &VMState2) {
        let mut canvas = context.canvas.borrow_mut();

        self.screen
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                let video = state.cpu.video();
                for byte_offset in 0..video.len() {
                    let vm_byte = video[byte_offset];

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

        canvas
            .copy(&self.screen, None, Some(*SCREEN_TARGET))
            .unwrap();
    }
}

pub struct Instructions {
    address: usize,
}

impl Instructions {
    fn new() -> Instructions {
        Instructions { address: 512 }
    }
}

impl Renderable for Instructions {
    fn rect(&self) -> Rect {
        *INSTRUCTION_FRAME
    }

    fn render(&mut self, context: ContextRef, state: &VMState2) {
        let mut canvas = context.canvas.borrow_mut();
        let mut writer = context.writer.borrow_mut();
        let rect = self.rect();
        let default = Style::Default;
        let address = Style::Address;
        let instruction = Style::Instruction;

        let hl_width = rect.width() - 20;
        let x = rect.left() + 20;
        let mut y = rect.top() + 20;
        let vm = &state.cpu;

        if vm.pc < self.address || vm.pc - self.address > 30 || vm.pc - self.address < 4 {
            self.address = vm.pc - 4;
        }

        for offset in 0..25 {
            let addr = self.address + offset * 2;
            let highlighted = vm.pc == addr;

            if highlighted {
                let rect = Rect::new(x - 10, y - 3, hl_width, FONT_SPACING as u32);
                canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                canvas.fill_rect(rect).unwrap();
            }

            let inst = vm.fetch(addr as u16);
            let result = Chip8::disassemble(inst);

            text!(canvas, writer {
                address @ x, y => format!("{:04X}", addr)
                instruction @ x + 85, y => format!("{:04X}", inst)
                default @ x + 170, y => result.operation
                default @ x + 280, y => result.params
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
        let mut writer = context.writer.borrow_mut();
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
        let mut writer = context.writer.borrow_mut();
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
            println!("{}", message);
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

struct TextureCache {
    cache: HashMap<String, Rc<Texture>>,
    cache2: HashMap<String, Texture>,
}

impl TextureCache {
    fn get(&self, key: String) -> Option<Rc<Texture>> {
        self.cache.get(&key).map(|x| x.clone())
    }

    fn put(&mut self, key: String, value: Rc<Texture>) {
        self.cache.insert(key, value);
    }

    fn put2(&mut self, key: String, value: Texture) {
        self.cache2.insert(key, value);
    }

    fn get2<'a>(&'a mut self, key: String) -> Option<&'a Texture> {
        self.cache2.get(&String::from("text"))
    }
}

struct FontWriter<'a> {
    creator: Box<TextureCreator<WindowContext>>,
    cache: Box<TextureCache>,
    font: Font<'a, 'static>,
}

impl<'a> FontWriter<'a> {
    fn write(&mut self, text: &str, style: Style) -> Rc<Texture> {
        let key = String::from(format!("{}|{}", text, style));
        //let mut cache = self.cache.borrow_mut();

        if let Some(texture) = self.cache.get(key.clone()) {
            return texture.clone();
        }

        let color = self.color(style);
        let texture = self.build(&text, &self.font, color);
        self.cache.put(key.clone(), Rc::new(texture));
        self.cache.get(key).unwrap()
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
    writer: Rc<RefCell<FontWriter<'a>>>,
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
        let screen = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
            .unwrap();
        let canvas = Rc::new(RefCell::new(canvas));

        let panels = vec![
            panel!(Instructions::new()),
            panel!(Screen::new(screen)),
            panel!(Registers::new()),
            panel!(Log::new()),
        ];

        let font = ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap();

        let mut cache = TextureCache {
            cache: HashMap::new(),
            cache2: HashMap::new(),
        };
        let writer2 = FontWriter {
            creator: Box::new(texture_creator),
            font: font,
            cache: Box::new(cache),
        };

        let context = Rc::new(Context {
            log: log,
            canvas: canvas.clone(),
            writer: Rc::new(RefCell::new(writer2)),
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
