use std::ops::{AddAssign, BitOr};

use crate::gbc::config::CGMode;

#[derive(Default)]
pub struct IORegister<const ADDR: u16, const WRITE: u8, const READ: u8 = WRITE> {
    raw: u8,
}

impl<const ADDR: u16, const WRITE: u8, const READ: u8> IORegister<ADDR, WRITE, READ> {
    pub fn reset(&mut self, mode: &CGMode) { self.raw = mode.initial_value(ADDR); }
    pub fn set(&mut self, bit: u8) { self.raw |= 1 << bit }
    pub fn clear(&mut self, bit: u8) { self.raw &= !(1 << bit) }
    pub fn is_set(&self, bit: u8) -> bool { self.raw & (1 << bit) > 0 }
    pub fn is_clear(&self, bit: u8) -> bool { self.raw & (1 << bit) == 0 }
    pub fn read_raw(&self) -> u8 { self.raw }
    pub fn write_raw(&mut self, value: u8) { self.raw = value }
}

impl<const ADDR: u16, const WRITE: u8, const READ: u8> AddAssign<u8> for IORegister<ADDR, WRITE, READ> {
    fn add_assign(&mut self, rhs: u8) { self.raw = self.raw.wrapping_add(rhs); }
}

impl<const ADDR: u16, const WRITE: u8, const READ: u8> BitOr<u8> for &IORegister<ADDR, WRITE, READ> {
    type Output = u8;
    fn bitor(self, rhs: u8) -> Self::Output { self.read() | rhs }
}

impl<const ADDR: u16, const WRITE: u8, const READ: u8> PartialEq for IORegister<ADDR, WRITE, READ> {
    fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}

impl<const ADDR: u16, const WRITE: u8, const READ: u8> IOReg for IORegister<ADDR, WRITE, READ> {
    fn read(&self) -> u8 { (self.raw & READ) | !READ }
    fn write(&mut self, value: u8) { self.raw = (self.raw & !WRITE) | (value & WRITE) }
}

pub trait IOReg {
    fn read(&self) -> u8;
    fn write(&mut self, value: u8);
}
