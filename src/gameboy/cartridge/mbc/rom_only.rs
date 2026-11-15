use crate::gameboy::cartridge::header::{Addon, CartridgeHeader};
use crate::gameboy::memory::map;
use crate::save_state_struct;

use super::Mbc;

#[derive(Default)]
pub struct RomOnly {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

save_state_struct!(RomOnly { ram });

impl Mbc for RomOnly {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let ram_size = if header.addons.contains(&Addon::Ram) { map::SRAM_BANK_SIZE } else { 0 };
        let ram = vec![0; ram_size];
        let mut rom = rom;
        rom.resize(map::ROM_END as usize, 0);
        Some(Self { rom, ram })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.rom[addr as usize],
            map::SRAM_START..map::SRAM_END if !self.ram.is_empty() => self.ram[(addr - map::SRAM_START) as usize],
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if matches!(addr, map::SRAM_START..map::SRAM_END) && !self.ram.is_empty() {
            self.ram[(addr - map::SRAM_START) as usize] = value;
        }
    }

    fn rom(&self) -> Vec<u8> { self.rom.clone() }

    fn ram(&self) -> Vec<u8> { self.ram.clone() }

    fn load_ram(&mut self, ram: Vec<u8>) {
        if self.ram.is_empty() {
            return;
        }
        let mut ram = ram;
        ram.resize(self.ram.len(), 0);
        self.ram = ram;
    }
}
