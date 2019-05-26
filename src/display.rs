use cache::RefCache;
use cpu::OpCode;
use logger::Logger;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::video::Window;
use sdl2::Sdl;
use sdl2_sys::SDL_WindowFlags;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::path::Path;
use std::rc::Rc;
use vm::UpdateState;

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 576;
const FONT_SIZE: u16 = 28;
const LINE_HEIGHT: i32 = 43;
const N_MESSAGES: usize = 7;
const N_INSTRUCTIONS: usize = 25;

lazy_static! {
    static ref FONT_PATH: &'static Path = Path::new("../../resources/SourceCodePro-Semibold.ttf");
    static ref COLOR_DEFAULT: Color = Color::RGBA(166, 172, 205, 0xFF);
    static ref COLOR_BLUE: Color = Color::RGBA(130, 170, 255, 255);
    static ref COLOR_MAGENTA: Color = Color::RGBA(198, 146, 233, 255);
    static ref COLOR_BG: Color = Color::RGBA(27, 31, 43, 0xFF);
    static ref COLOR_PANEL: Color = Color::RGBA(42, 46, 62, 0xFF);
    static ref COLOR_HIGHLIGHT: Color = Color::RGBA(27, 31, 43, 255);
    static ref COLOR_PX_OFF: Color = Color::RGBA(0x8F, 0x91, 0x85, 0xFF);
    static ref COLOR_PX_ON: Color = Color::RGBA(0x11, 0x13, 0x2B, 0xFF);
    static ref RECT_SCREEN_TARGET: Rect = Rect::new(40, 40, 1470, 735);
    static ref RECT_SCREEN: Rect = Rect::new(20, 20, 1510, 775);
    static ref RECT_LOG: Rect = Rect::new(20, 815, 700, 317);
    static ref RECT_INSTRUCTIONS: Rect = Rect::new(1550, 20, 478, 1112);
    static ref RECT_REGISTERS: Rect = Rect::new(740, 815, 790, 317);
}

pub type TextureCache = RefCache<String, Texture>;
type ContextRef<'a> = Rc<Context<'a>>;

macro_rules! text {
    ($context:ident { $($style:expr => $x:expr, $y:expr => $text:expr)* } ) => ({
        let mut canvas = $context.canvas.borrow_mut();
        $({
            let key = format!("{}|{}", $text, $style);
            let text: String = if $text == "" { " ".to_owned() } else { $text.to_owned() };
            let texture = $context.cache.get(&key).unwrap_or_else(|| {
                let color = $style.color();
                let surface = $context.font.render(&text).blended(color).unwrap();
                let creator = canvas.texture_creator();
                let texture = creator.create_texture_from_surface(&surface).unwrap();
                $context.cache.put(key.clone(), texture);
                $context.cache.get(&key).unwrap()
            });
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

#[derive(Debug)]
pub enum Style {
    Default,
    Address,
    Instruction,
}

impl Style {
    fn color(&self) -> Color {
        match self {
            Style::Default => *COLOR_DEFAULT,
            Style::Address => *COLOR_BLUE,
            Style::Instruction => *COLOR_MAGENTA,
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Context<'a> {
    cache: &'a TextureCache,
    canvas: Rc<RefCell<Canvas<Window>>>,
    log: &'static Logger,
    font: Font<'a, 'static>,
}

trait Component {
    fn rect(&self) -> Rect;

    fn update(&mut self, context: ContextRef, state: &UpdateState);
    fn render(&mut self, context: ContextRef, state: &UpdateState);
}

pub struct Screen {}

impl Screen {
    #[inline(always)]
    fn new() -> Screen {
        Screen {}
    }
}

impl Component for Screen {
    #[inline(always)]
    fn rect(&self) -> Rect {
        *RECT_SCREEN
    }

    fn update(&mut self, _ctx: ContextRef, _state: &UpdateState) {}

    fn render(&mut self, context: ContextRef, state: &UpdateState) {
        let mut canvas = context.canvas.borrow_mut();
        let screen = context
            .cache
            .get_mut(&"screen".to_owned())
            .unwrap_or_else(|| {
                let texture = canvas
                    .texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
                    .unwrap();
                context.cache.put("screen".to_owned(), texture);
                context.cache.get_mut(&"screen".to_owned()).unwrap()
            });

        screen
            .with_lock(None, |buffer: &mut [u8], _: usize| {
                let video = state.cpu.video();
                for byte_offset in 0..video.len() {
                    let byte = video[byte_offset];

                    for bit_offset in 0..8 {
                        let i = (byte_offset * 8 * 3) + (bit_offset * 3);
                        let color = match byte & (1 << (7 - bit_offset)) {
                            0 => *COLOR_PX_OFF,
                            _ => *COLOR_PX_ON,
                        };

                        buffer[i] = color.r;
                        buffer[i + 1] = color.g;
                        buffer[i + 2] = color.b;
                    }
                }
            })
            .unwrap();

        canvas
            .copy(&screen, None, Some(*RECT_SCREEN_TARGET))
            .unwrap();
    }
}

pub struct Instructions {
    offset: usize,
    instructions: [u16; N_INSTRUCTIONS],
    highlighted: usize,
}

impl Instructions {
    #[inline(always)]
    fn new() -> Instructions {
        Instructions {
            offset: 0,
            instructions: [0; N_INSTRUCTIONS],
            highlighted: 0,
        }
    }
}

impl Component for Instructions {
    #[inline(always)]
    fn rect(&self) -> Rect {
        *RECT_INSTRUCTIONS
    }

    fn update(&mut self, _ctx: ContextRef, state: &UpdateState) {
        let cpu = &state.cpu;
        if cpu.pc() < self.offset + 4 || cpu.pc() > self.offset + N_INSTRUCTIONS * 2 - 4 {
            self.offset = cpu.pc() - 4;
        }
        for i in 0..N_INSTRUCTIONS {
            let address = self.offset + i * 2;
            let inst = cpu.fetch(address);
            self.instructions[i] = inst;
            if state.cpu.pc() == address {
                self.highlighted = address;
            };
        }
    }

    fn render(&mut self, context: ContextRef, _state: &UpdateState) {
        let x = self.rect().left() + 20;
        let mut y = self.rect().top() + 20;

        for i in 0..N_INSTRUCTIONS {
            let address = self.offset + i * 2;
            let inst = self.instructions[i];

            if self.highlighted == address {
                let width = self.rect().width() - 20;
                let rect = Rect::new(x - 10, y - 3, width, LINE_HEIGHT as u32);
                let mut canvas = context.canvas.borrow_mut();
                canvas.set_draw_color(*COLOR_HIGHLIGHT);
                canvas.fill_rect(rect).unwrap();
            }

            let (op, params) = OpCode::disassemble(inst);

            text!(context {
                Style::Address     => x,       y => format!("{:04X}", address)
                Style::Instruction => x + 85,  y => format!("{:04X}", inst)
                Style::Default     => x + 170, y => op
                Style::Default     => x + 280, y => params
            });

            y += LINE_HEIGHT;
        }
    }
}

pub struct Registers {
    pub v: [u8; 16],
    pub pc: usize,
    pub sp: usize,
    pub i: usize,
    pub dt: u8,
    pub st: u8,
    pub hz: u32,
    pub fps: i32,
}

impl Registers {
    #[inline(always)]
    fn new() -> Registers {
        Registers {
            pc: 0,
            sp: 0,
            v: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            hz: 0,
            fps: 0,
        }
    }
}

impl Component for Registers {
    #[inline(always)]
    fn rect(&self) -> Rect {
        *RECT_REGISTERS
    }

    fn update(&mut self, _ctx: ContextRef, state: &UpdateState) {
        self.pc = state.cpu.pc();
        self.sp = state.cpu.sp();
        self.i = state.cpu.i();
        self.dt = state.cpu.dt();
        self.st = state.cpu.st();
        self.fps = state.run.fps;
        self.hz = state.run.hz;
        self.v.clone_from_slice(&state.cpu.registers());
    }

    fn render(&mut self, context: ContextRef, _state: &UpdateState) {
        let rect = self.rect();
        let mut x = rect.left() + 20;
        let separator = Rect::new(rect.left() + 20, rect.top() + 110, rect.width() - 40, 5);

        {
            let mut canvas = context.canvas.borrow_mut();
            canvas.set_draw_color(*COLOR_BG);
            canvas.fill_rect(separator).unwrap();
        }

        for col in 0..4 {
            let mut y = rect.top() + 135;
            for row in 0..4 {
                let i = col * 4 + row;
                let v = self.v[i];
                text!(context {
                    Style::Default     => x,       y => format!("V{:X}", i)
                    Style::Address     => x + 60,  y => format!("{:02X}", v)
                    Style::Instruction => x + 100, y => format!("({})", v)
                });
                y += LINE_HEIGHT;
            }

            x += 200;
        }

        let x = rect.left() + 20;
        let y = rect.top() + 10;

        text!(context {
            Style::Default => x,       y      => "PC"
            Style::Address => x + 60,  y      => format!("{:04X}", self.pc)
            Style::Default => x + 200, y      => "ST"
            Style::Address => x + 260, y      => format!("{:02X}", self.st)
            Style::Default => x + 400, y      => "DT"
            Style::Address => x + 460, y      => format!("{:02X}", self.dt)
            Style::Default => x + 600, y      => "SP"
            Style::Address => x + 660, y      => format!("{:02X}", self.sp)
            Style::Default => x,       y + 40 => "I"
            Style::Address => x + 60,  y + 40 => format!("{:04X }", self.i)
            Style::Default => x + 200, y + 40 => "HZ"
            Style::Address => x + 260, y + 40 => format!("{:04}", self.hz)
            Style::Default => x + 400, y + 40 => "FPS"
            Style::Address => x + 460, y + 40 => format!("{:02}", self.fps)
        });
    }
}

pub struct Log {
    messages: VecDeque<String>,
}

impl Log {
    #[inline(always)]
    fn new() -> Log {
        Log {
            messages: VecDeque::new(),
        }
    }
}

impl Component for Log {
    #[inline(always)]
    fn rect(&self) -> Rect {
        *RECT_LOG
    }

    fn update(&mut self, ctx: ContextRef, _state: &UpdateState) {
        if ctx.log.unread() > 0 {
            let read = ctx.log.read();
            self.messages.extend(read);
            while self.messages.len() > N_MESSAGES {
                self.messages.pop_front();
            }
        }
    }

    fn render(&mut self, context: ContextRef, _state: &UpdateState) {
        let font = Style::Default;

        let mut y = self.rect().top() + 10;
        for message in &self.messages {
            text!(context {
                font => self.rect().left() + 20, y => message.to_owned()
            });
            y += LINE_HEIGHT;
        }
    }
}

struct Panel(Box<Component>);

impl Component for Panel {
    #[inline(always)]
    fn rect(&self) -> Rect {
        self.0.rect()
    }

    #[inline(always)]
    fn update(&mut self, _ctx: ContextRef, _state: &UpdateState) {
        self.0.update(_ctx, _state)
    }

    fn render(&mut self, context: ContextRef, state: &UpdateState) {
        {
            let mut canvas = context.canvas.borrow_mut();
            canvas.set_draw_color(*COLOR_PANEL);
            canvas.fill_rect(self.0.rect()).unwrap();
        }

        self.0.render(context, state);
    }
}

pub struct Display<'a> {
    context: ContextRef<'a>,
    panels: Vec<Panel>,
    frame: u128,
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

        let panels = vec![
            panel!(Instructions::new()),
            panel!(Screen::new()),
            panel!(Registers::new()),
            panel!(Log::new()),
        ];

        let context = Rc::new(Context {
            cache,
            log,
            canvas: Rc::new(RefCell::new(canvas)),
            font: ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap(),
        });

        Display {
            context,
            panels,
            frame: 0,
        }
    }

    pub fn update(&mut self, state: &UpdateState) {
        let context = &self.context;

        {
            let mut canvas = context.canvas.borrow_mut();
            canvas.set_draw_color(*COLOR_BG);
            canvas.clear();
        }

        for p in &mut self.panels {
            // TODO less awful
            if self.frame % 7 < 2 {
                p.update(context.clone(), state);
            }
            p.render(context.clone(), state);
        }

        let mut canvas = self.context.canvas.borrow_mut();
        canvas.present();
        self.frame += 1;
    }

    pub fn focused(&self) -> bool {
        let canvas = self.context.canvas.borrow();
        let flags = canvas.window().window_flags() as u32;
        let focused = SDL_WindowFlags::SDL_WINDOW_INPUT_FOCUS as u32;
        (flags & focused) == focused
    }
}
