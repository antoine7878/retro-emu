use std::error::Error;
use std::process::Command;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::sleep;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};

use crate::args::Args;
use crate::debugger::{DebugAction, Debugger, PROMPT, StopReason};
use crate::gbc::{CYCLES_PER_FRAME, FRAME_RATE, GameBoy, HEIGHT, SCALE, WIDTH};
use crate::prompt::{self, Prompt};

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

enum Mode {
    Running,
    Tracing(u32),
    Paused,
}

pub struct Frontend {
    event_pump: EventPump,
    canvas: Canvas<Window>,
    texture: Texture,
    mode: Mode,
    commands: Receiver<String>,
    prompt_ready: Sender<()>,
    prompt_pending: bool,
    last_cmd: String,
    dbg: Option<Debugger>,
    terminal: Option<String>,
    fps: FpsCounter,
    dirty: bool,
    last_present: Instant,
}

const PRESENT_INTERVAL: Duration = Duration::from_nanos(1_000_000_000 / FRAME_RATE as u64);

impl Frontend {
    pub fn new(args: Args, sdl: Sdl) -> Result<Self, Box<dyn Error>> {
        let event_pump = sdl.event_pump()?;
        let (canvas, texture) = Self::new_texture(&sdl)?;
        let mode = Mode::Running;
        let dbg = Debugger::new(&args);
        let (commands, prompt_ready) = Self::new_debug_thread();
        let last_cmd = String::new();
        let terminal = std::env::var("__CFBundleIdentifier").ok();
        let prompt_pending = false;
        let fps = FpsCounter::default();
        Ok(Self {
            event_pump,
            canvas,
            texture,
            dbg,
            mode,
            commands,
            prompt_ready,
            prompt_pending,
            last_cmd,
            terminal,
            fps,
            dirty: false,
            last_present: Instant::now(),
        })
    }

    pub fn reset(&mut self) {
        self.mode = Mode::Running;
        self.last_cmd = String::new();
        self.terminal = std::env::var("__CFBundleIdentifier").ok();
        self.fps = FpsCounter::default();
    }

    fn new_texture(sdl: &Sdl) -> Result<(Canvas<Window>, Texture), Box<dyn Error>> {
        let window = sdl
            .video()?
            .window("RetroEmu", WIDTH as u32 * SCALE, HEIGHT as u32 * SCALE)
            .position_centered()
            .always_on_top()
            .build()?;
        let mut canvas = window.into_canvas().build()?;
        canvas.set_logical_size(WIDTH as u32, HEIGHT as u32)?;
        canvas.set_integer_scale(true)?;
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)?;
        Ok((canvas, texture))
    }

    fn new_debug_thread() -> (Receiver<String>, Sender<()>) {
        let (tx, commands) = std::sync::mpsc::channel();
        let (prompt_ready, ready) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut prompt = Prompt::default();
            while ready.recv().is_ok() {
                let Some(line) = prompt.read_line(PROMPT) else { std::process::exit(0) };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        (commands, prompt_ready)
    }

    pub fn run(&mut self, game_boy: &mut GameBoy) -> Result<(), Box<dyn Error>> {
        let rom_name = game_boy.bus().cartridge().header().rom_name.as_str().trim().to_lowercase();
        let _ = self.canvas.window_mut().set_title(&format!("{rom_name}"));
        if self.dbg.is_some() {
            game_boy.set_trace(true);
        }
        let mut next_frame = Instant::now();
        'running: loop {
            next_frame += game_boy.config.frame_time.unwrap_or_default();

            let mut pause_requested = false;
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } | Event::KeyDown { scancode: Some(Scancode::Escape), .. } => break 'running,
                    Event::KeyDown { scancode: Some(Scancode::P), repeat: false, .. } => pause_requested = true,
                    _ => {}
                }
            }
            if pause_requested && self.dbg.is_some() && !matches!(self.mode, Mode::Paused) {
                self.stop(StopReason::Pause, game_boy);
            }

            while let Ok(mut cmd) = self.commands.try_recv() {
                self.prompt_pending = false;
                if cmd.is_empty() {
                    if let Mode::Tracing(_) = self.mode {
                        self.pause(game_boy);
                        continue;
                    }
                    cmd = self.last_cmd.clone();
                }
                self.handle_command(&cmd, game_boy);
            }

            match self.mode {
                Mode::Running => {
                    if let Some(reason) = game_boy.run_frame(self.dbg.as_mut()) {
                        self.stop(reason, game_boy);
                    }
                }
                Mode::Tracing(n) => {
                    if let Some(dbg) = self.dbg.as_mut() {
                        dbg.set_step_budget(n);
                    }
                    match game_boy.run_frame(self.dbg.as_mut()) {
                        None | Some(StopReason::Step) => {
                            if let Some(dbg) = &self.dbg {
                                dbg.draw_live(game_boy);
                            }
                        }
                        Some(reason) => self.stop(reason, game_boy),
                    }
                }
                Mode::Paused => {}
            }

            self.draw(game_boy)?;

            if let Some((emu, lcd)) = self.fps.tick(game_boy.cycle_count()) {
                let rom_name = game_boy.bus().cartridge().header().rom_name.as_str().trim().to_lowercase();
                let _ = self.canvas.window_mut().set_title(&format!("{rom_name} - {emu:.0} fps ({lcd:.1})"));
            }

            if let Some(frame_time) = game_boy.config.frame_time {
                let now = Instant::now();
                match next_frame.checked_duration_since(now) {
                    Some(remaining) => sleep(remaining),
                    None if now - next_frame > frame_time * 5 => next_frame = now,
                    None => {}
                }
            }
        }
        prompt::restore_terminal();
        Ok(())
    }

    fn draw(&mut self, game_boy: &mut GameBoy) -> Result<(), Box<dyn Error>> {
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

    fn handle_command(&mut self, cmd: &str, game_boy: &mut GameBoy) {
        let Some(dbg) = self.dbg.as_mut() else { return };
        self.last_cmd = cmd.to_string();
        match dbg.handle_command(cmd) {
            DebugAction::Continue => {
                self.mode = Mode::Running;
                self.canvas.window_mut().raise();
            }
            DebugAction::Step => self.mode = Mode::Running,
            DebugAction::Trace(n) => {
                self.mode = Mode::Tracing(n);
                self.show_prompt(game_boy);
            }
            DebugAction::Stay => self.pause(game_boy),
            DebugAction::Reset => {
                game_boy.reset();
                self.reset();
            }
        }
    }
    fn focus_terminal(&self) {
        if let Some(id) = &self.terminal {
            let _ = Command::new("open").args(["-b", id]).status();
        }
    }

    fn stop(&mut self, reason: StopReason, game_boy: &GameBoy) {
        if let Some(dbg) = self.dbg.as_mut() {
            dbg.log(reason.to_string());
        }
        self.focus_terminal();
        self.pause(game_boy);
    }

    fn pause(&mut self, game_boy: &GameBoy) {
        self.mode = Mode::Paused;
        self.show_prompt(game_boy);
    }

    fn show_prompt(&mut self, game_boy: &GameBoy) {
        let Some(dbg) = &self.dbg else { return };
        if self.prompt_pending {
            dbg.draw_live(game_boy);
        } else {
            dbg.draw(game_boy);
            let _ = self.prompt_ready.send(());
            self.prompt_pending = true;
        }
    }
}
