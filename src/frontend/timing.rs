use std::thread::sleep;
use std::time::{Duration, Instant};

use super::input::Turbo;
use crate::gameboy::GameBoy;

pub const TURBO_FACTOR: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Normal,
    Fast,
    Unlimited,
}

#[derive(Default)]
struct Boost {
    toggled: bool,
    held: bool,
}

impl Boost {
    fn active(&self) -> bool { self.toggled || self.held }
}

pub struct Pacer {
    fast: Boost,
    unlimited: Boost,
    normal_frame_time: Option<Duration>,
    next_frame: Instant,
}

impl Pacer {
    pub fn new(game_boy: &GameBoy) -> Self {
        Self {
            fast: Boost::default(),
            unlimited: Boost::default(),
            normal_frame_time: game_boy.config.frame_time,
            next_frame: Instant::now(),
        }
    }

    pub fn speed(&self) -> Speed {
        match (self.unlimited.active(), self.fast.active()) {
            (true, _) => Speed::Unlimited,
            (false, true) => Speed::Fast,
            (false, false) => Speed::Normal,
        }
    }

    pub fn begin_frame(&mut self, game_boy: &GameBoy) {
        self.next_frame += game_boy.config.frame_time.unwrap_or_default();
    }

    pub fn toggle_turbo(&mut self, turbo: Turbo, game_boy: &mut GameBoy) -> bool {
        let before = self.speed();
        let boost = self.boost(turbo);
        boost.toggled = !boost.toggled;
        self.apply(before, game_boy)
    }

    pub fn hold_turbo(&mut self, turbo: Turbo, held: bool, game_boy: &mut GameBoy) -> bool {
        let before = self.speed();
        self.boost(turbo).held = held;
        self.apply(before, game_boy)
    }

    fn boost(&mut self, turbo: Turbo) -> &mut Boost {
        match turbo {
            Turbo::Fast => &mut self.fast,
            Turbo::Unlimited => &mut self.unlimited,
        }
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
