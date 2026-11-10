use std::collections::HashSet;

use crate::gameboy::config::CGMode;

const ROM_SIZE_FLAG: usize = 0x0148;
const RAM_SIZE_FLAG: usize = 0x149;

const TITLE_START: usize = 0x0134;
const TITLE_END: usize = 0x0143;
pub const CGB_FLAG: usize = 0x143;
const CARTIDGE_TYPE: usize = 0x0147;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbcKind {
    RomOnly,
    Mbc1,
    Mbc2,
    Mmm01,
    Mbc3,
    Mbc5,
    Mbc6,
    Mbc7,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Addon {
    Ram,
    Battery,
    Timer,
    Rumble,
    Sensor,
}

pub struct CartridgeHeader {
    pub rom_bank_count: u16,
    pub ram_bank_count: u8,
    pub rom_name: String,
    pub cgb_mode: CGMode,
    pub mbc_kind: MbcKind,
    pub addons: HashSet<Addon>,
}

impl CartridgeHeader {
    pub fn new(rom: &[u8]) -> Self {
        let rom_bank_count = Self::rom_bank_count(rom);
        let ram_bank_count = Self::ram_bank_count(rom);
        let rom_name = Self::rom_name(rom);
        let cgb_mode = Self::mode(rom);
        let (mbc_kind, addons) = Self::cartridge_type(rom);
        CartridgeHeader { rom_bank_count, ram_bank_count, rom_name, cgb_mode, mbc_kind, addons }
    }

    fn rom_name(rom: &[u8]) -> String {
        rom[TITLE_START..TITLE_END].iter().take_while(|&&b| b != 0).map(|&b| b as char).collect()
    }

    fn mode(rom: &[u8]) -> CGMode {
        match rom[CGB_FLAG] {
            0x80 => CGMode::Enhanced,
            0xC0 => CGMode::Cgb,
            _ => CGMode::Dmg,
        }
    }

    fn cartridge_type(rom: &[u8]) -> (MbcKind, HashSet<Addon>) {
        use Addon::*;
        let (kind, addons): (MbcKind, &[Addon]) = match rom[CARTIDGE_TYPE] {
            0x00 => (MbcKind::RomOnly, &[]),
            0x01 => (MbcKind::Mbc1, &[]),
            0x02 => (MbcKind::Mbc1, &[Ram]),
            0x03 => (MbcKind::Mbc1, &[Ram, Battery]),
            0x05 => (MbcKind::Mbc2, &[]),
            0x06 => (MbcKind::Mbc2, &[Battery]),
            0x08 => (MbcKind::RomOnly, &[Ram]),
            0x09 => (MbcKind::RomOnly, &[Ram, Battery]),
            0x0B => (MbcKind::Mmm01, &[]),
            0x0C => (MbcKind::Mmm01, &[Ram]),
            0x0D => (MbcKind::Mmm01, &[Ram, Battery]),
            0x0F => (MbcKind::Mbc3, &[Timer, Battery]),
            0x10 => (MbcKind::Mbc3, &[Timer, Ram, Battery]),
            0x11 => (MbcKind::Mbc3, &[]),
            0x12 => (MbcKind::Mbc3, &[Ram]),
            0x13 => (MbcKind::Mbc3, &[Ram, Battery]),
            0x19 => (MbcKind::Mbc5, &[]),
            0x1A => (MbcKind::Mbc5, &[Ram]),
            0x1B => (MbcKind::Mbc5, &[Ram, Battery]),
            0x1C => (MbcKind::Mbc5, &[Rumble]),
            0x1D => (MbcKind::Mbc5, &[Rumble, Ram]),
            0x1E => (MbcKind::Mbc5, &[Rumble, Ram, Battery]),
            0x20 => (MbcKind::Mbc6, &[]),
            0x22 => (MbcKind::Mbc7, &[Sensor, Rumble, Ram, Battery]),
            0xFC => (MbcKind::PocketCamera, &[]),
            0xFD => (MbcKind::BandaiTama5, &[]),
            0xFE => (MbcKind::HuC3, &[]),
            0xFF => (MbcKind::HuC1, &[Ram, Battery]),
            _ => unimplemented!("Invalid cartridge type"),
        };
        (kind, addons.iter().copied().collect())
    }

    fn rom_bank_count(rom: &[u8]) -> u16 {
        match rom[ROM_SIZE_FLAG] {
            s @ (0x00..0x08) => 1 << (s + 1),
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => unimplemented!("Invalid rom size"),
        }
    }

    fn ram_bank_count(rom: &[u8]) -> u8 {
        match rom[RAM_SIZE_FLAG] {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => unimplemented!("Invalid ram size"),
        }
    }
}
