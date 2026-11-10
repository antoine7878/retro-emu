use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::args::Args;

use super::cart::CGB_FLAG;
use super::memory_map::{
    BCPD, BCPS, BGP, BOOT_DISABLE, DIV, DMA, HDMA1, HDMA2, HDMA3, HDMA4, HDMA5, IE, IF, JOYPAD, KEY0, KEY1, LCDC, LY,
    LYC, NR10, NR11, NR12, NR13, NR14, NR21, NR22, NR23, NR24, NR30, NR31, NR32, NR33, NR34, NR41, NR42, NR43, NR44,
    NR50, NR51, NR52, OBP0, OBP1, OCPD, OCPS, RP, SB, SC, SCX, SCY, STAT, SVBK, TAC, TIMA, TMA, VBK, WX, WY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CGMode {
    Dmg,
    Enhanced,
    Cgb,
}

impl CGMode {
    fn to_int(&self) -> usize {
        match self {
            Self::Dmg => 0,
            Self::Enhanced => 1,
            Self::Cgb => 1,
        }
    }

    pub fn initial_value(&self, addr: u16) -> u8 {
        let i = self.to_int();
        let vals = match addr {
            JOYPAD => [0xCF, 0xC7],
            SB => [0x00, 0x00],
            SC => [0x7E, 0x7F],
            DIV => [0x18, 0xFF],
            TIMA => [0x00, 0x00],
            TMA => [0x00, 0x00],
            TAC => [0xF8, 0xF8],
            IF => [0xE1, 0xE1],
            NR10 => [0x80, 0x80],
            NR11 => [0xBF, 0xBF],
            NR12 => [0xF3, 0xF3],
            NR13 => [0xFF, 0xFF],
            NR14 => [0xBF, 0xBF],
            NR21 => [0x3F, 0x3F],
            NR22 => [0x00, 0x00],
            NR23 => [0xFF, 0xFF],
            NR24 => [0xBF, 0xBF],
            NR30 => [0x7F, 0x7F],
            NR31 => [0xFF, 0xFF],
            NR32 => [0x9F, 0x9F],
            NR33 => [0xFF, 0xFF],
            NR34 => [0xBF, 0xBF],
            NR41 => [0xFF, 0xFF],
            NR42 => [0x00, 0x00],
            NR43 => [0x00, 0x00],
            NR44 => [0xBF, 0xBF],
            NR50 => [0x77, 0x77],
            NR51 => [0xF3, 0xF3],
            NR52 => [0xF1, 0xF1],
            LCDC => [0x91, 0x91],
            STAT => [0x81, 0xFF],
            SCY => [0x00, 0x00],
            SCX => [0x00, 0x00],
            LY => [0x91, 0xFF],
            LYC => [0x00, 0x00],
            DMA => [0xFF, 0x00],
            BGP => [0xFC, 0xFC],
            OBP0 => [0xFF, 0xFF],
            OBP1 => [0xFF, 0xFF],
            WY => [0x00, 0x00],
            WX => [0x00, 0x00],
            KEY0 => [0x00, 0xFF],
            KEY1 => [0x00, 0x7E],
            VBK => [0x00, 0xFE],
            BOOT_DISABLE => [0x00, 0xFF],
            HDMA1 => [0x00, 0xFF],
            HDMA2 => [0x00, 0xFF],
            HDMA3 => [0x00, 0xFF],
            HDMA4 => [0x00, 0xFF],
            HDMA5 => [0x00, 0xFF],
            RP => [0x00, 0x3E],
            BCPS => [0x00, 0xFF],
            BCPD => [0x00, 0xFF],
            OCPS => [0x00, 0xFF],
            OCPD => [0x00, 0xFF],
            SVBK => [0x00, 0xF8],
            IE => [0x00, 0x00],
            _ => return 0xFF,
        };
        vals[i]
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub rng_seed: u32,
    pub mode: CGMode,
    pub frame_time: Option<Duration>,
}

impl Config {
    pub fn new(args: &Args, rom: &[u8]) -> Result<Self, Box<dyn Error>> {
        let rng_seed = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
        let mode = Self::mode(rom);
        let frame_time = (args.max_fps > 0).then(|| Duration::from_secs(1) / args.max_fps);
        Ok(Self { rng_seed, mode, frame_time })
    }

    fn mode(rom: &[u8]) -> CGMode {
        match rom[CGB_FLAG as usize] {
            0x80 => CGMode::Enhanced,
            0xC0 => CGMode::Cgb,
            _ => CGMode::Dmg,
        }
    }
}
