extern crate sdl2;

use cpu::Chip8;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::Font;
use sdl2::video::Window;

use cpu::Chip8State;
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 576;

lazy_static! {
    static ref TEXT_COLOR: Color = Color::RGBA(166, 172, 205, 0xFF);
    static ref ADDR_COLOR: Color = Color::RGBA(130, 170, 255, 255);
    static ref INST_COLOR: Color = Color::RGBA(198, 146, 233, 255);
    static ref BG_COLOR: Color = Color::RGBA(27, 31, 43, 0xFF);
    static ref PANEL_COLOR: Color = Color::RGBA(42, 46, 62, 0xFF);
    //static ref INSTRUCTION_HL_COLOR: Color = Color::RGBA(1, 102, 255, 0xFF);
    static ref INSTRUCTION_HL_COLOR: Color = Color::RGBA(27, 31, 43, 255);
    static ref SCREEN_PX_OFF_COLOR: Color = Color::RGBA(0x8F, 0x91, 0x85, 0xFF);
    static ref SCREEN_PX_ON_COLOR: Color = Color::RGBA(0x11, 0x13, 0x2B, 0xFF);
    static ref SCREEN_TARGET: Rect = Rect::new(40, 40, 1470, 735);
    static ref SCREEN_FRAME: Rect = Rect::new(20, 20, 1510, 775);
    static ref LOG_FRAME: Rect = Rect::new(20, 815, 700, 317);
    static ref INSTRUCTION_FRAME: Rect = Rect::new(1550, 20, 478, 1112);
    static ref REGISTER_FRAME: Rect = Rect::new(740, 815, 790, 317);
}

pub struct Context2 {
    pub sdl: Rc<RefCell<sdl2::Sdl>>,
    pub ttf: Rc<RefCell<sdl2::ttf::Sdl2TtfContext>>,
}

pub struct Display2<'a> {
    context: &'a Context2,
    window: Window,
}

impl<'a> Display2<'a> {
    pub fn new(context: &'a Context2) -> Display2 {
        let window = context
            .sdl
            .borrow_mut()
            .video()
            .unwrap()
            .window("CHIP-8", WINDOW_WIDTH, WINDOW_HEIGHT)
            .allow_highdpi()
            .build()
            .unwrap();

        Display2 {
            context: context,
            window: window,
        }
    }
}

pub struct Context<'a, 'b> {
    pub canvas: Canvas<Window>,
    pub screen: Texture<'b>,
    pub font: Font<'a, 'b>,
}

pub struct Display {
    address: usize,
}

impl Display {
    pub fn new() -> Display {
        Display { address: 512 }
    }

    pub fn redraw(&mut self, context: &mut Context, state: &Chip8State) {
        context.canvas.set_draw_color(*BG_COLOR);
        context.canvas.clear();

        self.draw_screen(context, state);
        self.draw_log(context);
        self.draw_instructions(context, state);
        self.draw_registers(context, state);

        context.canvas.present();
    }

    fn draw_panel(&mut self, context: &mut Context, rect: Rect) {
        context.canvas.set_draw_color(*PANEL_COLOR);
        context.canvas.fill_rect(rect).unwrap();

        //Rect::new(x - 1, y + 1, hihglight_width, spacing as u32))
        //.unwrap();

        /*
        let ul = Point::new(rect.left(), rect.top());
        let ur = Point::new(rect.right(), rect.top());
        let ll = Point::new(rect.left(), rect.bottom());
        let lr = Point::new(rect.right(), rect.bottom());
        */

        /*context.canvas.set_draw_color(*PANEL_HL_COLOR);
        context.canvas.draw_line(ll, lr).unwrap();
        context.canvas.draw_line(ur, lr).unwrap();

        context.canvas.set_draw_color(*PANEL_SH_COLOR);
        context.canvas.draw_line(ul, ur).unwrap();
        context.canvas.draw_line(ul, ll).unwrap();*/
    }

    fn draw_screen(&mut self, context: &mut Context, state: &Chip8State) {
        self.draw_panel(context, *SCREEN_FRAME);

        context
            .screen
            .with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                for byte_offset in 0..state.video.len() {
                    let vm_byte = state.video[byte_offset];

                    for bit_offset in 0..8 {
                        // 8 pixel bits per video byte, 3 texture bytes per pixel.
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

        context
            .canvas
            .copy(&context.screen, None, Some(*SCREEN_TARGET))
            .unwrap();
    }

    fn draw_log(&mut self, context: &mut Context) {
        let panel = *LOG_FRAME;
        self.draw_panel(context, panel);

        let x = panel.left() + 5;
        let y = panel.top() + 3;

        let text = "This is a log message with data";
        self.draw_text(context, &text, x, y, *TEXT_COLOR);
    }

    fn draw_instructions(&mut self, context: &mut Context, state: &Chip8State) {
        let panel = *INSTRUCTION_FRAME;
        let spacing = ((context.font.recommended_line_spacing() as f32) * 1.18) as i32;
        let hihglight_width = panel.width() - 25;
        let x = panel.left() + 15;
        let mut y = panel.top() + 10;

        self.draw_panel(context, panel);

        if state.pc < self.address || state.pc - self.address > 30 || state.pc - self.address < 4 {
            self.address = state.pc - 4;
        }

        for offset in 0..26 {
            let addr = self.address + offset * 2;
            let highlighted = state.pc == addr;

            if highlighted {
                context.canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                context
                    .canvas
                    .fill_rect(Rect::new(x - 4, y - 3, hihglight_width, spacing as u32))
                    .unwrap();
            }

            let result = Chip8::disassemble2(addr, state.memory);
            let addr_text = format!("{:04X}", result.address);
            self.draw_text(context, &addr_text, x, y, *ADDR_COLOR);
            let b2 = result.instruction as u8;
            let b1 = (result.instruction >> 8) as u8;
            let b1_text = format!("{:02X}", b1);
            let b2_text = format!("{:02X}", b2);
            self.draw_text(context, &b1_text, x + 90, y, *INST_COLOR);
            self.draw_text(context, &b2_text, x + 130, y, *INST_COLOR);
            self.draw_text(context, &result.operation, x + 190, y, *TEXT_COLOR);
            if result.params != "" {
                self.draw_text(context, &result.params, x + 310, y, *TEXT_COLOR);
            }

            y += spacing;
        }
    }

    fn draw_registers(&mut self, context: &mut Context, state: &Chip8State) {
        let panel = *REGISTER_FRAME;
        //let spacing = context.font.recommended_line_spacing();
        let spacing = ((context.font.recommended_line_spacing() as f32) * 1.2) as i32;
        self.draw_panel(context, panel);
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
                self.draw_text(context, &reg_text, x, y, *TEXT_COLOR);
                self.draw_text(context, &state_text, x + 60, y, *ADDR_COLOR);
                self.draw_text(context, &trans_text, x + 100, y, *INST_COLOR);
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

        self.draw_text(context, &format!("PC {:04X}", pc), x, y, *TEXT_COLOR);
        x += 200;

        self.draw_text(context, &format!("S {}", sp), x, y, *TEXT_COLOR);
        x += 200;

        self.draw_text(context, &format!("DT {:04X}", dt), x, y, *TEXT_COLOR);
        x += 200;

        self.draw_text(context, &format!("ST {:04X}", st), x, y, *TEXT_COLOR);

        x = panel.left() + 15;
        y += 50;

        self.draw_text(context, &format!(" I {:04X}", i), x, y, *TEXT_COLOR);
    }

    fn draw_text(&mut self, context: &mut Context, text: &str, x: i32, y: i32, color: Color) {
        let surface = context.font.render(text).blended(color).unwrap();
        let texture_creator = context.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let target = Rect::new(x, y, surface.width(), surface.height());

        context.canvas.copy(&texture, None, Some(target)).unwrap();
    }
}
