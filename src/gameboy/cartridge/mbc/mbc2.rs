use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::map;
use crate::save_state_struct;

use super::{Banks, Mbc};

pub struct Mbc2 {
    mem: Banks,
    selected_rom_bank: usize,
    ram_enabled: bool,
}

save_state_struct!(Mbc2 { mem, selected_rom_bank, ram_enabled });

const RAM_SIZE: usize = 512;

impl Mbc for Mbc2 {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let mem = Banks::new(rom, header.rom_bank_count as usize, RAM_SIZE);
        Some(Self { mem, selected_rom_bank: 1, ram_enabled: false })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_HALF => self.mem.read_rom(0, addr),
            map::ROM_HALF..map::ROM_END => self.mem.read_rom(self.selected_rom_bank, addr),
            map::SRAM_START..map::SRAM_END if self.ram_enabled => self.mem.read_ram(0, ram_addr(addr)) | 0xF0,
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x4000 if addr & 0x0100 == 0 => self.ram_enabled = value & 0x0F == 0x0A,
            0x0000..0x4000 => self.selected_rom_bank = usize::max(value as usize & 0x0F, 1),
            map::SRAM_START..map::SRAM_END if self.ram_enabled => self.mem.write_ram(0, ram_addr(addr), value),
            _ => (),
        }
    }

    fn rom(&self) -> Vec<u8> { self.mem.rom() }
    fn ram(&self) -> Vec<u8> { self.mem.ram() }
    fn load_ram(&mut self, ram: Vec<u8>) { self.mem.load_ram(ram) }
}

fn ram_addr(addr: u16) -> u16 { map::SRAM_START | (addr & 0x01FF) }
