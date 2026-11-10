mod mbc1;
mod rom_only;

use mbc1::Mbc1;
use rom_only::RomOnly;

use super::header::{CartridgeHeader, MbcKind};

pub trait MBC {
    fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub enum Mbcs {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
}

impl MBC for Mbcs {
    fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self {
        match header.mbc_kind {
            MbcKind::RomOnly => Mbcs::RomOnly(RomOnly::from_header(rom, &header)),
            MbcKind::Mbc1 => Mbcs::Mbc1(Mbc1::from_header(rom, &header)),
            _ => unimplemented!("mbc"),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match self {
            Mbcs::RomOnly(m) => m.read(addr),
            Mbcs::Mbc1(m) => m.read(addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match self {
            Mbcs::RomOnly(m) => m.write(addr, value),
            Mbcs::Mbc1(m) => m.write(addr, value),
        }
    }
}
