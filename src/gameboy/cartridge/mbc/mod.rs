mod mbc1;
mod rom_only;

use mbc1::Mbc1;
use rom_only::RomOnly;

use super::header::{CartridgeHeader, MbcKind};

pub trait MBC: Sized {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self>;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
    fn rom(&self) -> Vec<u8>;
}

pub enum Mbcs {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
}

impl Default for Mbcs {
    fn default() -> Self { Mbcs::RomOnly(RomOnly::default()) }
}

impl MBC for Mbcs {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        match header.mbc_kind {
            MbcKind::None => RomOnly::try_from_header(rom, header).map(Mbcs::RomOnly),
            MbcKind::Mbc1 => Mbc1::try_from_header(rom, header).map(Mbcs::Mbc1),
            _ => None,
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

    fn rom(&self) -> Vec<u8> {
        match self {
            Mbcs::RomOnly(m) => m.rom(),
            Mbcs::Mbc1(m) => m.rom(),
        }
    }
}
