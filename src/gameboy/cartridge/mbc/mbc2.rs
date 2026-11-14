use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::map;

use super::MBC;

pub struct Mbc2 {
    rom: Vec<u8>,
    ram: [u8; RAM_SIZE],
    selected_rom_bank: usize,
    rom_bank_count: usize,
    ram_enabled: bool,
}

const RAM_SIZE: usize = 512;

impl MBC for Mbc2 {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let rom_bank_count = header.rom_bank_count as usize;
        let rom_size = rom_bank_count * map::ROM_BANK_SIZE;
        let mut rom = rom;
        rom.resize(rom_size, 0xFF);
        let ram = [0; RAM_SIZE];
        let m = Self { rom, ram, rom_bank_count, selected_rom_bank: 1, ram_enabled: false };
        Some(m)
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..0x4000 => self.rom[addr as usize],
            0x4000..map::ROM_END => self.rom[self.selected_rom_bank * map::ROM_BANK_SIZE + (addr as usize - 0x4000)],
            map::SRAM_START..map::SRAM_END if self.ram_enabled => self.ram[(addr & 0x01FF) as usize] | 0xF0,
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x4000 if addr & 0x0100 == 0 => self.ram_enabled = value & 0x0F == 0x0A,
            0x0000..0x4000 => self.selected_rom_bank = usize::max(value as usize & 0x0F, 1) & (self.rom_bank_count - 1),
            map::SRAM_START..map::SRAM_END if self.ram_enabled => self.ram[(addr & 0x01FF) as usize] = value,
            _ => (),
        }
    }

    fn rom(&self) -> Vec<u8> { self.rom.clone() }
    fn ram(&self) -> &[u8] { self.ram.as_slice() }
    fn load_ram(&mut self, ram: Vec<u8>) {
        let mut ram = ram;
        ram.resize(RAM_SIZE, 0);
        self.ram.copy_from_slice(&ram);
    }
}
