use crate::gameboy::cartridge::header::{Addon, CartridgeHeader};
use crate::gameboy::memory::map;

use super::MBC;

#[derive(Default)]
pub struct RomOnly {
    rom: Vec<u8>,
    ram: Option<[u8; 1 << 13]>,
}

impl MBC for RomOnly {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let ram = header.addons.contains(&Addon::Ram).then_some([0; 1 << 13]);
        Some(Self { rom, ram })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.rom[(addr & 0x3FFF) as usize],
            map::SRAM_START..map::SRAM_END => self.ram.map_or_else(|| 0xFF, |r| r[(addr - map::SRAM_START) as usize]),
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        let Some(ram) = &mut self.ram else { return };
        if matches!(addr, map::SRAM_START..map::SRAM_END) {
            ram[(addr - map::SRAM_START) as usize] = value;
        }
    }

    fn rom(&self) -> Vec<u8> { self.rom.clone() }
}
