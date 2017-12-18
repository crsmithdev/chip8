extern crate lib;
use lib::vm;

#[cfg(test)]
mod integration_tests {

    use super::*;

    fn get_pixel(vm: &vm::Chip8, x: u16, y: u16) -> u8 {
        let byte_offset = (y as u32) * vm.pitch + (x as u32) / vm.pitch;
        let byte = vm.video[byte_offset as usize];
        let bit_offset = x % 8;
        let bit = (byte & (1 << (7 - bit_offset))) >> (7 - bit_offset);
        //println!("{} {} {:08b} {} {}", x, y, byte, bit_offset, bit);

        return bit;
    }

    #[test]
    fn test_jp() {
        let mut vm = vm::Chip8::new();

        vm.execute(&[0x10FF]).unwrap();
        assert_eq!(vm.pc, 0xFF);
    }

    #[test]
    fn test_ld() {
        let mut vm = vm::Chip8::new();

        vm.execute(&[
            0x60FF,             // LD v0, FF
            0x8100,             // LD v1, v0
        ]).unwrap();

        assert_eq!(vm.v[0], 0xFF);
        assert_eq!(vm.v[1], 0xFF);
    }

    #[test]
    fn test_drw() {
        let mut vm = vm::Chip8::new();
        vm.memory[vm.i as usize] = 0b10000000;

        for i in 0..16 {
            vm.execute(&[
                0x6000 + i,     // LD v0, i
                0x6100 + i,     // LD v1, i
                0xD011,         // DRW v0, v1, 1
            ]).unwrap();

            assert_eq!(1, get_pixel(&vm, i, i));

            vm.execute(&[
                0xD011,         // DRW v0, v1, 1
            ]).unwrap();

            assert_eq!(0, get_pixel(&vm, i, i));
        }
    }

    #[test]
    fn test_drw_multiline() {
        let mut vm = vm::Chip8::new();
        vm.memory[vm.i as usize] = 0b11000000;
        vm.memory[(vm.i + 1) as usize] = 0b11000000;

        for i in 0..15 {
            vm.execute(&[
                0x6000 + i,     // LD v0, i
                0x6100 + i,     // LD v1, i
                0xD012,         // DRW v0, v1, 2
            ]).unwrap();

            assert_eq!(1, get_pixel(&vm, i, i));
            assert_eq!(1, get_pixel(&vm, i + 1, i));
            assert_eq!(1, get_pixel(&vm, i, i + 1));
            assert_eq!(1, get_pixel(&vm, i + 1, i + 1));

            vm.execute(&[
                0xD012,         // DRW v0, v1, 1
            ]).unwrap();

            assert_eq!(0, get_pixel(&vm, i, i));
            assert_eq!(0, get_pixel(&vm, i + 1, i));
            assert_eq!(0, get_pixel(&vm, i, i + 1));
            assert_eq!(0, get_pixel(&vm, i + 1, i + 1));
        }
    }
}
