use crate::gameboy::config::ConsoleType;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

#[derive(Default)]
pub struct Timer {
    div: IORegister<{ map::DIV }, 0xFF>,
    tima: IORegister<{ map::TIMA }, 0xFF>,
    tma: IORegister<{ map::TMA }, 0xFF>,
    tac: IORegister<{ map::TAC }, 0x07>,
    tima_clock: u8,
    div_clock: u8,
}

impl Timer {
    pub fn reset(&mut self, mode: &ConsoleType) {
        self.div.reset(mode);
        self.tima.reset(mode);
        self.tma.reset(mode);
        self.tac.reset(mode);
        self.div_clock = 0;
        self.reset_tima_clock();
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            map::DIV => self.div.read(),
            map::TIMA => self.tima.read(),
            map::TMA => self.tma.read(),
            map::TAC => self.tac.read_raw(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            map::DIV => {
                self.div.write_raw(0);
                self.div_clock = 0;
            }
            map::TIMA => self.tima.write(value),
            map::TMA => self.tma.write(value),
            map::TAC => {
                self.tac.write(value);
                self.reset_tima_clock();
            }
            _ => unreachable!(),
        }
    }

    const CLOCKS: [u8; 4] = [255, 3, 15, 63];
    fn reset_tima_clock(&mut self) { self.tima_clock = Self::CLOCKS[self.tac.read_raw() as usize & 0b11]; }

    pub fn tick(&mut self, ifl: &mut IORegister<{ map::IF }, 0x1F>) {
        self.tick_div();
        if self.tac.read_raw() & 0b100 == 0 {
            return;
        }
        if self.tima_clock == 0 {
            self.tick_tima(ifl);
            self.reset_tima_clock();
        } else {
            self.tima_clock -= 1;
        }
    }

    fn tick_div(&mut self) {
        if self.div_clock == 63 {
            self.div += 1;
            self.div_clock = 0;
        } else {
            self.div_clock += 1;
        }
    }

    fn tick_tima(&mut self, ifl: &mut IORegister<{ map::IF }, 0x1F>) {
        if self.tima.read_raw() == 0xFF {
            self.tima.write_raw(self.tma.read_raw());
            ifl.set(2);
        } else {
            self.tima += 1;
        }
    }
}
