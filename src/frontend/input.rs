use std::error::Error;

use sdl2::controller::{Axis, Button as SdlButton, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::{EventPump, GameControllerSubsystem, Sdl};

use super::autofire::Autofire;
use super::timing::Speed;
use crate::gameboy::Button;

const DEAD_ZONE: i16 = 8000;
const M_DEAD_ZONE: i16 = -DEAD_ZONE;
const TRIGGER_THRESHOLD: i16 = i16::MAX / 2;

pub enum Action {
    Quit,
    ToggleTurbo(Speed),
    HoldTurbo(Speed, bool),
    Press(Button),
    Release(Button),
    LoadFile(String),
    SaveState,
}

impl Action {
    pub fn button(b: Button, pressed: bool) -> Self { if pressed { Self::Press(b) } else { Self::Release(b) } }
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
    autofire_a: Autofire,
    autofire_b: Autofire,
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
            autofire_a: Autofire::new(Button::A),
            autofire_b: Autofire::new(Button::B),
        })
    }

    pub fn poll(&mut self) -> Vec<Action> {
        let mut out = Vec::new();
        let events: Vec<Event> = self.event_pump.poll_iter().collect();
        for event in events {
            match event {
                Event::Quit { .. } => out.push(Action::Quit),
                Event::DropFile { filename, .. } => out.push(Action::LoadFile(filename)),
                Event::KeyDown { scancode: Some(s), repeat: false, .. } => self.on_key(s, true, &mut out),
                Event::KeyUp { scancode: Some(s), .. } => self.on_key(s, false, &mut out),
                Event::ControllerButtonDown { button, .. } => self.on_button(button, true, &mut out),
                Event::ControllerButtonUp { button, .. } => self.on_button(button, false, &mut out),
                Event::ControllerAxisMotion { axis, value, .. } => self.on_axis(axis, value, &mut out),
                Event::ControllerDeviceAdded { which, .. } => {
                    if let Ok(controller) = self.subsystem.open(which) {
                        self.controller = Some(controller);
                        self.last_rumble = 0;
                    }
                }
                Event::ControllerDeviceRemoved { which, .. }
                    if self.controller.as_ref().is_some_and(|c| c.instance_id() == which) =>
                {
                    self.controller = None;
                    for axis in [Axis::LeftX, Axis::LeftY, Axis::TriggerLeft, Axis::TriggerRight] {
                        self.on_axis(axis, 0, &mut out);
                    }
                    self.clear_autofire(&mut out);
                }
                _ => {}
            }
        }
        self.tick_autofire(&mut out);
        out
    }

    fn clear_autofire(&mut self, out: &mut Vec<Action>) {
        self.autofire_a.set(false, out);
        self.autofire_b.set(false, out);
    }

    fn tick_autofire(&mut self, out: &mut Vec<Action>) {
        self.autofire_a.tick(out);
        self.autofire_b.tick(out);
    }

    fn on_key(&mut self, s: Scancode, pressed: bool, out: &mut Vec<Action>) {
        match s {
            Scancode::Escape if pressed => out.push(Action::Quit),
            Scancode::Num1 if pressed => out.push(Action::SaveState),
            Scancode::Space => out.push(Action::HoldTurbo(Speed::Unlimited, pressed)),
            Scancode::M => self.autofire_b.set(pressed, out),
            Scancode::Comma => self.autofire_a.set(pressed, out),
            _ => out.extend(Self::key(s).map(|b| Action::button(b, pressed))),
        }
    }

    fn on_button(&mut self, b: SdlButton, pressed: bool, out: &mut Vec<Action>) {
        match b {
            SdlButton::X => self.autofire_b.set(pressed, out),
            SdlButton::Y => self.autofire_a.set(pressed, out),
            SdlButton::LeftShoulder if pressed => out.push(Action::ToggleTurbo(Speed::Unlimited)),
            SdlButton::RightShoulder if pressed => out.push(Action::ToggleTurbo(Speed::Fast)),
            _ => out.extend(Self::pad(b).map(|b| Action::button(b, pressed))),
        }
    }

    fn on_axis(&mut self, axis: Axis, value: i16, out: &mut Vec<Action>) {
        match axis {
            Axis::TriggerRight => Self::trigger(&mut self.trigger_right, value, Speed::Fast, out),
            Axis::TriggerLeft => Self::trigger(&mut self.trigger_left, value, Speed::Unlimited, out),
            Axis::LeftX => Self::stick(&mut self.stick_x, value, Button::Left, Button::Right, out),
            Axis::LeftY => Self::stick(&mut self.stick_y, value, Button::Up, Button::Down, out),
            _ => {}
        }
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

    fn trigger(held: &mut bool, value: i16, speed: Speed, out: &mut Vec<Action>) {
        let pressed = value > TRIGGER_THRESHOLD;
        if pressed != *held {
            *held = pressed;
            out.push(Action::HoldTurbo(speed, pressed));
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
            Scancode::W => Some(Button::Up),
            Scancode::S => Some(Button::Down),
            Scancode::A => Some(Button::Left),
            Scancode::D => Some(Button::Right),
            Scancode::J => Some(Button::B),
            Scancode::K => Some(Button::A),
            Scancode::U => Some(Button::Select),
            Scancode::I => Some(Button::Start),
            _ => None,
        }
    }
}
