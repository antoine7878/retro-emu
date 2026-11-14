use std::error::Error;

use sdl2::controller::{Axis, Button as SdlButton, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::{EventPump, GameControllerSubsystem, Sdl};

use crate::gameboy::Button;

const DEAD_ZONE: i16 = 8000;
const M_DEAD_ZONE: i16 = -DEAD_ZONE;

pub enum Action {
    Quit,
    ToggleTurbo,
    Press(Button),
    Release(Button),
}

pub struct Input {
    event_pump: EventPump,
    subsystem: GameControllerSubsystem,
    controller: Option<GameController>,
    stick_x: Option<Button>,
    stick_y: Option<Button>,
    last_rumble: u16,
}

impl Input {
    pub fn new(sdl: &Sdl) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            event_pump: sdl.event_pump()?,
            subsystem: sdl.game_controller()?,
            controller: None,
            stick_x: None,
            stick_y: None,
            last_rumble: 0,
        })
    }

    pub fn poll(&mut self) -> Vec<Action> {
        let mut out = Vec::new();
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { scancode: Some(Scancode::Escape), .. } => out.push(Action::Quit),
                Event::KeyDown { scancode: Some(s), repeat: false, .. } => out.extend(Self::key(s).map(Action::Press)),
                Event::KeyUp { scancode: Some(s), .. } => out.extend(Self::key(s).map(Action::Release)),
                Event::ControllerButtonDown { button: SdlButton::LeftShoulder | SdlButton::RightShoulder, .. } => {
                    out.push(Action::ToggleTurbo)
                }
                Event::ControllerButtonDown { button, .. } => out.extend(Self::pad(button).map(Action::Press)),
                Event::ControllerButtonUp { button, .. } => out.extend(Self::pad(button).map(Action::Release)),
                Event::ControllerAxisMotion { axis: Axis::LeftX, value, .. } => {
                    Self::stick(&mut self.stick_x, value, Button::Left, Button::Right, &mut out)
                }
                Event::ControllerAxisMotion { axis: Axis::LeftY, value, .. } => {
                    Self::stick(&mut self.stick_y, value, Button::Up, Button::Down, &mut out)
                }
                Event::ControllerDeviceAdded { which, .. } => {
                    if let Ok(controller) = self.subsystem.open(which) {
                        self.controller = Some(controller);
                        self.last_rumble = 0;
                    }
                }
                _ => {}
            }
        }
        out
    }

    pub fn rumble(&mut self, level: f32) {
        let Some(c) = &mut self.controller else { return };
        let strength = (level.clamp(0.0, 1.0) * u16::MAX as f32) as u16;
        if strength == self.last_rumble {
            return;
        }
        self.last_rumble = strength;
        let _ = c.set_rumble(strength, strength, if strength == 0 { 0 } else { u32::MAX });
    }

    fn stick(held: &mut Option<Button>, value: i16, neg: Button, pos: Button, out: &mut Vec<Action>) {
        let new = match value {
            DEAD_ZONE.. => Some(pos),
            ..M_DEAD_ZONE => Some(neg),
            _ => None,
        };
        if new == *held {
            return;
        }
        out.extend(held.take().map(Action::Release));
        out.extend(new.map(Action::Press));
        *held = new;
    }

    fn pad(button: SdlButton) -> Option<Button> {
        match button {
            SdlButton::DPadUp => Some(Button::Up),
            SdlButton::DPadDown => Some(Button::Down),
            SdlButton::DPadLeft => Some(Button::Left),
            SdlButton::DPadRight => Some(Button::Right),
            SdlButton::A => Some(Button::B),
            SdlButton::B => Some(Button::A),
            SdlButton::Start => Some(Button::Start),
            SdlButton::Back => Some(Button::Select),
            _ => None,
        }
    }

    fn key(scancode: Scancode) -> Option<Button> {
        match scancode {
            Scancode::E => Some(Button::Up),
            Scancode::D => Some(Button::Down),
            Scancode::S => Some(Button::Left),
            Scancode::F => Some(Button::Right),
            Scancode::J => Some(Button::B),
            Scancode::K => Some(Button::A),
            Scancode::U => Some(Button::Select),
            Scancode::I => Some(Button::Start),
            _ => None,
        }
    }
}
