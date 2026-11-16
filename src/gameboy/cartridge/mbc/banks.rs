use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::map;
use crate::save_state_struct;

#[derive(Default)]
pub struct Banks {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

save_state_struct!(Banks { ram });

impl Banks {
    pub fn new(mut rom: Vec<u8>, rom_banks: usize, ram_size: usize) -> Self {
        rom.resize(rom_banks * map::ROM_BANK_SIZE, 0);
        Self { rom, ram: vec![0; ram_size] }
    }

    pub fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self {
        Self::new(rom, header.rom_bank_count as usize, header.ram_bank_count as usize * map::SRAM_BANK_SIZE)
    }

    pub fn has_ram(&self) -> bool { !self.ram.is_empty() }
    pub fn read_rom(&self, bank: usize, addr: u16) -> u8 { self.rom[self.rom_offset(bank, addr)] }
    pub fn read_ram(&self, bank: usize, addr: u16) -> u8 { self.ram[self.ram_offset(bank, addr)] }

    pub fn write_ram(&mut self, bank: usize, addr: u16, value: u8) {
        let offset = self.ram_offset(bank, addr);
        self.ram[offset] = value;
    }

    pub fn rom(&self) -> Vec<u8> { self.rom.clone() }
    pub fn ram(&self) -> Vec<u8> { self.ram.clone() }

    pub fn load_ram(&mut self, mut ram: Vec<u8>) {
        ram.resize(self.ram.len(), 0);
        self.ram = ram;
    }

    fn rom_offset(&self, bank: usize, addr: u16) -> usize {
        let mask = (self.rom.len() / map::ROM_BANK_SIZE).saturating_sub(1);
        (bank & mask) * map::ROM_BANK_SIZE + addr as usize % map::ROM_BANK_SIZE
    }

    fn ram_offset(&self, bank: usize, addr: u16) -> usize {
        let mask = (self.ram.len() / map::SRAM_BANK_SIZE).saturating_sub(1);
        (bank & mask) * map::SRAM_BANK_SIZE + (addr - map::SRAM_START) as usize
    }
}
