use crate::gameboy::config::CGMode;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map::{IE, IF};

#[derive(Default)]
pub struct Interrupter {
    ie: IORegister<IE, 0xFF>,
    ifl: IORegister<IF, 0x1F>,
}

impl Interrupter {
    pub fn pending(&self) -> u8 { self.ie.read() & self.ifl.read() & 0x1F }
    pub fn ack(&mut self, bit: u8) { self.ifl.clear(bit); }

    pub fn reset(&mut self, mode: &CGMode) {
        self.ie.reset(mode);
        self.ifl.reset(mode);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            IE => self.ie.read(),
            IF => self.ifl.read(),
            _ => unreachable!("Address not in interrupter"),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            IE => self.ie.write(value),
            IF => self.ifl.write(value),
            _ => unreachable!("Address not in interrupter"),
        }
    }

    pub fn ifl_mut(&mut self) -> &mut IORegister<IF, 0x1F> { &mut self.ifl }
}
