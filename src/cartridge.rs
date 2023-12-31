use crate::rom::Rom;
use std::fs::File;
use std::io::Read;

pub fn load_rom(path: &str) -> Rom {
    let mut f = File::open(path).expect("no file found");
    let metadata = std::fs::metadata(path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let rom = Rom::new(&buffer).expect("load error");
    rom
}

pub mod test {
    use super::*;

    pub fn snake_rom() -> Rom {
        load_rom("roms/snake.nes")
    }

    pub fn test_rom() -> Rom {
        load_rom("tests/roms/nestest.nes")
    }

    pub fn test_rom_hellow() -> Rom {
        load_rom("samples/helloworld/asm/hello.nes")
    }
}
