use crate::gameboy::config::ConsoleType;
use crate::gameboy::memory::io_reg::IOReg;
use crate::gameboy::memory::map;

const UNUSED_BITS: u8 = 0b1100_0000;
const SELECT_BUTTONS: u8 = 0b0010_0000;
const SELECT_DPAD: u8 = 0b0001_0000;
const SELECT_MASK: u8 = SELECT_BUTTONS | SELECT_DPAD;
const LINE_MASK: u8 = 0b0000_1111;
const ALL_RELEASED: u8 = LINE_MASK;

const LINE_0: u8 = 0b0000_0001;
const LINE_1: u8 = 0b0000_0010;
const LINE_2: u8 = 0b0000_0100;
const LINE_3: u8 = 0b0000_1000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Start,
    Select,
    A,
    B,
    Down,
    Up,
    Left,
    Right,
}

impl Button {
    fn mask(&self) -> u8 {
        match self {
            Button::Start => SELECT_BUTTONS | LINE_3,
            Button::Select => SELECT_BUTTONS | LINE_2,
            Button::B => SELECT_BUTTONS | LINE_1,
            Button::A => SELECT_BUTTONS | LINE_0,
            Button::Down => SELECT_DPAD | LINE_3,
            Button::Up => SELECT_DPAD | LINE_2,
            Button::Left => SELECT_DPAD | LINE_1,
            Button::Right => SELECT_DPAD | LINE_0,
        }
    }

    fn line(&self) -> u8 { self.mask() & LINE_MASK }

    fn is_action(&self) -> bool { self.mask() & SELECT_BUTTONS != 0 }
}

pub struct Joypad {
    select: u8,
    dpad: u8,
    buttons: u8,
}

impl IOReg for Joypad {
    fn read(&self) -> u8 {
        let mut lines = ALL_RELEASED;
        if self.select & SELECT_DPAD == 0 {
            lines &= self.dpad;
        }
        if self.select & SELECT_BUTTONS == 0 {
            lines &= self.buttons;
        }
        UNUSED_BITS | self.select | lines
    }

    fn write(&mut self, value: u8) { self.select = value & SELECT_MASK; }
}

impl Default for Joypad {
    fn default() -> Self { Self { select: SELECT_MASK, dpad: ALL_RELEASED, buttons: ALL_RELEASED } }
}

impl Joypad {
    pub fn reset(&mut self, mode: &ConsoleType) {
        self.select = mode.initial_value(map::JOYPAD) & SELECT_MASK;
        self.dpad = ALL_RELEASED;
        self.buttons = ALL_RELEASED;
    }

    fn group(&mut self, button: Button) -> &mut u8 {
        if button.is_action() { &mut self.buttons } else { &mut self.dpad }
    }

    pub fn press(&mut self, button: Button) -> bool {
        let before = self.read();
        *self.group(button) &= !button.line();
        before & !self.read() & LINE_MASK != 0
    }
    pub fn release(&mut self, button: Button) { *self.group(button) |= button.line(); }
}
