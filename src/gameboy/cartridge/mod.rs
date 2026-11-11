mod header;
mod mbc;

pub use header::CartridgeHeader;
use mbc::{MBC, Mbcs};

pub struct Cartridge {
    mbc: Mbcs,
    header: CartridgeHeader,
}

impl Default for Cartridge {
    fn default() -> Self {
        let bad_rom = include_bytes!("./../../../rscs/bad_game.gb").to_vec();
        Cartridge::try_new(bad_rom).unwrap()
    }
}

impl Cartridge {
    pub fn try_new(rom: Vec<u8>) -> Option<Self> {
        let header = CartridgeHeader::try_new(&rom)?;
        let mbc = Mbcs::try_from_header(rom, &header)?;
        Some(Self { header, mbc })
    }

    pub fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    pub fn write(&mut self, addr: u16, value: u8) { self.mbc.write(addr, value) }
    pub fn header(&self) -> &CartridgeHeader { &self.header }
    pub fn rom(&self) -> Vec<u8> { self.mbc.rom() }
}
