//mod cpu;
//use super::cpu;
//extern crate lib;
//use lib::cpu;
extern crate chip8;
use chip8::cpu;

#[cfg(test)]
mod integration_tests {

    use super::*;

    fn get_pixel(cpu: &cpu::Chip8, x: u16, y: u16) -> u8 {
        let byte_offset = (y as u32) * cpu.pitch as u32 + (x as u32) / cpu.pitch as u32;
        let byte = cpu.video[byte_offset as usize];
        let bit_offset = x % 8;
        let bit = (byte & (1 << (7 - bit_offset))) >> (7 - bit_offset);
        //println!("{} {} {:08b} {} {}", x, y, byte, bit_offset, bit);

        return bit;
    }

    #[test]
    fn test_jp() {
        let mut cpu = cpu::Chip8::new();

        cpu.execute(&[0x10FF]).unwrap();
        assert_eq!(cpu.pc, 0xFF);
    }

    #[test]
    fn test_ld() {
        let mut cpu = cpu::Chip8::new();

        cpu.execute(&[
            0x60FF, // LD v0, FF
            0x8100, // LD v1, v0
        ])
        .unwrap();

        assert_eq!(cpu.v[0], 0xFF);
        assert_eq!(cpu.v[1], 0xFF);
    }

    #[test]
    fn test_drw() {
        let mut cpu = cpu::Chip8::new();
        cpu.memory[cpu.i as usize] = 0b10000000;

        for i in 0..16 {
            cpu.execute(&[
                0x6000 + i, // LD v0, i
                0x6100 + i, // LD v1, i
                0xD011,     // DRW v0, v1, 1
            ])
            .unwrap();

            assert_eq!(1, get_pixel(&cpu, i, i));

            cpu.execute(&[
                0xD011, // DRW v0, v1, 1
            ])
            .unwrap();

            assert_eq!(0, get_pixel(&cpu, i, i));
        }
    }

    #[test]
    fn test_drw_multiline() {
        let mut cpu = cpu::Chip8::new();
        cpu.memory[cpu.i as usize] = 0b11000000;
        cpu.memory[(cpu.i + 1) as usize] = 0b11000000;

        for i in 0..15 {
            cpu.execute(&[
                0x6000 + i, // LD v0, i
                0x6100 + i, // LD v1, i
                0xD012,     // DRW v0, v1, 2
            ])
            .unwrap();

            assert_eq!(1, get_pixel(&cpu, i, i));
            assert_eq!(1, get_pixel(&cpu, i + 1, i));
            assert_eq!(1, get_pixel(&cpu, i, i + 1));
            assert_eq!(1, get_pixel(&cpu, i + 1, i + 1));

            cpu.execute(&[
                0xD012, // DRW v0, v1, 1
            ])
            .unwrap();

            assert_eq!(0, get_pixel(&cpu, i, i));
            assert_eq!(0, get_pixel(&cpu, i + 1, i));
            assert_eq!(0, get_pixel(&cpu, i, i + 1));
            assert_eq!(0, get_pixel(&cpu, i + 1, i + 1));
        }
    }
}
