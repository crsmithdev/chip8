extern crate sdl2;

use std::path::Path;
use std::error::Error;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::ttf::{Font, Sdl2TtfContext};

use cpu::{disassemble, Chip8State};

const FONT_SIZE: u16 = 28;

lazy_static! {
    static ref FONT_PATH: &'static Path = Path::new("../../resources/SourceCodePro-Semibold.ttf");

    static ref TEXT_COLOR: Color = Color::RGBA(0xFD, 0xF6, 0xE3, 0xFF);
    static ref BG_COLOR: Color = Color::RGBA(0x00, 0x2B, 0x36, 0xFF);
    static ref PANEL_HL_COLOR: Color = Color::RGBA(0x58, 0x6E, 0x75, 0xFF);
    static ref PANEL_SH_COLOR: Color = Color::RGBA(0x00, 0x20, 0x20, 0xFF);
    static ref INSTRUCTION_HL_COLOR: Color = Color::RGBA(0x2a, 0xa1, 0x98, 0xFF);
    static ref SCREEN_PX_OFF_COLOR: Color = Color::RGBA(0x8F, 0x91, 0x85, 0xFF);
    static ref SCREEN_PX_ON_COLOR: Color = Color::RGBA(0x11, 0x13, 0x2B, 0xFF);

    static ref SCREEN_TARGET: Rect = Rect::new(24, 24, 1536, 768);
    static ref SCREEN_FRAME: Rect = Rect::new(20, 20, 1540, 772);
    static ref LOG_FRAME: Rect = Rect::new(20, 808, 1540, 392);
    static ref INSTRUCTION_FRAME: Rect = Rect::new(1582, 20, 446, 580);
    static ref REGISTER_FRAME: Rect = Rect::new(1582, 620, 446, 580);
}

pub struct GfxSubsystem<'ttf, 'b> {
    font: Font<'ttf, 'b>,
    canvas: Canvas<Window>,
    address: usize,
    screen: Texture<'b>,
}

impl<'ttf, 'b> GfxSubsystem<'ttf, 'b> {
    pub fn new(
        canvas: Canvas<Window>,
        ttf_context: &'ttf Sdl2TtfContext,
        texture_creator: &'b TextureCreator<WindowContext>,
    ) -> GfxSubsystem<'ttf, 'b> {
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

    pub fn redraw(&mut self, state: &Chip8State) {
        self.canvas.set_draw_color(*BG_COLOR);
        self.canvas.clear();

        self.draw_screen(state);
        self.draw_log();
        self.draw_instructions(state);
        self.draw_registers(state);

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

    fn draw_screen(&mut self, state: &Chip8State) {
        self.draw_panel(*SCREEN_FRAME);

        self.screen
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

        self.canvas
            .copy(&self.screen, None, Some(*SCREEN_TARGET))
            .unwrap();
    }

    fn draw_log(&mut self) {
        let panel = *LOG_FRAME;
        self.draw_panel(panel);

        let x = panel.left() + 5;
        let y = panel.top() + 3;

        let text = " ";
        self.draw_text(&text, x, y, *TEXT_COLOR);
    }

    fn draw_instructions(&mut self, state: &Chip8State) {
        let panel = *INSTRUCTION_FRAME;
        let spacing = self.font.recommended_line_spacing();
        let hihglight_width = panel.width() - 10;
        let x = panel.left() + 5;
        let mut y = panel.top();

        self.draw_panel(panel);

        if state.pc < self.address || state.pc - self.address > 30 || state.pc - self.address < 4 {
            self.address = state.pc - 4;
        }

        for offset in 0..16 {
            let addr = self.address + offset * 2;

            if state.pc == addr {
                self.canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                self.canvas
                    .fill_rect(Rect::new(x - 1, y + 1, hihglight_width, spacing as u32))
                    .unwrap();
            }

            let instruction = ((state.memory[addr] as u16) << 8) + (state.memory[addr + 1] as u16);
            match disassemble(instruction) {
                Ok(text) => {
                    let text = format!("{:04X}: {}", addr, text);
                    self.draw_text(&text, x, y, *TEXT_COLOR);
                }
                Err(err) => {
                    let text = format!("{:04X}: {}", addr, err.description());
                    self.draw_text(&text, x, y, *TEXT_COLOR);
                }
            };

            y += spacing;
        }
    }

    fn draw_registers(&mut self, state: &Chip8State) {
        let panel = *REGISTER_FRAME;
        let spacing = self.font.recommended_line_spacing();
        self.draw_panel(panel);

        let mut x = panel.left() + 5;
        let mut y = panel.top() + 3;

        for i in 0..0x10 {
            let line = format!("V{:X} = {:2X}", i, state.v[i]);
            self.draw_text(&line, x, y, *TEXT_COLOR);
            y += spacing;
        }

        let pc = state.pc;
        let sp = state.sp;
        let dt = state.dt;
        let st = state.st;
        let i = state.i;

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
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let target = Rect::new(x, y, surface.width(), surface.height());

        self.canvas.copy(&texture, None, Some(target)).unwrap();
    }
}
