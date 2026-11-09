use super::flag::{FlagSpec, Flags};
use super::instruction::AluOp;

impl AluOp {
    pub const fn flag_spec(&self) -> FlagSpec {
        match self {
            AluOp::Add | AluOp::Adc => FlagSpec::znhc("R0RR"),
            AluOp::Sub | AluOp::Sbc | AluOp::Cp => FlagSpec::znhc("R1RR"),
            AluOp::Xor | AluOp::Or => FlagSpec::znhc("R000"),
            AluOp::And => FlagSpec::znhc("R010"),
        }
    }

    pub const fn operation(&self) -> fn(u8, u8, bool) -> (u8, Flags) {
        match self {
            AluOp::Add => |a, b, _| add8(a, b, false),
            AluOp::Sub => |a, b, _| sub8(a, b, false),
            AluOp::Adc => |a, b, c| add8(a, b, c),
            AluOp::Sbc => |a, b, c| sub8(a, b, c),
            AluOp::Xor => |a, b, _| xor(a, b),
            AluOp::Or => |a, b, _| or(a, b),
            AluOp::And => |a, b, _| and(a, b),
            AluOp::Cp => |a, b, _| (a, sub8(a, b, false).1),
        }
    }
}

pub fn add8(lhs: u8, rhs: u8, carry: bool) -> (u8, Flags) {
    let (lhs, rhs) = (lhs as u16, rhs as u16);
    let r = lhs.wrapping_add(rhs).wrapping_add(carry as u16);
    let x = lhs ^ rhs ^ r;
    let result = r as u8;
    let flags = Flags { z: result == 0, n: false, h: x & 0x10 != 0, c: x & 0x100 != 0 };
    (result, flags)
}

pub fn sub8(lhs: u8, rhs: u8, carry: bool) -> (u8, Flags) {
    let (lhs, rhs) = (lhs as u16, rhs as u16);
    let r = lhs.wrapping_sub(rhs).wrapping_sub(carry as u16);
    let x = lhs ^ rhs ^ r;
    let result = r as u8;
    let flags = Flags { z: result == 0, n: false, h: x & 0x10 != 0, c: x & 0x100 != 0 };
    (result, flags)
}

pub fn xor(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs ^ rhs;
    let flags = Flags { z: result == 0, n: false, h: false, c: false };
    (result, flags)
}

pub fn or(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs | rhs;
    let flags = Flags { z: result == 0, n: false, h: false, c: false };
    (result, flags)
}

pub fn and(lhs: u8, rhs: u8) -> (u8, Flags) {
    let result = lhs & rhs;
    let flags = Flags { z: result == 0, n: false, h: true, c: false };
    (result, flags)
}

pub fn add16(lhs: u16, rhs: u16, carry: bool) -> (u16, Flags) {
    let (lhs, rhs) = (lhs as u32, rhs as u32);
    let r = lhs.wrapping_add(rhs).wrapping_add(carry as u32);
    let x = lhs ^ rhs ^ r;
    let result = r as u16;
    let flags = Flags { z: result == 0, n: false, h: x & 0x1000 != 0, c: x & 0x10000 != 0 };
    (result, flags)
}

pub fn add_sp_e8(sp: u16, e: i8) -> (u16, Flags) {
    let rhs = e as i16 as u16;
    let result = sp.wrapping_add(rhs);
    let x = sp ^ rhs ^ result;
    (result, Flags { z: false, n: false, h: x & 0x10 != 0, c: x & 0x100 != 0 })
}

pub fn sub16(lhs: u16, rhs: u16, carry: bool) -> (u16, Flags) {
    let (lhs, rhs) = (lhs as u32, rhs as u32);
    let r = lhs.wrapping_sub(rhs).wrapping_sub(carry as u32);
    let x = lhs ^ rhs ^ r;
    let result = r as u16;
    let flags = Flags { z: result == 0, n: false, h: x & 0x1000 != 0, c: x & 0x10000 != 0 };
    (result, flags)
}
