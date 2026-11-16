use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::map;
use crate::save_state_struct;

use super::{Banks, Mbc};

#[derive(Default)]
pub struct Mbc5 {
    mem: Banks,
    rom_select_0: usize,
    rom_select_1: usize,
    ram_select: usize,
    ram_enabled: bool,

    rumble: bool,
    motor: bool,
    motor_on: u32,
    motor_writes: u32,
}

#[rustfmt::skip]
save_state_struct!(Mbc5 { mem, rom_select_0, rom_select_1, ram_select, ram_enabled, rumble, motor, motor_on, motor_writes });

impl Mbc for Mbc5 {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        let mem = Banks::from_header(rom, header);
        Some(Self { mem, rumble: header.has_rumble(), rom_select_0: 1, ..Default::default() })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.mem.read_rom(self.selected_rom_bank(addr), addr),
            map::SRAM_START..map::SRAM_END if self.ram_enabled && self.mem.has_ram() => {
                self.mem.read_ram(self.ram_select, addr)
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
            map::SRAM_START..map::SRAM_END if self.ram_enabled && self.mem.has_ram() => {
                self.mem.write_ram(self.ram_select, addr, value)
            }
            _ => (),
        }
    }

    fn rom(&self) -> Vec<u8> { self.mem.rom() }
    fn ram(&self) -> Vec<u8> { self.mem.ram() }
    fn load_ram(&mut self, ram: Vec<u8>) { self.mem.load_ram(ram) }

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
    fn selected_rom_bank(&self, addr: u16) -> usize {
        match addr {
            map::ROM_START..map::ROM_HALF => 0,
            map::ROM_HALF..map::ROM_END => (self.rom_select_1 & 0x01) << 8 | (self.rom_select_0 & 0xFF),
            _ => unreachable!(),
        }
    }
}
