extern crate sdl2;

use cpu::Chip8;
use sdl2::pixels::PixelFormatEnum;
use std::path::Path;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;

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

pub struct TextRenderer<'a> {
    pub canvas: &'a Canvas<Window>,
    pub font: Font<'a, 'static>,
}

impl<'a> TextRenderer<'a> {
    pub fn new(
        context: &'a sdl2::ttf::Sdl2TtfContext,
        canvas: &'a Canvas<Window>,
    ) -> TextRenderer<'a> {
        let mut font = context.load_font(*FONT_PATH, FONT_SIZE).unwrap();
        font.set_hinting(sdl2::ttf::Hinting::Mono);

        TextRenderer {
            canvas: canvas,
            font: font,
        }
    }
}

pub struct Display<'a> {
    pub canvas: Canvas<Window>,
    font: Font<'a, 'static>,
    address: usize,
}

impl<'a> Display<'a> {
    pub fn new(sdl_context: sdl2::Sdl, ttf_context: &'a sdl2::ttf::Sdl2TtfContext) -> Display<'a> {
        //let mut event_pump = sdl_context.event_pump().unwrap();
        let window = sdl_context
            .video()
            .unwrap()
            .window("CHIP-8", WINDOW_WIDTH, WINDOW_HEIGHT)
            .allow_highdpi()
            .build()
            .unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();

        let mut font = ttf_context.load_font(*FONT_PATH, FONT_SIZE).unwrap();
        font.set_hinting(sdl2::ttf::Hinting::Mono);

        Display {
            canvas: canvas,
            font: font,
            address: 512,
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
        self.canvas.set_draw_color(*PANEL_COLOR);
        self.canvas.fill_rect(rect).unwrap();
    }

    fn draw_screen(&mut self, state: &Chip8State) {
        self.draw_panel(*SCREEN_FRAME);
        let texture_creator = self.canvas.texture_creator();
        let mut screen = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
            .unwrap();

        screen
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
            .copy(&screen, None, Some(*SCREEN_TARGET))
            .unwrap();
    }

    fn draw_log(&mut self) {
        let panel = *LOG_FRAME;
        self.draw_panel(panel);

        let x = panel.left() + 5;
        let y = panel.top() + 3;

        let text = "This is a log message with data";
        self.draw_text(&text, x, y, *TEXT_COLOR);
    }

    fn draw_instructions(&mut self, state: &Chip8State) {
        let panel = *INSTRUCTION_FRAME;
        let spacing = ((self.font.recommended_line_spacing() as f32) * 1.18) as i32;
        let hihglight_width = panel.width() - 25;
        let x = panel.left() + 15;
        let mut y = panel.top() + 10;

        self.draw_panel(panel);

        if state.pc < self.address || state.pc - self.address > 30 || state.pc - self.address < 4 {
            self.address = state.pc - 4;
        }

        for offset in 0..26 {
            let addr = self.address + offset * 2;
            let highlighted = state.pc == addr;

            if highlighted {
                self.canvas.set_draw_color(*INSTRUCTION_HL_COLOR);
                self.canvas
                    .fill_rect(Rect::new(x - 4, y - 3, hihglight_width, spacing as u32))
                    .unwrap();
            }

            let result = Chip8::disassemble2(addr, state.memory);
            let addr_text = format!("{:04X}", result.address);
            self.draw_text(&addr_text, x, y, *ADDR_COLOR);
            let b2 = result.instruction as u8;
            let b1 = (result.instruction >> 8) as u8;
            let b1_text = format!("{:02X}", b1);
            let b2_text = format!("{:02X}", b2);
            self.draw_text(&b1_text, x + 90, y, *INST_COLOR);
            self.draw_text(&b2_text, x + 130, y, *INST_COLOR);
            self.draw_text(&result.operation, x + 190, y, *TEXT_COLOR);
            if result.params != "" {
                self.draw_text(&result.params, x + 310, y, *TEXT_COLOR);
            }

            y += spacing;
        }
    }

    fn draw_registers(&mut self, state: &Chip8State) {
        let panel = *REGISTER_FRAME;
        //let spacing = context.font.recommended_line_spacing();
        let spacing = ((self.font.recommended_line_spacing() as f32) * 1.2) as i32;
        self.draw_panel(panel);
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
                self.draw_text(&reg_text, x, y, *TEXT_COLOR);
                self.draw_text(&state_text, x + 60, y, *ADDR_COLOR);
                self.draw_text(&trans_text, x + 100, y, *INST_COLOR);
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

        self.draw_text(&format!("PC {:04X}", pc), x, y, *TEXT_COLOR);
        x += 200;

        self.draw_text(&format!("S {}", sp), x, y, *TEXT_COLOR);
        x += 200;

        self.draw_text(&format!("DT {:04X}", dt), x, y, *TEXT_COLOR);
        x += 200;

        self.draw_text(&format!("ST {:04X}", st), x, y, *TEXT_COLOR);

        x = panel.left() + 15;
        y += 50;

        self.draw_text(&format!(" I {:04X}", i), x, y, *TEXT_COLOR);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, color: Color) {
        let surface = self.font.render(text).blended(color).unwrap();
        let texture_creator = self.canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let target = Rect::new(x, y, surface.width(), surface.height());

        self.canvas.copy(&texture, None, Some(target)).unwrap();
        //unsafe { texture.destroy() };
    }
}
