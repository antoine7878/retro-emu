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
        let file_name = format!("save_{}", header.rom_name);
        Some(Self { header, mbc, file_name })
    }

    pub fn load_ram(&mut self) {
        let Ok(mut file) = File::open(&self.file_name) else { return };
        if self.header.has_rtc() {
            let mut clock = [0u8; 5];
            let Ok(()) = file.read_exact(&mut clock) else { return };
            self.mbc.load_rtc(clock);
        }
        if self.header.has_battery() {
            let mut ram = Vec::new();
            let Ok(_) = file.read_to_end(&mut ram) else { return };
            self.mbc.load_ram(ram);
        }
    }

    pub fn save_ram(&self) {
        if self.header.has_rtc() {
            let Ok(mut file) = File::create(&self.file_name) else { return };
            let fake_clock = [0u8; 5];
            let Ok(()) = file.write_all(&fake_clock) else { return };
        }
        if self.header.has_battery() {
            let ram = self.mbc.ram();
            let Ok(mut file) = File::create(&self.file_name) else { return };
            let Ok(()) = file.write_all(ram) else { return };
        }
    }

    pub fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    pub fn write(&mut self, addr: u16, value: u8) { self.mbc.write(addr, value); }
    pub fn header(&self) -> &CartridgeHeader { &self.header }
    pub fn rom(&self) -> Vec<u8> { self.mbc.rom() }
    pub fn take_rumble(&mut self) -> f32 { self.mbc.take_rumble() }
}
