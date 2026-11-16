use crate::gameboy::cpu::registers::{CARRY_FLAG, HALF_CARRY_FLAG, SUB_FLAG, ZERO_FLAG};

#[derive(Clone, Copy, Default)]
pub struct Flags {
    pub z: bool,
    pub n: bool,
    pub h: bool,
    pub c: bool,
}

impl From<Flags> for u8 {
    fn from(f: Flags) -> Self {
        (f.z as u8) << ZERO_FLAG | (f.n as u8) << SUB_FLAG | (f.h as u8) << HALF_CARRY_FLAG | (f.c as u8) << CARRY_FLAG
    }
}

#[derive(Clone, Copy)]
pub enum FlagEffect {
    Result,
    Clear,
    Set,
    Keep,
}

impl FlagEffect {
    const fn from_byte(value: u8) -> Self {
        match value {
            b'R' => Self::Result,
            b'0' => Self::Clear,
            b'1' => Self::Set,
            b'-' => Self::Keep,
            _ => panic!("bad flag effect"),
        }
    }

    pub const fn resolve(self, current: u8, new: bool) -> u8 {
        match self {
            FlagEffect::Result => new as u8,
            FlagEffect::Clear => 0,
            FlagEffect::Set => 1,
            FlagEffect::Keep => current,
        }
    }
}

#[derive(Clone, Copy)]
pub struct FlagSpec(pub [FlagEffect; 4]);

impl FlagSpec {
    pub const fn znhc(s: &str) -> Self {
        let mut ret = [FlagEffect::Keep; 4];
        let s = s.as_bytes();
        let mut i = 0;
        while i < 4 {
            ret[i] = FlagEffect::from_byte(s[i]);
            i += 1;
        }
        Self(ret)
    }
}
