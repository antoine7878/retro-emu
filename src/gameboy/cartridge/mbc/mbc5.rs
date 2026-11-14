use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::map;

use super::MBC;

#[derive(Default)]
pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank_count: usize,
    ram_bank_count: usize,
    rom_select_0: usize,
    rom_select_1: usize,
    ram_select: usize,
    ram_enabled: bool,

    rumble: bool,
    motor: bool,
    motor_on: u32,
    motor_writes: u32,
}

impl MBC for Mbc5 {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let rom_bank_count = header.rom_bank_count as usize;
        let rom_size = rom_bank_count * map::ROM_BANK_SIZE;
        let mut rom = rom;
        rom.resize(rom_size, 0xFF);
        let ram_bank_count = header.ram_bank_count as usize;
        let ram_size = ram_bank_count * map::SRAM_BANK_SIZE;
        let ram = vec![0; ram_size];
        let rumble = header.has_rumble();
        let m = Self { rom, ram, rom_bank_count, ram_bank_count, rumble, rom_select_0: 1, ..Default::default() };
        Some(m)
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.rom[self.rom_offset(addr)],
            map::SRAM_START..map::SRAM_END if self.ram_enabled && !self.ram.is_empty() => {
                self.ram[self.ram_offset(addr)]
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000 => self.ram_enabled = value & 0x0F == 0x0A,
            0x2000..0x3000 => self.rom_select_0 = value as usize,
            0x3000..0x4000 => self.rom_select_1 = value as usize & 0x01,
            0x4000..0x6000 if self.rumble => {
                self.ram_select = value as usize & 0x07;
                self.motor = value & 0x08 != 0;
                self.motor_writes += 1;
                self.motor_on += self.motor as u32;
            }
            0x4000..0x6000 => self.ram_select = value as usize & 0x0F,
            map::SRAM_START..map::SRAM_END if self.ram_enabled && !self.ram.is_empty() => {
                let offset = self.ram_offset(addr);
                self.ram[offset] = value;
            }
            _ => (),
        }
    }

    fn rom(&self) -> Vec<u8> { self.rom.clone() }
    fn ram(&self) -> &[u8] { self.ram.as_slice() }
    fn load_ram(&mut self, ram: Vec<u8>) {
        let mut ram = ram;
        ram.resize(self.ram_bank_count * map::SRAM_BANK_SIZE, 0);
        self.ram.copy_from_slice(&ram);
    }

    fn take_rumble(&mut self) -> f32 {
        let level = match self.motor_writes {
            0 => self.motor as u8 as f32,
            n => self.motor_on as f32 / n as f32,
        };
        self.motor_on = 0;
        self.motor_writes = 0;
        level
    }
}

impl Mbc5 {
    fn rom_offset(&self, addr: u16) -> usize {
        self.selected_rom_bank(addr) * map::ROM_BANK_SIZE + (addr & 0x3FFF) as usize
    }

    fn ram_offset(&self, addr: u16) -> usize {
        self.selected_ram_bank() * map::SRAM_BANK_SIZE + (addr - map::SRAM_START) as usize
    }

    fn selected_rom_bank(&self, addr: u16) -> usize {
        let b1 = self.rom_select_1 & 0x01;
        let b0 = self.rom_select_0 & 0xFF;
        let bank = match addr {
            map::ROM_START..map::ROM_HALF => 0,
            map::ROM_HALF..map::ROM_END => b1 << 8 | b0,
            _ => unreachable!(),
        };
        bank as usize & self.rom_bank_count.saturating_sub(1)
    }

    fn selected_ram_bank(&self) -> usize { self.ram_select & self.ram_bank_count.saturating_sub(1) }
}
