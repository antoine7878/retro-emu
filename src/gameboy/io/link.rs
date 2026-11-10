use std::io::Write;

use crate::gameboy::config::ConsoleType;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

#[derive(Default)]
pub struct Link {
    buffer: IORegister<{ map::SB }, 0xFF>,
    ctrl: IORegister<{ map::SC }, 0x83>,
    print: bool,
}

impl Link {
    pub fn set_print(&mut self, on: bool) { self.print = on; }

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
        if self.print {
            let mut out = std::io::stdout();
            let _ = out.write_all(&[self.buffer.read()]);
            let _ = out.flush();
        }
        self.buffer.write(0xFF);
        self.ctrl.clear(7);
        self.ctrl.clear(0);
    }
}
