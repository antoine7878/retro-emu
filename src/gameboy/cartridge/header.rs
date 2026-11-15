use std::collections::HashSet;

use crate::gameboy::Model;
use crate::gameboy::config::Mode;
use crate::{save_state_enum, save_state_struct};

const ROM_SIZE_FLAG: usize = 0x0148;
const RAM_SIZE_FLAG: usize = 0x149;

const TITLE_START: usize = 0x0134;
const TITLE_END: usize = 0x0143;
pub const CGB_FLAG: usize = 0x143;
const CARTIDGE_TYPE: usize = 0x0147;
const HEADER_CHECKSUM: usize = 0x014D;
const GLOBAL_CHECKSUM: usize = 0x014E;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MbcKind {
    #[default]
    None,
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

save_state_enum!(MbcKind { None, Mbc1, Mbc2, Mmm01, Mbc3, Mbc5, Mbc6, Mbc7, PocketCamera, BandaiTama5, HuC3, HuC1 });

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Addon {
    #[default]
    Ram,
    Battery,
    Rtc,
    Rumble,
    Sensor,
}

save_state_enum!(Addon { Ram, Battery, Rtc, Rumble, Sensor });

#[derive(Default)]
pub struct CartridgeHeader {
    pub rom_bank_count: u16,
    pub ram_bank_count: u8,
    pub rom_name: String,
    pub mode: Mode,
    pub mbc_kind: MbcKind,
    pub addons: HashSet<Addon>,
    pub global_checksum: u16,
}

#[rustfmt::skip]
save_state_struct!(CartridgeHeader { rom_bank_count, ram_bank_count, rom_name, mode, mbc_kind, addons, global_checksum });

impl CartridgeHeader {
    pub fn try_new(model: Model, rom: &[u8]) -> Option<Self> {
        let rom_bank_count = Self::rom_bank_count(rom)?;
        let ram_bank_count = Self::ram_bank_count(rom)?;
        let rom_name = Self::parse_rom_name(rom)?;
        let cgb_mode = Self::parse_mode(rom)?;
        if cgb_mode == Mode::Cgb && model == Model::Dmg {
            return None;
        }
        let (mbc_kind, addons) = Self::cartridge_type(rom)?;
        let global_checksum = u16::from_be_bytes([*rom.get(GLOBAL_CHECKSUM)?, *rom.get(GLOBAL_CHECKSUM + 1)?]);
        let ram_bank_count = if ram_bank_count == 0 && addons.contains(&Addon::Ram) { 1 } else { ram_bank_count };
        Some(CartridgeHeader {
            rom_bank_count,
            ram_bank_count,
            rom_name,
            mode: cgb_mode,
            mbc_kind,
            addons,
            global_checksum,
        })
    }

    pub fn is_rom(data: &[u8]) -> bool {
        let Some(bytes) = data.get(TITLE_START..HEADER_CHECKSUM) else { return false };
        let checksum = bytes.iter().fold(0u8, |x, &b| x.wrapping_sub(b).wrapping_sub(1));
        data.get(HEADER_CHECKSUM) == Some(&checksum)
        // && Self::try_new(data).is_some()
    }

    fn parse_rom_name(rom: &[u8]) -> Option<String> {
        let title_range = rom.get(TITLE_START..TITLE_END)?;
        Some(title_range.iter().take_while(|&&b| b != 0).map(|&b| b as char).collect())
    }

    fn parse_mode(rom: &[u8]) -> Option<Mode> {
        let mode = match rom.get(CGB_FLAG)? {
            0x80 => Mode::Enhanced,
            0xC0 => Mode::Cgb,
            _ => Mode::Dmg,
        };
        Some(mode)
    }

    fn cartridge_type(rom: &[u8]) -> Option<(MbcKind, HashSet<Addon>)> {
        use Addon::*;
        let (kind, addons): (MbcKind, &[Addon]) = match rom.get(CARTIDGE_TYPE)? {
            0x00 => (MbcKind::None, &[]),
            0x01 => (MbcKind::Mbc1, &[]),
            0x02 => (MbcKind::Mbc1, &[Ram]),
            0x03 => (MbcKind::Mbc1, &[Ram, Battery]),
            0x05 => (MbcKind::Mbc2, &[]),
            0x06 => (MbcKind::Mbc2, &[Battery]),
            0x08 => (MbcKind::None, &[Ram]),
            0x09 => (MbcKind::None, &[Ram, Battery]),
            0x0B => (MbcKind::Mmm01, &[]),
            0x0C => (MbcKind::Mmm01, &[Ram]),
            0x0D => (MbcKind::Mmm01, &[Ram, Battery]),
            0x0F => (MbcKind::Mbc3, &[Rtc, Battery]),
            0x10 => (MbcKind::Mbc3, &[Rtc, Ram, Battery]),
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
            _ => {
                eprintln!("Invalid cartridge type");
                return None;
            }
        };
        Some((kind, addons.iter().copied().collect()))
    }

    fn rom_bank_count(rom: &[u8]) -> Option<u16> {
        match rom.get(ROM_SIZE_FLAG)? {
            s @ (0x00..0x08) => Some(1 << (s + 1)),
            0x52 => Some(72),
            0x53 => Some(80),
            0x54 => Some(96),
            _ => {
                eprintln!("Invalid rom size");
                None
            }
        }
    }

    fn ram_bank_count(rom: &[u8]) -> Option<u8> {
        match rom.get(RAM_SIZE_FLAG)? {
            0x00 => Some(0),
            0x02 => Some(1),
            0x03 => Some(4),
            0x04 => Some(16),
            0x05 => Some(8),
            _ => {
                eprintln!("Invalid ram size");
                None
            }
        }
    }

    pub fn mode(&self) -> Mode { self.mode }
    pub fn has_battery(&self) -> bool { self.addons.contains(&Addon::Battery) }
    pub fn has_rumble(&self) -> bool { self.addons.contains(&Addon::Rumble) }
    pub fn has_rtc(&self) -> bool { self.addons.contains(&Addon::Rtc) }
}
