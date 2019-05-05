extern crate sdl2;

use cpu::Chip8;
use sdl2::pixels::PixelFormatEnum;
use std::path::Path;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::Window;
use sdl2::Sdl;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use cpu::Chip8State;
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 576;
const FONT_SIZE: u16 = 28;

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

pub trait Renderable {
    fn render(&mut self, state: &Chip8State);
    fn draw_text<'a, 'b>(
        &self,
        text: &str,
        x: i32,
        y: i32,
        color: Color,
        font: &Font<'a, 'b>,
        canvas: &mut Canvas<Window>,
    ) {
        let surface = font.render(text).blended(color).unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let target = Rect::new(x, y, surface.width(), surface.height());

        canvas.copy(&texture, None, Some(target)).unwrap();
    }
}

pub struct Panel<'a> {
    rect: Rect,
    canvas: Rc<RefCell<Canvas<Window>>>,
    contents: Box<dyn Renderable + 'a>,
}

impl<'a> Renderable for Panel<'a> {
    fn render(&mut self, state: &Chip8State) {
        {
            let mut canvas = self.canvas.borrow_mut();
            canvas.set_draw_color(*PANEL_COLOR);
            canvas.fill_rect(self.rect).unwrap();
        }
        self.contents.render(state);
    }
}

pub struct Screen {
    canvas: Rc<RefCell<Canvas<Window>>>,
}

impl Renderable for Screen {
    fn render(&mut self, state: &Chip8State) {
        let mut canvas = self.canvas.borrow_mut();
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

pub struct Instructions<'a, 'b> {
    address: usize,
    font: Rc<RefCell<Font<'a, 'b>>>,
    canvas: Rc<RefCell<Canvas<Window>>>,
}

impl<'a, 'b> Renderable for Instructions<'a, 'b> {
    fn render(&mut self, state: &Chip8State) {
        let panel = *INSTRUCTION_FRAME;
        let hihglight_width = panel.width() - 25;
        let x = panel.left() + 15;
        let mut y = panel.top() + 10;

        if state.pc < self.address || state.pc - self.address > 30 || state.pc - self.address < 4 {
            self.address = state.pc - 4;
        }

        let mut r = self.canvas.borrow_mut();
        let canvas = r.deref_mut();
        let mut f = self.font.borrow_mut();
        let font = f.deref_mut();
        let spacing = ((font.recommended_line_spacing() as f32) * 1.18) as i32;
        for offset in 0..26 {
            let addr = self.address + offset * 2;
            let highlighted = state.pc == addr;

            if highlighted {
                canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                canvas
                    .fill_rect(Rect::new(x - 4, y - 3, hihglight_width, spacing as u32))
                    .unwrap();
            }

            let result = Chip8::disassemble2(addr, state.memory);
            let addr_text = format!("{:04X}", result.address);
            self.draw_text(&addr_text, x, y, *ADDR_COLOR, font, canvas);
            let b2 = result.instruction as u8;
            let b1 = (result.instruction >> 8) as u8;
            let b1_text = format!("{:02X}", b1);
            let b2_text = format!("{:02X}", b2);
            self.draw_text(&b1_text, x + 90, y, *INST_COLOR, font, canvas);
            self.draw_text(&b2_text, x + 130, y, *INST_COLOR, font, canvas);
            self.draw_text(&result.operation, x + 190, y, *TEXT_COLOR, font, canvas);
            if result.params != "" {
                self.draw_text(&result.params, x + 310, y, *TEXT_COLOR, font, canvas);
            }

            y += spacing;
        }
    }
}

pub struct Registers<'a, 'b> {
    font: Rc<RefCell<Font<'a, 'b>>>,
    canvas: Rc<RefCell<Canvas<Window>>>,
}

impl<'a, 'b> Renderable for Registers<'a, 'b> {
    fn render(&mut self, state: &Chip8State) {
        let panel = *REGISTER_FRAME;
        let mut b = self.font.borrow_mut();
        let font = b.deref_mut();
        let mut c = self.canvas.borrow_mut();
        let canvas = c.deref_mut();
        let spacing = ((font.recommended_line_spacing() as f32) * 1.2) as i32;
        let state = state;

        let mut x = panel.left() + 15;

        for col in 0..4 {
            let mut y = panel.top() + 10;
            for row in 0..4 {
                let i = col * 4 + row;
                let value = state.v[i];
                let reg_text = format!("V{:X}", i);
                let state_text = format!("{:02X}", value);
                let trans_text = format!("({})", value);
                self.draw_text(&reg_text, x, y, *TEXT_COLOR, font, canvas);
                self.draw_text(&state_text, x + 60, y, *ADDR_COLOR, font, canvas);
                self.draw_text(&trans_text, x + 100, y, *INST_COLOR, font, canvas);
                y += spacing;
            }
            x += 200;
        }

        let pc = state.pc;
        let sp = state.sp;
        let dt = state.dt;
        let st = state.st;
        let i = state.i;

        x = panel.left() + 15;
        let mut y = panel.top() + 200;

        self.draw_text(&format!("PC {:04X}", pc), x, y, *TEXT_COLOR, font, canvas);
        x += 200;

        self.draw_text(&format!("S {}", sp), x, y, *TEXT_COLOR, font, canvas);
        x += 200;

        self.draw_text(&format!("DT {:04X}", dt), x, y, *TEXT_COLOR, font, canvas);
        x += 200;

        self.draw_text(&format!("ST {:04X}", st), x, y, *TEXT_COLOR, font, canvas);

        x = panel.left() + 15;
        y += 50;

        self.draw_text(&format!(" I {:04X}", i), x, y, *TEXT_COLOR, font, canvas);
    }
}

pub struct Log<'a, 'b> {
    font: Rc<RefCell<Font<'a, 'b>>>,
    canvas: Rc<RefCell<Canvas<Window>>>,
}

impl<'a, 'b> Renderable for Log<'a, 'b> {
    fn render(&mut self, _state: &Chip8State) {
        let panel = *LOG_FRAME;
        let mut font_ref = self.font.borrow_mut();
        let font = font_ref.deref_mut();
        let mut canvas_ref = self.canvas.borrow_mut();
        let canvas = canvas_ref.deref_mut();

        let x = panel.left() + 5;
        let y = panel.top() + 3;

        let text = "This is a log message with data";
        self.draw_text(&text, x, y, *TEXT_COLOR, font, canvas);
    }
}

pub struct Display<'a> {
    pub canvas: Rc<RefCell<Canvas<Window>>>,
    screen: Panel<'a>,
    instructions: Panel<'a>,
    registers: Panel<'a>,
    log: Panel<'a>,
}

impl<'a> Renderable for Display<'a> {
    fn render(&mut self, state: &Chip8State) {
        {
            let mut canvas = self.canvas.borrow_mut();
            canvas.set_draw_color(*BG_COLOR);
            canvas.clear();
        }

        self.instructions.render(state);
        self.registers.render(state);
        self.log.render(state);
        self.screen.render(state);
        self.canvas.borrow_mut().present();
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
        let canvas = Rc::new(RefCell::new(canvas));

        let screen = Screen {
            canvas: canvas.clone(),
        };
        let screen_panel = Panel {
            rect: *SCREEN_FRAME,
            canvas: canvas.clone(),
            contents: Box::new(screen),
        };

        let instructions = Instructions {
            address: 512,
            font: Rc::new(RefCell::new(
                ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap(),
            )),
            canvas: canvas.clone(),
        };
        let instructions_panel = Panel {
            rect: *INSTRUCTION_FRAME,
            canvas: canvas.clone(),
            contents: Box::new(instructions),
        };

        let registers = Registers {
            font: Rc::new(RefCell::new(
                ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap(),
            )),
            canvas: canvas.clone(),
        };
        let registers_panel = Panel {
            rect: *REGISTER_FRAME,
            canvas: canvas.clone(),
            contents: Box::new(registers),
        };

        let log = Log {
            font: Rc::new(RefCell::new(
                ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap(),
            )),
            canvas: canvas.clone(),
        };
        let log_panel = Panel {
            rect: *LOG_FRAME,
            canvas: canvas.clone(),
            contents: Box::new(log),
        };

        Display {
            canvas: canvas.clone(),
            screen: screen_panel,
            instructions: instructions_panel,
            registers: registers_panel,
            log: log_panel,
        }
    }
}
