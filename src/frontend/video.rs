use std::error::Error;
use std::time::{Duration, Instant};

use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use super::timing::TURBO_FACTOR;
use super::{FRAME_RATE, SCALE};
use crate::gameboy::{CYCLES_PER_FRAME, GameBoy, HEIGHT, WIDTH};

const PRESENT_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / FRAME_RATE as u64);

struct FpsCounter {
    window_start: Instant,
    start_cycle: u64,
    presented: u32,
}

impl Default for FpsCounter {
    fn default() -> Self { Self { window_start: Instant::now(), start_cycle: 0, presented: 0 } }
}

impl FpsCounter {
    fn tick(&mut self, cycle: u64) -> Option<(f64, f64)> {
        let elapsed = self.window_start.elapsed();
        if elapsed < Duration::from_secs(1) {
            return None;
        }
        let secs = elapsed.as_secs_f64();
        let frames = cycle.saturating_sub(self.start_cycle) as f64 / CYCLES_PER_FRAME as f64;
        let fps = (frames / secs, self.presented as f64 / secs);
        *self = Self { start_cycle: cycle, ..Self::default() };
        Some(fps)
    }
}

pub struct Video {
    canvas: Canvas<Window>,
    texture: Texture,
    fps: FpsCounter,
    dirty: bool,
    last_present: Instant,
}

impl Video {
    pub fn new(sdl: &Sdl) -> Result<Self, Box<dyn Error>> {
        let (w, h) = (WIDTH as u32, HEIGHT as u32);
        let window = sdl.video()?.window("RetroEmu", w * SCALE, h * SCALE).position_centered().always_on_top().build()?;
        let mut canvas = window.into_canvas().present_vsync().build()?;
        canvas.set_logical_size(w, h)?;
        canvas.set_integer_scale(true)?;
        let texture = canvas.texture_creator().create_texture_streaming(PixelFormatEnum::RGB24, w, h)?;
        Ok(Self { canvas, texture, fps: FpsCounter::default(), dirty: false, last_present: Instant::now() })
    }

    pub fn reset_fps(&mut self) { self.fps = FpsCounter::default(); }

    pub fn raise(&mut self) { self.canvas.window_mut().raise(); }

    pub fn set_title(&mut self, game_boy: &GameBoy, turbo: bool, fps: Option<(f64, f64)>) {
        let rom = game_boy.bus().cartridge().header().rom_name.as_str().trim().to_lowercase();
        let speed = if turbo { format!(" x{TURBO_FACTOR}") } else { String::new() };
        let title = match fps {
            Some((emu, lcd)) => format!("{rom} - {emu:.0} fps ({lcd:.1}){speed}"),
            None => format!("{rom}{speed}"),
        };
        let _ = self.canvas.window_mut().set_title(&title);
    }

    pub fn update_fps(&mut self, game_boy: &GameBoy, turbo: bool) {
        if let Some(fps) = self.fps.tick(game_boy.cycle_count()) {
            self.set_title(game_boy, turbo, Some(fps));
        }
    }

    pub fn draw(&mut self, game_boy: &mut GameBoy) -> Result<(), Box<dyn Error>> {
        self.dirty |= game_boy.take_redraw();
        let throttled = game_boy.config.frame_time.is_none_or(|t| t < PRESENT_INTERVAL);
        if !self.dirty || (throttled && self.last_present.elapsed() < PRESENT_INTERVAL) {
            return Ok(());
        }
        self.texture.update(None, game_boy.pixels(), WIDTH * 3)?;
        self.canvas.copy(&self.texture, None, None)?;
        self.canvas.present();
        self.fps.presented += 1;
        self.dirty = false;
        self.last_present = Instant::now();
        Ok(())
    }
}
