use super::FRAME_RATE;
use super::input::Action;
use crate::gameboy::Button;

const AUTOFIRE_RATE: u32 = 10;
const AUTOFIRE_PERIOD: u32 = FRAME_RATE / AUTOFIRE_RATE;
const AUTOFIRE_RELEASE: u32 = AUTOFIRE_PERIOD / 2;

pub struct Autofire {
    button: Button,
    held: bool,
    frame: u32,
}

impl Autofire {
    pub fn new(button: Button) -> Self { Self { button, held: false, frame: 0 } }

    pub fn set(&mut self, held: bool, out: &mut Vec<Action>) {
        if held == self.held {
            return;
        }
        self.held = held;
        self.frame = 0;
        out.push(Action::button(self.button, held));
    }

    pub fn tick(&mut self, out: &mut Vec<Action>) {
        if !self.held {
            return;
        }
        self.frame = (self.frame + 1) % AUTOFIRE_PERIOD;
        match self.frame {
            0 => out.push(Action::Press(self.button)),
            AUTOFIRE_RELEASE => out.push(Action::Release(self.button)),
            _ => {}
        }
    }
}
