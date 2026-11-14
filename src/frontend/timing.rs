use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::gameboy::GameBoy;

pub const TURBO_FACTOR: u32 = 2;

pub struct Pacer {
    pub turbo: bool,
    normal_frame_time: Option<Duration>,
    next_frame: Instant,
}

impl Pacer {
    pub fn new(game_boy: &GameBoy) -> Self {
        Self { turbo: false, normal_frame_time: game_boy.config.frame_time, next_frame: Instant::now() }
    }

    pub fn begin_frame(&mut self, game_boy: &GameBoy) {
        self.next_frame += game_boy.config.frame_time.unwrap_or_default();
    }

    pub fn toggle_turbo(&mut self, game_boy: &mut GameBoy) {
        self.turbo = !self.turbo;
        game_boy.config.frame_time = self.normal_frame_time.map(|t| if self.turbo { t / TURBO_FACTOR } else { t });
        self.next_frame = Instant::now();
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
