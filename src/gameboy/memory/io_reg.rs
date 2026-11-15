use std::ops::AddAssign;

use crate::gameboy::serializer::{SaveState, SerdeError, StateReader, StateWriter};

#[derive(Default)]
pub struct IORegister<const WRITE: u8, const READ: u8 = WRITE> {
    raw: u8,
}

impl<const WRITE: u8, const READ: u8> SaveState for IORegister<WRITE, READ> {
    fn save(&self, w: &mut StateWriter) { w.write(&[self.raw]); }

    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        self.raw = r.take(1)?[0];
        Ok(())
    }
}

impl<const WRITE: u8, const READ: u8> IORegister<WRITE, READ> {
    pub fn reset(&mut self) { *self = Self::default(); }
    pub fn set(&mut self, bit: u8) { self.raw |= 1 << bit }
    pub fn clear(&mut self, bit: u8) { self.raw &= !(1 << bit) }
    pub fn is_set(&self, bit: u8) -> bool { self.raw & (1 << bit) > 0 }
    pub fn is_clear(&self, bit: u8) -> bool { self.raw & (1 << bit) == 0 }
    pub fn read_raw(&self) -> u8 { self.raw }
    pub fn write_raw(&mut self, value: u8) { self.raw = value }

    pub fn write_bit(&mut self, bit: u8, value: bool) { if value { self.set(bit) } else { self.clear(bit) } }
}

impl<const WRITE: u8, const READ: u8> AddAssign<u8> for IORegister<WRITE, READ> {
    fn add_assign(&mut self, rhs: u8) { self.raw = self.raw.wrapping_add(rhs); }
}

impl<const WRITE: u8, const READ: u8> IOReg for IORegister<WRITE, READ> {
    fn read(&self) -> u8 { (self.raw & READ) | !READ }
    fn write(&mut self, value: u8) { self.raw = (self.raw & !WRITE) | (value & WRITE) }
}

pub trait IOReg {
    fn read(&self) -> u8;
    fn write(&mut self, value: u8);
}
