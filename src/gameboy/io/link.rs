use crate::gameboy::config::ConsoleType;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

#[derive(Default)]
pub struct Link {
    buffer: IORegister<{ map::SB }, 0xFF>,
    ctrl: IORegister<{ map::SC }, 0x83>,
}

impl Link {
    pub fn init_reg(&mut self, mode: &ConsoleType) {
        self.buffer.reset(mode);
        self.ctrl.reset(mode);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            map::SB => self.buffer.read(),
            map::SC => self.ctrl.read(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            map::SB => self.buffer.write(value),
            map::SC => {
                self.ctrl.write(value);
                if self.ctrl.is_set(7) && self.ctrl.is_set(0) {
                    self.transfer();
                }
            }
            _ => unreachable!(),
        }
    }

    fn transfer(&mut self) {
        self.buffer.write(0xFF);
        self.ctrl.clear(7);
        self.ctrl.clear(0);
    }
}
