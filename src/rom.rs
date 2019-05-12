#[allow(dead_code)]
pub static ROM: &'static [u64] = &[
    // 0x0000
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0xA0, 0xA0, 0xF0, 0x20, 0x20, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F

          // 0x0040
];

pub static BOOT: &'static [u8] = &[
    0xA2, 0x5B, 0x60, 0x0B, 0x61, 0x03, 0x62, 0x07, 0xD0, 0x17, 0x70, 0x07, 0xF2, 0x1E, 0xD0, 0x17,
    0x70, 0x07, 0xF2, 0x1E, 0xD0, 0x17, 0x70, 0x07, 0xF2, 0x1E, 0xD0, 0x17, 0x70, 0x07, 0xF2, 0x1E,
    0xD0, 0x17, 0x70, 0x05, 0xF2, 0x1E, 0xD0, 0x17, 0xF2, 0x1E, 0xA2, 0x5A, 0xC0, 0x3F, 0xC1, 0x1F,
    0x62, 0x01, 0x63, 0x01, 0xD0, 0x11, 0x64, 0x02, 0xF4, 0x15, 0xF4, 0x07, 0x34, 0x00, 0x12, 0x3A,
    0xD0, 0x11, 0x80, 0x24, 0x81, 0x34, 0xD0, 0x11, 0x41, 0x00, 0x63, 0x01, 0x41, 0x1F, 0x63, 0xFF,
    0x40, 0x00, 0x62, 0x01, 0x40, 0x3F, 0x62, 0xFF, 0x12, 0x36, 0x80, 0x78, 0xCC, 0xC0, 0xC0, 0xC0,
    0xCC, 0x78, 0xCC, 0xCC, 0xCC, 0xFC, 0xCC, 0xCC, 0xCC, 0xFC, 0x30, 0x30, 0x30, 0x30, 0x30, 0xFC,
    0xF8, 0xCC, 0xCC, 0xF8, 0xC0, 0xC0, 0xC0, 0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x78, 0xCC,
    0xCC, 0x78, 0xCC, 0xCC, 0x78,
];
