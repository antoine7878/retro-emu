mod header;
mod mbc;

pub use header::CartridgeHeader;
use mbc::{MBC, Mbcs};

#[derive(Default)]
pub struct Cartridge {
    mbc: Mbcs,
    header: CartridgeHeader,
}

impl Cartridge {
    pub fn try_new(rom: Vec<u8>) -> Option<Self> {
        let header = CartridgeHeader::try_new(&rom)?;
        let mbc = Mbcs::try_from_header(rom, &header);
        Some(Self { header, mbc })
    }

    pub fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    pub fn write(&mut self, addr: u16, value: u8) { self.mbc.write(addr, value) }
    pub fn header(&self) -> &CartridgeHeader { &self.header }
    pub fn rom(&self) -> Vec<u8> { self.mbc.rom() }
}
