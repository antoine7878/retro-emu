use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::gameboy::memory::map;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CGMode {
    #[default]
    Dmg,
    Enhanced,
    Cgb,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub rng_seed: u32,
    pub mode: ConsoleType,
    pub frame_time: Option<Duration>,
}

impl Config {
    pub fn new(max_fps: u32) -> Result<Self, Box<dyn Error>> {
        let rng_seed = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
        let mode = ConsoleType::Dmg;
        let frame_time = (max_fps > 0).then(|| Duration::from_secs(1) / max_fps);
        Ok(Self { rng_seed, mode, frame_time })
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleType {
    Dmg,
    Cgb,
}

impl ConsoleType {
    fn to_int(&self) -> usize {
        match self {
            Self::Dmg => 0,
            Self::Cgb => 1,
        }
    }

    pub fn initial_value(&self, addr: u16) -> u8 {
        let vals = match addr {
            map::JOYPAD => [0xCF, 0xC7],
            map::SB => [0x00, 0x00],
            map::SC => [0x7E, 0x7F],
            map::DIV => [0x18, 0xFF],
            map::TIMA => [0x00, 0x00],
            map::TMA => [0x00, 0x00],
            map::TAC => [0xF8, 0xF8],
            map::IF => [0xE1, 0xE1],
            map::NR10 => [0x80, 0x80],
            map::NR11 => [0xBF, 0xBF],
            map::NR12 => [0xF3, 0xF3],
            map::NR13 => [0xFF, 0xFF],
            map::NR14 => [0xBF, 0xBF],
            map::NR21 => [0x3F, 0x3F],
            map::NR22 => [0x00, 0x00],
            map::NR23 => [0xFF, 0xFF],
            map::NR24 => [0xBF, 0xBF],
            map::NR30 => [0x7F, 0x7F],
            map::NR31 => [0xFF, 0xFF],
            map::NR32 => [0x9F, 0x9F],
            map::NR33 => [0xFF, 0xFF],
            map::NR34 => [0xBF, 0xBF],
            map::NR41 => [0xFF, 0xFF],
            map::NR42 => [0x00, 0x00],
            map::NR43 => [0x00, 0x00],
            map::NR44 => [0xBF, 0xBF],
            map::NR50 => [0x77, 0x77],
            map::NR51 => [0xF3, 0xF3],
            map::NR52 => [0xF1, 0xF1],
            map::LCDC => [0x91, 0x91],
            map::STAT => [0x81, 0xFF],
            map::SCY => [0x00, 0x00],
            map::SCX => [0x00, 0x00],
            map::LY => [0x91, 0xFF],
            map::LYC => [0x00, 0x00],
            map::DMA => [0xFF, 0x00],
            map::BGP => [0xFC, 0xFC],
            map::OBP0 => [0xFF, 0xFF],
            map::OBP1 => [0xFF, 0xFF],
            map::WY => [0x00, 0x00],
            map::WX => [0x00, 0x00],
            map::KEY0 => [0x00, 0xFF],
            map::KEY1 => [0x00, 0x7E],
            map::VBK => [0x00, 0xFE],
            map::BOOT_DISABLE => [0x00, 0xFF],
            map::HDMA1 => [0x00, 0xFF],
            map::HDMA2 => [0x00, 0xFF],
            map::HDMA3 => [0x00, 0xFF],
            map::HDMA4 => [0x00, 0xFF],
            map::HDMA5 => [0x00, 0xFF],
            map::RP => [0x00, 0x3E],
            map::BCPS => [0x00, 0xFF],
            map::BCPD => [0x00, 0xFF],
            map::OCPS => [0x00, 0xFF],
            map::OCPD => [0x00, 0xFF],
            map::SVBK => [0x00, 0xF8],
            map::IE => [0x00, 0x00],
            _ => return 0xFF,
        };
        vals[self.to_int()]
    }
}
