use crate::gameboy::config::ConsoleType;
use crate::gameboy::io::interrupt::InterruptFlag;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

#[derive(Default)]
pub struct Timer {
    counter: u16,
    tima: IORegister<{ map::TIMA }, 0xFF>,
    tma: IORegister<{ map::TMA }, 0xFF>,
    tac: IORegister<{ map::TAC }, 0x07>,
}

impl Timer {
    const BITS: [u8; 4] = [9, 3, 5, 7];

    pub fn reset(&mut self, mode: &ConsoleType) {
        self.counter = (mode.initial_value(map::DIV) as u16) << 8;
        self.tima.reset(mode);
        self.tma.reset(mode);
        self.tac.reset(mode);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            map::DIV => (self.counter >> 8) as u8,
            map::TIMA => self.tima.read(),
            map::TMA => self.tma.read(),
            map::TAC => self.tac.read_raw(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8, ifl: &mut InterruptFlag) {
        let old = self.signal();
        match addr {
            map::DIV => self.counter = 0,
            map::TIMA => self.tima.write(value),
            map::TMA => self.tma.write(value),
            map::TAC => self.tac.write(value),
            _ => unreachable!(),
        }
        if old && !self.signal() {
            self.tick_tima(ifl);
        }
    }

    fn signal(&self) -> bool {
        let tac = self.tac.read_raw();
        tac & 0b100 != 0 && (self.counter >> Self::BITS[(tac & 0b11) as usize]) & 1 != 0
    }

    pub fn tick(&mut self, ifl: &mut InterruptFlag) {
        let old = self.signal();
        self.counter = self.counter.wrapping_add(4);
        if old && !self.signal() {
            self.tick_tima(ifl);
        }
    }

    fn tick_tima(&mut self, ifl: &mut InterruptFlag) {
        if self.tima.read_raw() == 0xFF {
            self.tima.write_raw(self.tma.read_raw());
            ifl.set(2);
        } else {
            self.tima += 1;
        }
    }
}
