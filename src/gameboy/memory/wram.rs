use crate::gameboy::config::ConsoleType;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

pub struct WRam {
    banks: [[u8; map::WRAM_SIZE]; 8],
    svbk: IORegister<{ map::SVBK }, 0x07>,
}

impl Default for WRam {
    fn default() -> Self { Self { banks: [[0; map::WRAM_SIZE]; 8], svbk: IORegister::default() } }
}

const FIRST_END: u16 = map::WRAM_START + (map::WRAM_SIZE / 2) as u16;

impl WRam {
    pub fn reset(&mut self, mode: &ConsoleType) {
        self.svbk.reset(mode);
        self.banks = [[0; map::WRAM_SIZE]; 8];
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            map::WRAM_START..FIRST_END => self.banks[0][(addr - map::WRAM_START) as usize],
            FIRST_END..map::WRAM_END => {
                self.banks[self.svbk.read_raw().max(1) as usize & 0x07][(addr - map::WRAM_START) as usize]
            }
            map::SVBK => self.svbk.read(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            map::WRAM_START..FIRST_END => self.banks[0][(addr - map::WRAM_START) as usize] = value,
            FIRST_END..map::WRAM_END => {
                self.banks[self.svbk.read_raw().max(1) as usize & 0x07][(addr - map::WRAM_START) as usize] = value
            }
            map::SVBK => self.svbk.write(value),
            _ => unreachable!(),
        }
    }
}
