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
        let mut rom = rom;
        rom.resize(map::ROM_END as usize, 0xFF);
        Some(Self { rom, ram })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.rom[(addr) as usize],
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

    fn ram(&self) -> &[u8] {
        match &self.ram {
            Some(ram) => ram.as_slice(),
            None => &[],
        }
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        self.ram.as_mut().map(|r| {
            let mut ram = ram;
            ram.resize(r.len(), 0);
            r.copy_from_slice(ram.as_slice());
        });
    }
}
