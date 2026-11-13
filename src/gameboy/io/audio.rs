#![allow(unused)]

use crate::gameboy::memory::io_reg::IORegister;
use crate::gameboy::memory::map;

#[derive(Default)]
pub struct Audio {
    pub nr10: IORegister<{ map::NR10 }, 0x7F>,
    pub nr11: IORegister<{ map::NR11 }, 0xFF, 0xC0>,
    pub nr12: IORegister<{ map::NR12 }, 0xFF>,
    pub nr13: IORegister<{ map::NR13 }, 0xFF, 0x00>,
    pub nr14: IORegister<{ map::NR14 }, 0xC7, 0x40>,

    pub nr21: IORegister<{ map::NR21 }, 0xFF, 0xC0>,
    pub nr22: IORegister<{ map::NR22 }, 0xFF>,
    pub nr23: IORegister<{ map::NR23 }, 0xFF, 0x00>,
    pub nr24: IORegister<{ map::NR24 }, 0xC7, 0x40>,

    pub nr30: IORegister<{ map::NR30 }, 0x80>,
    pub nr31: IORegister<{ map::NR31 }, 0xFF, 0x00>,
    pub nr32: IORegister<{ map::NR32 }, 0x60>,
    pub nr33: IORegister<{ map::NR33 }, 0xFF, 0x00>,
    pub nr34: IORegister<{ map::NR34 }, 0xC7, 0x40>,

    pub nr41: IORegister<{ map::NR41 }, 0x3F, 0x00>,
    pub nr42: IORegister<{ map::NR42 }, 0xFF>,
    pub nr43: IORegister<{ map::NR43 }, 0xFF>,
    pub nr44: IORegister<{ map::NR44 }, 0xC0, 0x40>,

    pub nr50: IORegister<{ map::NR50 }, 0xFF>,
    pub nr51: IORegister<{ map::NR51 }, 0xFF>,
    pub nr52: IORegister<{ map::NR52 }, 0x80, 0x8F>,

    pub pcm12: IORegister<{ map::PCM12 }, 0x00, 0xFF>,
    pub pcm34: IORegister<{ map::PCM34 }, 0x00, 0xFF>,

    pub w_ram: [u8; 16],
}
