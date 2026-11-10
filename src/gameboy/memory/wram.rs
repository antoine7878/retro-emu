use crate::gameboy::config::CGMode;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map::{SVBK, WRAM_END, WRAM_SIZE, WRAM_START};

pub struct WRam {
    banks: [[u8; WRAM_SIZE]; 8],
    svbk: IORegister<SVBK, 0x07>,
}

impl Default for WRam {
    fn default() -> Self { Self { banks: [[0; WRAM_SIZE]; 8], svbk: IORegister::default() } }
}

const FIRST_END: u16 = WRAM_START + (WRAM_SIZE / 2) as u16;

impl WRam {
    pub fn reset(&mut self, mode: &CGMode) {
        self.svbk.reset(mode);
        self.banks = [[0; WRAM_SIZE]; 8];
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            WRAM_START..FIRST_END => self.banks[0][(addr - WRAM_START) as usize],
            FIRST_END..WRAM_END => {
                self.banks[self.svbk.read_raw().max(1) as usize & 0x07][(addr - WRAM_START) as usize]
            }
            SVBK => self.svbk.read(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            WRAM_START..FIRST_END => self.banks[0][(addr - WRAM_START) as usize] = value,
            FIRST_END..WRAM_END => {
                self.banks[self.svbk.read_raw().max(1) as usize & 0x07][(addr - WRAM_START) as usize] = value
            }
            SVBK => self.svbk.write(value),
            _ => unreachable!(),
        }
    }
}
