use crate::gameboy::cartridge::header::{Addon, CartridgeHeader};
use crate::gameboy::memory::map;
use crate::save_state_struct;

use super::{Banks, Mbc};

#[derive(Default)]
pub struct RomOnly {
    mem: Banks,
}

save_state_struct!(RomOnly { mem });

impl Mbc for RomOnly {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let ram_size = if header.addons.contains(&Addon::Ram) { map::SRAM_BANK_SIZE } else { 0 };
        Some(Self { mem: Banks::new(rom, 2, ram_size) })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.mem.read_rom((addr >= map::ROM_HALF) as usize, addr),
            map::SRAM_START..map::SRAM_END if self.mem.has_ram() => self.mem.read_ram(0, addr),
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        if matches!(addr, map::SRAM_START..map::SRAM_END) && self.mem.has_ram() {
            self.mem.write_ram(0, addr, value);
        }
    }

    fn rom(&self) -> Vec<u8> { self.mem.rom() }
    fn ram(&self) -> Vec<u8> { self.mem.ram() }
    fn load_ram(&mut self, ram: Vec<u8>) { self.mem.load_ram(ram) }
}
