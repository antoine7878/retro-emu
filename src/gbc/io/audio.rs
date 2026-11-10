use crate::gbc::memory::io_reg::IORegister;
use crate::gbc::memory::map::{
    NR10, NR11, NR12, NR13, NR14, NR21, NR22, NR23, NR24, NR30, NR31, NR32, NR33, NR34, NR41, NR42, NR43, NR44, NR50,
    NR51, NR52, PCM12, PCM34,
};

#[derive(Default)]
pub struct Audio {
    pub nr10: IORegister<NR10, 0x7F>,
    pub nr11: IORegister<NR11, 0xFF, 0xC0>,
    pub nr12: IORegister<NR12, 0xFF>,
    pub nr13: IORegister<NR13, 0xFF, 0x00>,
    pub nr14: IORegister<NR14, 0xC7, 0x40>,

    pub nr21: IORegister<NR21, 0xFF, 0xC0>,
    pub nr22: IORegister<NR22, 0xFF>,
    pub nr23: IORegister<NR23, 0xFF, 0x00>,
    pub nr24: IORegister<NR24, 0xC7, 0x40>,

    pub nr30: IORegister<NR30, 0x80>,
    pub nr31: IORegister<NR31, 0xFF, 0x00>,
    pub nr32: IORegister<NR32, 0x60>,
    pub nr33: IORegister<NR33, 0xFF, 0x00>,
    pub nr34: IORegister<NR34, 0xC7, 0x40>,

    pub nr41: IORegister<NR41, 0x3F, 0x00>,
    pub nr42: IORegister<NR42, 0xFF>,
    pub nr43: IORegister<NR43, 0xFF>,
    pub nr44: IORegister<NR44, 0xC0, 0x40>,

    pub nr50: IORegister<NR50, 0xFF>,
    pub nr51: IORegister<NR51, 0xFF>,
    pub nr52: IORegister<NR52, 0x80, 0x8F>,

    pub pcm12: IORegister<PCM12, 0x00, 0xFF>,
    pub pcm34: IORegister<PCM34, 0x00, 0xFF>,

    pub w_ram: [u8; 16],
}
