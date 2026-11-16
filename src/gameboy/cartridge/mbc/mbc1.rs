use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;
use crate::save_state_struct;

use super::{Banks, Mbc};

#[derive(Default)]
pub struct Mbc1 {
    mem: Banks,
    bank1: IORegister<0x1F>,
    bank2: IORegister<0x03>,
    ram_enabled: bool,
    banking_mode: u8,
}

save_state_struct!(Mbc1 { mem, bank1, bank2, ram_enabled, banking_mode });

impl Mbc for Mbc1 {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        Some(Self { mem: Banks::from_header(rom, header), ..Default::default() })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.mem.read_rom(self.selected_rom_bank(addr), addr),
            map::SRAM_START..map::SRAM_END if self.ram_enabled => self.mem.read_ram(self.selected_ram_bank(), addr),
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000 => self.ram_enabled = value & 0x0F == 0x0A && self.mem.has_ram(),
            0x2000..0x4000 => self.bank1.write(value),
            0x4000..0x6000 => self.bank2.write(value),
            0x6000..0x8000 => self.banking_mode = value & 1,
            map::SRAM_START..map::SRAM_END if self.ram_enabled => {
                self.mem.write_ram(self.selected_ram_bank(), addr, value)
            }
            _ => (),
        }
    }

    fn rom(&self) -> Vec<u8> { self.mem.rom() }
    fn ram(&self) -> Vec<u8> { self.mem.ram() }
    fn load_ram(&mut self, ram: Vec<u8>) { self.mem.load_ram(ram) }
}

impl Mbc1 {
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
        bank as usize
    }

    fn selected_ram_bank(&self) -> usize {
        match self.banking_mode {
            0 => 0,
            1 => (self.bank2.read() & 0x03) as usize,
            _ => unreachable!(),
        }
    }
}
