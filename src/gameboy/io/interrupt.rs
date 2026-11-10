use crate::gameboy::config::ConsoleType;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

#[derive(Default)]
pub struct Interrupter {
    ie: IORegister<{ map::IE }, 0xFF>,
    ifl: IORegister<{ map::IF }, 0x1F>,
}

impl Interrupter {
    pub fn pending(&self) -> u8 { self.ie.read() & self.ifl.read() & 0x1F }
    pub fn ack(&mut self, bit: u8) { self.ifl.clear(bit); }

    pub fn reset(&mut self, mode: &ConsoleType) {
        self.ie.reset(mode);
        self.ifl.reset(mode);
    }

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

    pub fn ifl_mut(&mut self) -> &mut IORegister<{ map::IF }, 0x1F> { &mut self.ifl }
}
