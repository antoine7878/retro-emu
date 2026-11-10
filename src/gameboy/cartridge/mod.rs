mod header;
mod mbc;

pub use header::{CGB_FLAG, CartridgeHeader};
use mbc::{MBC, Mbcs};

pub struct Cartridge {
    mbc: Mbcs,
    header: CartridgeHeader,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let header = CartridgeHeader::new(&rom);
        let mbc = Mbcs::from_header(rom, &header);
        Self { header, mbc }
    }

    pub fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    pub fn write(&mut self, addr: u16, value: u8) { self.mbc.write(addr, value) }
    pub fn header(&self) -> &CartridgeHeader { &self.header }
}
