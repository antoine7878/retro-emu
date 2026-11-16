use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;
use crate::save_state_struct;

pub const VBLANK: u8 = 0;
pub const STAT: u8 = 1;
pub const TIMER: u8 = 2;
pub const SERIAL: u8 = 3;
pub const JOYPAD: u8 = 4;

pub type InterruptFlag = IORegister<0x1F>;

#[derive(Default)]
pub struct Interrupter {
    ie: IORegister<0xFF>,
    ifl: InterruptFlag,
}

save_state_struct!(Interrupter { ie, ifl });

impl Interrupter {
    pub fn pending(&self) -> u8 { self.ie.read() & self.ifl.read() & 0x1F }
    pub fn ack(&mut self, bit: u8) { self.ifl.clear(bit); }

    pub fn reset(&mut self) { *self = Self::default(); }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            map::IE => self.ie.read(),
            map::IF => self.ifl.read(),
            _ => unreachable!("Address not in interrupter"),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            map::IE => self.ie.write(value),
            map::IF => self.ifl.write(value),
            _ => unreachable!("Address not in interrupter"),
        }
    }

    pub fn ifl_mut(&mut self) -> &mut InterruptFlag { &mut self.ifl }
}
