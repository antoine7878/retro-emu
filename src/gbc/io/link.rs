use crate::gbc::config::CGMode;
use crate::gbc::memory::io_reg::{IOReg, IORegister};
use crate::gbc::memory::map::{SB, SC};

#[derive(Default)]
pub struct Link {
    buffer: IORegister<SB, 0xFF>,
    ctrl: IORegister<SC, 0x83>,
}

impl Link {
    pub fn init_reg(&mut self, mode: &CGMode) {
        self.buffer.reset(mode);
        self.ctrl.reset(mode);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            SB => self.buffer.read(),
            SC => self.ctrl.read(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            SB => self.buffer.write(value),
            SC => {
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
