use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

use super::MBC;

#[derive(Default)]
pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank_count: usize,
    ram_bank_count: usize,
    ram_size: usize,
    bank1: IORegister<0x00, 0x1F>,
    bank2: IORegister<0x00, 0x03>,
    ram_enabled: bool,
    banking_mode: u8,
}

impl MBC for Mbc1 {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let rom_bank_count = header.rom_bank_count as usize;
        let rom_size = rom_bank_count * map::CART_ROM_BANK_SIZE;
        let mut rom = rom;
        rom.resize(rom_size, 0xFF);
        let ram_bank_count = header.ram_bank_count as usize;
        let ram_size = ram_bank_count * map::CART_RAM_BANK_SIZE;
        let ram = vec![0; ram_size];
        Some(Self { rom, ram, rom_bank_count, ram_bank_count, ram_size, ..Default::default() })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.rom[self.rom_offset(addr)],
            map::SRAM_START..map::SRAM_END if self.ram_enabled => self.ram[self.ram_offset(addr)],
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000 => self.ram_enabled = value & 0x0F == 0x0A && self.ram_size > 0,
            0x2000..0x4000 => self.bank1.write(value),
            0x4000..0x6000 => self.bank2.write(value),
            0x6000..0x8000 => self.banking_mode = value & 1,
            map::SRAM_START..map::SRAM_END if self.ram_enabled => {
                let offset = self.ram_offset(addr);
                self.ram[offset] = value
            }
            _ => (),
        }
    }

    fn rom(&self) -> Vec<u8> { self.rom.clone() }
    fn ram(&self) -> &[u8] { self.ram.as_slice() }
    fn load_ram(&mut self, ram: Vec<u8>) { self.ram = ram; }
    fn load_rtc(&mut self, _: [u8; 5]) { unreachable!() }
}

impl Mbc1 {
    fn rom_offset(&self, addr: u16) -> usize {
        self.selected_rom_bank(addr) * map::CART_ROM_BANK_SIZE + (addr & 0x3FFF) as usize
    }

    fn ram_offset(&self, addr: u16) -> usize {
        self.selected_ram_bank() * map::CART_RAM_BANK_SIZE + (addr - map::SRAM_START) as usize
    }

    fn selected_rom_bank(&self, addr: u16) -> usize {
        let b1 = self.bank1.read() & 0x1F;
        let b2 = self.bank2.read() & 0x03;
        let bank = match (self.banking_mode, addr) {
            (0, map::ROM_START..map::ROM_HALF) => 0,
            (1, map::ROM_START..map::ROM_HALF) => b2 << 5,
            (0 | 1, map::ROM_HALF..map::ROM_END) if b1 == 0 => (b2 << 5) | 1,
            (0 | 1, map::ROM_HALF..map::ROM_END) => (b2 << 5) | b1,
            _ => unreachable!(),
        };
        bank as usize & (self.rom_bank_count - 1)
    }

    fn selected_ram_bank(&self) -> usize {
        let bank = match self.banking_mode {
            0 => 0,
            1 => self.bank2.read() & 0x03,
            _ => unreachable!(),
        };
        bank as usize & self.ram_bank_count.saturating_sub(1)
    }
}
