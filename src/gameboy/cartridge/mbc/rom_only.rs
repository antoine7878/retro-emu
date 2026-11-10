use crate::gameboy::cartridge::header::{Addon, CartridgeHeader};
use crate::gameboy::memory::map::{ROM_END, ROM_START, SRAM_END, SRAM_START};

use super::MBC;

pub struct RomOnly {
    rom: Vec<u8>,
    ram: Option<[u8; 1 << 13]>,
}

impl MBC for RomOnly {
    fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self {
        let ram = header.addons.contains(&Addon::Ram).then_some([0; 1 << 13]);
        Self { rom, ram }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            ROM_START..ROM_END => self.rom[(addr & 0x3FFF) as usize],
            SRAM_START..SRAM_END => self.ram.map_or_else(|| 0xFF, |r| r[(addr - SRAM_START) as usize]),
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        let Some(ram) = &mut self.ram else { return };
        if matches!(addr, SRAM_START..SRAM_END) {
            ram[(addr - SRAM_START) as usize] = value;
        }
    }
}
