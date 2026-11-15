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

use crate::gameboy::serializer::{SaveState, SerdeError, StateReader, StateWriter};

use super::header::{CartridgeHeader, MbcKind};

pub trait Mbc: Sized {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self>;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
    fn rom(&self) -> Vec<u8>;
    fn ram(&self) -> Vec<u8>;
    fn load_ram(&mut self, ram: Vec<u8>);
    fn take_rumble(&mut self) -> f32 { 0. }
    fn tick(&mut self, _: u8) {}
}

pub enum Mbcs {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
    Mbc2(Mbc2),
    Mbc3(Mbc3),
    Mbc5(Mbc5),
}

macro_rules! dispatch {
    ($self:expr, $field:ident $(, $arg:expr)*) => {
        match $self {
            Mbcs::RomOnly(m) => m.$field($($arg,)*),
            Mbcs::Mbc1(m) => m.$field($($arg,)*),
            Mbcs::Mbc2(m) => m.$field($($arg,)*),
            Mbcs::Mbc3(m) => m.$field($($arg,)*),
            Mbcs::Mbc5(m) => m.$field($($arg,)*),
        }
    };
}

impl SaveState for Mbcs {
    fn save(&self, w: &mut StateWriter) { dispatch!(self, save, w) }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> { dispatch!(self, load, r) }
}

impl Default for Mbcs {
    fn default() -> Self { Mbcs::RomOnly(RomOnly::default()) }
}

impl Mbc for Mbcs {
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

    fn read(&self, addr: u16) -> u8 { dispatch!(self, read, addr) }
    fn write(&mut self, addr: u16, value: u8) { dispatch!(self, write, addr, value) }
    fn rom(&self) -> Vec<u8> { dispatch!(self, rom) }
    fn ram(&self) -> Vec<u8> { dispatch!(self, ram) }
    fn load_ram(&mut self, ram: Vec<u8>) { dispatch!(self, load_ram, ram) }
    fn tick(&mut self, dots: u8) { dispatch!(self, tick, dots) }
    fn take_rumble(&mut self) -> f32 { dispatch!(self, take_rumble) }
}
