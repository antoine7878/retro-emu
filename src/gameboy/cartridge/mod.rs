mod header;
mod mbc;

use std::io::Write;
use std::{fs::File, io::Read};

pub use header::CartridgeHeader;
use mbc::{MBC, Mbcs};

pub struct Cartridge {
    mbc: Mbcs,
    header: CartridgeHeader,
    file_name: String,
}

impl Default for Cartridge {
    fn default() -> Self {
        let bad_rom = include_bytes!("./../../../rscs/error_roms/bad_game.gb").to_vec();
        Cartridge::try_new(bad_rom).unwrap()
    }
}

impl Cartridge {
    pub fn try_new(rom: Vec<u8>) -> Option<Self> {
        let header = CartridgeHeader::try_new(&rom)?;
        let mbc = Mbcs::try_from_header(rom, &header)?;
        let file_name = format!("{}.save", header.rom_name);
        Some(Self { header, mbc, file_name })
    }

    pub fn load_ram(&mut self) {
        if !self.header.has_battery() {
            return;
        }
        let mut ram = Vec::new();
        let Ok(_) = File::open(&self.file_name).and_then(|mut f| f.read_to_end(&mut ram)) else { return };
        self.mbc.load_ram(ram);
    }

    pub fn save_ram(&self) {
        if !self.header.has_battery() {
            return;
        }
        let ram = self.mbc.ram();
        let Ok(mut file) = File::create(&self.file_name) else { return };
        let Ok(()) = file.write_all(&ram) else { return };
    }

    pub fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    pub fn write(&mut self, addr: u16, value: u8) { self.mbc.write(addr, value); }
    pub fn header(&self) -> &CartridgeHeader { &self.header }
    pub fn rom(&self) -> Vec<u8> { self.mbc.rom() }
    pub fn take_rumble(&mut self) -> f32 { self.mbc.take_rumble() }
    pub fn tick(&mut self) { self.mbc.tick(); }
}
