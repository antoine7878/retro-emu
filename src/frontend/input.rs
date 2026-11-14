use std::error::Error;

use sdl2::controller::{Axis, Button as SdlButton, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::{EventPump, GameControllerSubsystem, Sdl};

use crate::gameboy::Button;

const DEAD_ZONE: i16 = 8000;
const M_DEAD_ZONE: i16 = -DEAD_ZONE;
const TRIGGER_THRESHOLD: i16 = i16::MAX / 2;

#[derive(Clone, Copy)]
pub enum Turbo {
    Fast,
    Unlimited,
}

pub enum Action {
    Quit,
    ToggleTurbo(Turbo),
    HoldTurbo(Turbo, bool),
    Press(Button),
    Release(Button),
}

pub struct Input {
    event_pump: EventPump,
    subsystem: GameControllerSubsystem,
    controller: Option<GameController>,
    stick_x: Option<Button>,
    stick_y: Option<Button>,
    trigger_left: bool,
    trigger_right: bool,
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
            trigger_left: false,
            trigger_right: false,
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
                Event::ControllerButtonDown { button, .. } => out.extend(Self::pad_action(button, true)),
                Event::ControllerButtonUp { button, .. } => out.extend(Self::pad_action(button, false)),
                Event::ControllerAxisMotion { axis: Axis::TriggerRight, value, .. } => {
                    Self::trigger(&mut self.trigger_right, value, Turbo::Fast, &mut out)
                }
                Event::ControllerAxisMotion { axis: Axis::TriggerLeft, value, .. } => {
                    Self::trigger(&mut self.trigger_left, value, Turbo::Unlimited, &mut out)
                }
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

    fn trigger(held: &mut bool, value: i16, turbo: Turbo, out: &mut Vec<Action>) {
        let pressed = value > TRIGGER_THRESHOLD;
        if pressed != *held {
            *held = pressed;
            out.push(Action::HoldTurbo(turbo, pressed));
        }
    }

    fn pad_action(button: SdlButton, pressed: bool) -> Option<Action> {
        match button {
            SdlButton::RightShoulder => pressed.then_some(Action::ToggleTurbo(Turbo::Fast)),
            SdlButton::LeftShoulder => pressed.then_some(Action::ToggleTurbo(Turbo::Unlimited)),
            _ => Self::pad(button).map(if pressed { Action::Press } else { Action::Release }),
        }
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
