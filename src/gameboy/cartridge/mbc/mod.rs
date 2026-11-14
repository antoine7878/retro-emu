mod mbc1;
mod mbc2;
mod mbc3;
mod mbc5;
mod rom_only;

use mbc1::Mbc1;
use mbc2::Mbc2;
use mbc3::Mbc3;
use mbc5::Mbc5;
use rom_only::RomOnly;

use super::header::{CartridgeHeader, MbcKind};

pub trait MBC: Sized {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self>;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
    fn rom(&self) -> Vec<u8>;
    fn ram(&self) -> Vec<u8>;
    fn load_ram(&mut self, ram: Vec<u8>);
    fn take_rumble(&mut self) -> f32 { 0. }
    fn tick(&mut self) {}
}

pub enum Mbcs {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
    Mbc2(Mbc2),
    Mbc3(Mbc3),
    Mbc5(Mbc5),
}

impl Default for Mbcs {
    fn default() -> Self { Mbcs::RomOnly(RomOnly::default()) }
}

impl MBC for Mbcs {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        match header.mbc_kind {
            MbcKind::None => RomOnly::try_from_header(rom, header).map(Mbcs::RomOnly),
            MbcKind::Mbc1 => Mbc1::try_from_header(rom, header).map(Mbcs::Mbc1),
            MbcKind::Mbc2 => Mbc2::try_from_header(rom, header).map(Mbcs::Mbc2),
            MbcKind::Mbc3 => Mbc3::try_from_header(rom, header).map(Mbcs::Mbc3),
            MbcKind::Mbc5 => Mbc5::try_from_header(rom, header).map(Mbcs::Mbc5),
            _ => None,
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match self {
            Mbcs::RomOnly(m) => m.read(addr),
            Mbcs::Mbc1(m) => m.read(addr),
            Mbcs::Mbc2(m) => m.read(addr),
            Mbcs::Mbc3(m) => m.read(addr),
            Mbcs::Mbc5(m) => m.read(addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match self {
            Mbcs::RomOnly(m) => m.write(addr, value),
            Mbcs::Mbc1(m) => m.write(addr, value),
            Mbcs::Mbc2(m) => m.write(addr, value),
            Mbcs::Mbc3(m) => m.write(addr, value),
            Mbcs::Mbc5(m) => m.write(addr, value),
        }
    }

    fn rom(&self) -> Vec<u8> {
        match self {
            Mbcs::RomOnly(m) => m.rom(),
            Mbcs::Mbc1(m) => m.rom(),
            Mbcs::Mbc2(m) => m.rom(),
            Mbcs::Mbc3(m) => m.rom(),
            Mbcs::Mbc5(m) => m.rom(),
        }
    }

    fn ram(&self) -> Vec<u8> {
        match self {
            Mbcs::RomOnly(m) => m.ram(),
            Mbcs::Mbc1(m) => m.ram(),
            Mbcs::Mbc2(m) => m.ram(),
            Mbcs::Mbc3(m) => m.ram(),
            Mbcs::Mbc5(m) => m.ram(),
        }
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        match self {
            Mbcs::RomOnly(m) => m.load_ram(ram),
            Mbcs::Mbc1(m) => m.load_ram(ram),
            Mbcs::Mbc2(m) => m.load_ram(ram),
            Mbcs::Mbc3(m) => m.load_ram(ram),
            Mbcs::Mbc5(m) => m.load_ram(ram),
        }
    }

    fn tick(&mut self) {
        if let Mbcs::Mbc3(m) = self {
            m.tick();
        }
    }

    fn take_rumble(&mut self) -> f32 {
        match self {
            Mbcs::Mbc5(m) => m.take_rumble(),
            _ => 0.,
        }
    }
}
