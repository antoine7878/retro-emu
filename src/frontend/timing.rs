use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::gameboy::GameBoy;

pub const TURBO_FACTOR: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Speed {
    Normal,
    Fast,
    Unlimited,
}

pub struct Pacer {
    latched: Speed,
    held_fast: bool,
    held_unlimited: bool,
    normal_frame_time: Option<Duration>,
    next_frame: Instant,
}

impl Pacer {
    pub fn new(game_boy: &GameBoy) -> Self {
        Self {
            latched: Speed::Normal,
            held_fast: false,
            held_unlimited: false,
            normal_frame_time: game_boy.config.frame_time,
            next_frame: Instant::now(),
        }
    }

    pub fn speed(&self) -> Speed {
        let held = match (self.held_unlimited, self.held_fast) {
            (true, _) => Speed::Unlimited,
            (false, true) => Speed::Fast,
            (false, false) => Speed::Normal,
        };
        held.max(self.latched)
    }

    pub fn begin_frame(&mut self, game_boy: &GameBoy) {
        self.next_frame += game_boy.config.frame_time.unwrap_or_default();
    }

    pub fn toggle_turbo(&mut self, speed: Speed, game_boy: &mut GameBoy) -> bool {
        let before = self.speed();
        self.latched = if self.latched == Speed::Normal { speed } else { Speed::Normal };
        self.apply(before, game_boy)
    }

    pub fn hold_turbo(&mut self, speed: Speed, held: bool, game_boy: &mut GameBoy) -> bool {
        let before = self.speed();
        match speed {
            Speed::Fast => self.held_fast = held,
            _ => self.held_unlimited = held,
        }
        self.apply(before, game_boy)
    }

    fn apply(&mut self, before: Speed, game_boy: &mut GameBoy) -> bool {
        let speed = self.speed();
        if speed == before {
            return false;
        }
        game_boy.config.frame_time = match speed {
            Speed::Normal => self.normal_frame_time,
            Speed::Fast => self.normal_frame_time.map(|t| t / TURBO_FACTOR),
            Speed::Unlimited => None,
        };
        self.next_frame = Instant::now();
        true
    }

    pub fn throttle(&mut self, game_boy: &GameBoy) {
        let Some(frame_time) = game_boy.config.frame_time else { return };
        let now = Instant::now();
        match self.next_frame.checked_duration_since(now) {
            Some(remaining) => sleep(remaining),
            None if now - self.next_frame > frame_time * 5 => self.next_frame = now,
            None => {}
        }
    }
}
