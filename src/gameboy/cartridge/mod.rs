mod header;
mod mbc;

use std::fs;

pub use header::CartridgeHeader;
use mbc::{MBC, Mbcs};

use crate::gameboy::roms;
use crate::gameboy::serializer::{SaveState, SerdeError, StateReader, StateWriter};
use crate::save_state_struct;

use super::Model;

pub const SAVE_DIR: &str = "rscs/save";
pub const MAGIC_SAVE: &[u8; 7] = b"GB_SAVE";
const SAVE_VERSION: u16 = 1;

pub struct Cartridge {
    mbc: Mbcs,
    header: CartridgeHeader,
    file_name: String,
}

save_state_struct!(Cartridge { mbc, header, file_name });

impl Default for Cartridge {
    fn default() -> Self {
        let bad_rom = roms::BAD_GAME.to_vec();
        Cartridge::try_new(Model::Dmg, bad_rom).unwrap()
    }
}

impl Cartridge {
    pub fn try_new(model: Model, rom: Vec<u8>) -> Option<Self> {
        let header = CartridgeHeader::try_new(model, &rom)?;
        let mbc = Mbcs::try_from_header(rom, &header)?;
        let file_name = format!("{}/{}.save", SAVE_DIR, header.rom_name);
        let mut cart = Self { header, mbc, file_name };
        cart.load_ram_file();
        Some(cart)
    }

    fn load_ram_file(&mut self) {
        if !self.header.has_battery() {
            return;
        }
        let Ok(data) = fs::read(&self.file_name) else { return };
        match self.parse_save(&data) {
            Ok(ram) => self.load_ram(ram),
            Err(e) => eprintln!("Ignoring save file {}: {}", self.file_name, e),
        }
    }

    pub fn save_ram(&self) {
        if !self.header.has_battery() {
            return;
        }
        if let Err(e) = fs::create_dir_all(SAVE_DIR).and_then(|_| fs::write(&self.file_name, self.save_data())) {
            eprintln!("Failed to write save file {}: {}", self.file_name, e);
        }
    }

    pub fn save_data(&self) -> Vec<u8> {
        let mut w = StateWriter::default();
        w.write(MAGIC_SAVE);
        SAVE_VERSION.save(&mut w);
        self.header.rom_name.save(&mut w);
        self.header.global_checksum.save(&mut w);
        self.mbc.ram().save(&mut w);
        w.into_bytes()
    }

    pub fn parse_save(&self, data: &[u8]) -> Result<Vec<u8>, SerdeError> {
        if !self.header.has_battery() {
            return SerdeError::new("save", "cartridge has no battery");
        }
        let mut r = StateReader::new(data);
        if r.take(MAGIC_SAVE.len())? != MAGIC_SAVE {
            return SerdeError::new("header", "not a save file");
        }
        let mut version = 0u16;
        version.load(&mut r)?;
        if version != SAVE_VERSION {
            return SerdeError::new("header", "unsupported version");
        }
        let mut rom_name = String::new();
        rom_name.load(&mut r)?;
        let mut global_checksum = 0u16;
        global_checksum.load(&mut r)?;
        if rom_name != self.header.rom_name || global_checksum != self.header.global_checksum {
            return SerdeError::new("header", "save belongs to another game");
        }
        let mut len = 0usize;
        len.load(&mut r)?;
        Ok(r.take(len)?.to_vec())
    }

    pub fn load_ram(&mut self, ram: Vec<u8>) { self.mbc.load_ram(ram); }

    pub fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    pub fn write(&mut self, addr: u16, value: u8) { self.mbc.write(addr, value); }
    pub fn header(&self) -> &CartridgeHeader { &self.header }
    pub fn rom(&self) -> Vec<u8> { self.mbc.rom() }
    pub fn take_rumble(&mut self) -> f32 { self.mbc.take_rumble() }
    pub fn tick(&mut self, dots: u8) { self.mbc.tick(dots); }
}
