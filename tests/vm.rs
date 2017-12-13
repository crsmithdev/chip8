extern crate lib;
use lib::vm;

#[cfg(test)]
mod integration_tests {

    /*
    fn mem_eq(left: &[u8], right: &[u8]) -> bool {
        (left.len() == right.len()) &&
        left.iter()
        .zip(right)
        .all(|(a,b)| a == b)
    }
    */

    fn to_instruction(string: &str) -> u16 {
        u16::from_str_radix(string, 16).unwrap()
    }

    use super::*;

    /*
    #[test]
    fn test_cls() {
        let mut c8 = chip8::chip8::Chip8::new();
        c8.cls().unwrap();
        assert!(mem_eq(&c8.video, &[0u8; 256]));
    }

    #[test]
    fn test_jp() {
        let mut c8 = chip8::chip8::Chip8::new();
        c8.jp(42).unwrap();
        assert_eq!(c8.pc, 42);
    }
    */

    #[test]
    fn test_ld_byte() {
        let mut vm = vm::Chip8::new();

        for i in 0..16 {
            let instruction = to_instruction(&format!("6{:X}FF", i));

            vm.execute(instruction).unwrap();
            assert_eq!(vm.v[i], 0xFF);
        }
    }

    #[test]
    fn test_ld_reg() {
        let mut vm = vm::Chip8::new();

        vm.execute(0x60FF).unwrap();

        for i in 1..16 {
            let instruction = to_instruction(&format!("8{:X}00", i));
            vm.execute(instruction).unwrap();
            assert_eq!(vm.v[i], 0xFF);
        }
    }

    #[test]
    fn test_drw() {
        let mut vm = vm::Chip8::new();

        let sprite = 0b10000000;
        let addr: u16 = 512;
        vm.memory[addr as usize] = sprite;
        vm.i = addr;
        vm.v[0] = 0;
        vm.v[1] = 0;

        for i in 0..8 {
            vm.v[0] = i;
            vm.execute(0xD011).unwrap();
            println!("{} {:08b}", i, 0b10000000 >> i);
            assert_eq!(vm.video[0], 0b10000000 >> i);
            vm.execute(0xD011).unwrap();
            assert_eq!(vm.video[0], 0);
        }

        //vm.execute(0xD011).unwrap();

//assert_eq!(vm.video[0], 0b10000000);
    }
}
