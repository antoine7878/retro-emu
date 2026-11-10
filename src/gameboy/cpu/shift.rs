use crate::gameboy::cpu::instruction::ShiftOp;

pub fn shift(op: ShiftOp, val: u8, c: u8) -> (u8, bool) {
    match op {
        ShiftOp::Rlc => rlc(val),
        ShiftOp::Rrc => rrc(val),
        ShiftOp::Rl => rl(val, c),
        ShiftOp::Rr => rr(val, c),
        ShiftOp::Sra => sra(val),
        ShiftOp::Srl => srl(val),
        ShiftOp::Sla => sla(val),
        ShiftOp::Swap => swap(val),
    }
}

pub const fn rlc(val: u8) -> (u8, bool) {
    let c = val >> 7;
    (val << 1 | c, c == 1)
}

pub const fn rrc(val: u8) -> (u8, bool) {
    let c = val & 1;
    (val >> 1 | (c << 7), c == 1)
}

pub const fn rl(val: u8, c: u8) -> (u8, bool) { (val << 1 | c, val >> 7 == 1) }

pub const fn rr(val: u8, c: u8) -> (u8, bool) { (val >> 1 | c << 7, val & 1 == 1) }

const fn sla(val: u8) -> (u8, bool) { (val << 1, val >> 7 == 1) }

const fn sra(val: u8) -> (u8, bool) { (val >> 1 | (val & 0x80), val & 1 == 1) }

const fn srl(val: u8) -> (u8, bool) { (val >> 1, val & 1 == 1) }

const fn swap(val: u8) -> (u8, bool) { (val.rotate_right(4), false) }
