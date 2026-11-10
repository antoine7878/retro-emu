// ----- Memory Size --------------------

pub const WRAM_SIZE: usize = (WRAM_END - WRAM_START) as usize;
pub const HRAM_SIZE: usize = (HRAM_END - HRAM_START) as usize;
pub const VRAM_SIZE: usize = (VRAM_END - VRAM_START) as usize;
pub const OAM_SIZE: usize = (OAM_END - OAM_START) as usize;
pub const CART_RAM_BANK_SIZE: usize = 1 << 13;
pub const CART_ROM_BANK_SIZE: usize = 1 << 14;

// ----- Memory Map --------------------

pub const ROM_START: u16 = 0x0000;
pub const ROM_HALF: u16 = 0x4000;
pub const ROM_END: u16 = 0x8000;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0xA000;
pub const SRAM_START: u16 = 0xA000;
pub const SRAM_END: u16 = 0xC000;
pub const WRAM_START: u16 = 0xC000;
pub const WRAM_END: u16 = 0xE000;
pub const ECHO_START: u16 = 0xE000;
pub const ECHO_END: u16 = 0xFE00;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFEA0;
pub const NOT_START: u16 = 0xFEA0;
pub const NOT_END: u16 = 0xFF00;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFF;

// ----- IO Ranges --------------------

pub const TIMER_START: u16 = DIV;
pub const TIMER_END: u16 = TAC + 1;
pub const AUDIO_START: u16 = NR10;
pub const AUDIO_END: u16 = NR52 + 1;
pub const WAVE_START: u16 = 0xFF30;
pub const WAVE_END: u16 = 0xFF40;
pub const PPU_START: u16 = LCDC;
pub const PPU_END: u16 = WX + 1;
pub const DMA_START: u16 = HDMA1;
pub const DMA_END: u16 = HDMA5 + 1;
pub const PAL_START: u16 = BCPS;
pub const PAL_END: u16 = OPRI + 1;

pub const U1: u16 = 0xFF03;
pub const U2S: u16 = 0xFF08;
pub const U2E: u16 = 0xFF0F;
pub const U3S: u16 = 0xFF27;
pub const U3E: u16 = 0xFF30;
pub const U5: u16 = 0xFF4E;
pub const U6S: u16 = 0xFF56;
pub const U6E: u16 = 0xFF68;
pub const U7S: u16 = 0xFF6D;
pub const U7E: u16 = 0xFF70;
pub const U8S: u16 = 0xFF71;
pub const U8E: u16 = 0xFF80;

// ----- Joypad --------------------

pub const JOYPAD: u16 = 0xFF00;

// ----- Link --------------------

pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

// ----- Timer --------------------

pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

// ----- Interrupts --------------------

pub const IF: u16 = 0xFF0F;
pub const IE: u16 = 0xFFFF;

// ----- Audio --------------------

pub const NR10: u16 = 0xFF10;
pub const NR11: u16 = 0xFF11;
pub const NR12: u16 = 0xFF12;
pub const NR13: u16 = 0xFF13;
pub const NR14: u16 = 0xFF14;
pub const NR21: u16 = 0xFF16;
pub const NR22: u16 = 0xFF17;
pub const NR23: u16 = 0xFF18;
pub const NR24: u16 = 0xFF19;
pub const NR30: u16 = 0xFF1A;
pub const NR31: u16 = 0xFF1B;
pub const NR32: u16 = 0xFF1C;
pub const NR33: u16 = 0xFF1D;
pub const NR34: u16 = 0xFF1E;
pub const NR41: u16 = 0xFF20;
pub const NR42: u16 = 0xFF21;
pub const NR43: u16 = 0xFF22;
pub const NR44: u16 = 0xFF23;
pub const NR50: u16 = 0xFF24;
pub const NR51: u16 = 0xFF25;
pub const NR52: u16 = 0xFF26;
pub const PCM12: u16 = 0xFF76;
pub const PCM34: u16 = 0xFF77;

// ----- PPU --------------------

pub const LCDC: u16 = 0xFF40;
pub const STAT: u16 = 0xFF41;
pub const SCY: u16 = 0xFF42;
pub const SCX: u16 = 0xFF43;
pub const LY: u16 = 0xFF44;
pub const LYC: u16 = 0xFF45;
pub const DMA: u16 = 0xFF46;
pub const BGP: u16 = 0xFF47;
pub const OBP0: u16 = 0xFF48;
pub const OBP1: u16 = 0xFF49;
pub const WY: u16 = 0xFF4A;
pub const WX: u16 = 0xFF4B;
pub const VBK: u16 = 0xFF4F;
pub const HDMA1: u16 = 0xFF51;
pub const HDMA2: u16 = 0xFF52;
pub const HDMA3: u16 = 0xFF53;
pub const HDMA4: u16 = 0xFF54;
pub const HDMA5: u16 = 0xFF55;
pub const BCPS: u16 = 0xFF68;
pub const BCPD: u16 = 0xFF69;
pub const OCPS: u16 = 0xFF6A;
pub const OCPD: u16 = 0xFF6B;
pub const OPRI: u16 = 0xFF6C;

// ----- CGB System --------------------

pub const KEY0: u16 = 0xFF4C;
pub const KEY1: u16 = 0xFF4D;
pub const BOOT_DISABLE: u16 = 0xFF50;
pub const RP: u16 = 0xFF56;
pub const SVBK: u16 = 0xFF70;

pub fn memory_range(name: &str) -> Option<(u16, u16)> {
    if name.eq_ignore_ascii_case("IO") {
        return None;
    }
    let mut addrs = (0..=u16::MAX).filter(|&addr| memory_location(addr).eq_ignore_ascii_case(name));
    let start = addrs.next()?;
    Some((start, addrs.last().unwrap_or(start)))
}

pub fn memory_location(addr: u16) -> &'static str {
    match addr {
        ROM_START..ROM_END => "ROM",
        VRAM_START..VRAM_END => "VRAM",
        SRAM_START..SRAM_END => "SRAM",
        WRAM_START..WRAM_END => "WRAM",
        ECHO_START..ECHO_END => "ECHO",
        OAM_START..OAM_END => "OAM",
        NOT_START..NOT_END => "UNUSABLE",
        JOYPAD => "JOYP",
        SB => "SB",
        SC => "SC",
        DIV => "DIV",
        TIMA => "TIMA",
        TMA => "TMA",
        TAC => "TAC",
        IF => "IF",
        NR10 => "NR10",
        NR11 => "NR11",
        NR12 => "NR12",
        NR13 => "NR13",
        NR14 => "NR14",
        NR21 => "NR21",
        NR22 => "NR22",
        NR23 => "NR23",
        NR24 => "NR24",
        NR30 => "NR30",
        NR31 => "NR31",
        NR32 => "NR32",
        NR33 => "NR33",
        NR34 => "NR34",
        NR41 => "NR41",
        NR42 => "NR42",
        NR43 => "NR43",
        NR44 => "NR44",
        NR50 => "NR50",
        NR51 => "NR51",
        NR52 => "NR52",
        WAVE_START..WAVE_END => "WAVE",
        LCDC => "LCDC",
        STAT => "STAT",
        SCY => "SCY",
        SCX => "SCX",
        LY => "LY",
        LYC => "LYC",
        DMA => "DMA",
        BGP => "BGP",
        OBP0 => "OBP0",
        OBP1 => "OBP1",
        WY => "WY",
        WX => "WX",
        KEY0 => "KEY0",
        KEY1 => "KEY1",
        VBK => "VBK",
        BOOT_DISABLE => "BOOT",
        HDMA1 => "HDMA1",
        HDMA2 => "HDMA2",
        HDMA3 => "HDMA3",
        HDMA4 => "HDMA4",
        HDMA5 => "HDMA5",
        RP => "RP",
        BCPS => "BCPS",
        BCPD => "BCPD",
        OCPS => "OCPS",
        OCPD => "OCPD",
        OPRI => "OPRI",
        SVBK => "SVBK",
        PCM12 => "PCM12",
        PCM34 => "PCM34",
        HRAM_START..HRAM_END => "HRAM",
        IE => "IE",
        _ => "IO",
    }
}
