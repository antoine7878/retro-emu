mod autofire;
mod debug;
mod input;
mod timing;
mod video;

use std::error::Error;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};

use sdl2::Sdl;

use crate::cli::Args;
use crate::debugger::{Debugger, prompt};
use crate::gameboy::GameBoy;
use crate::utils::timestamp;
use debug::{DebugSession, Effect};
use input::{Action, Input};
use timing::Pacer;
use video::Video;

pub const SCALE: u32 = 4;
pub const FRAME_RATE: u32 = 60;
const STATE_DIR: &str = "rscs/state";

pub struct Frontend {
    video: Video,
    input: Input,
    debug: Option<DebugSession>,
}

impl Frontend {
    pub fn new(args: Args, sdl: Sdl) -> Result<Self, Box<dyn Error>> {
        Ok(Self { video: Video::new(&sdl)?, input: Input::new(&sdl)?, debug: DebugSession::new(&args) })
    }

    pub fn run(&mut self, game_boy: &mut GameBoy) -> Result<(), Box<dyn Error>> {
        let mut pacer = Pacer::new(game_boy);
        self.video.set_title(game_boy, pacer.speed(), None);
        game_boy.set_trace(self.debug.is_some());
        while self.handle_input(game_boy, &mut pacer) {
            pacer.begin_frame(game_boy);
            self.process_commands(game_boy);
            catch_unwind(AssertUnwindSafe(|| self.step(game_boy))).map_err(|_| Self::crash(game_boy))?;
            self.input.rumble(game_boy.take_rumble());
            self.video.draw(game_boy)?;
            self.video.update_fps(game_boy, pacer.speed());
            pacer.throttle(game_boy);
        }
        prompt::restore_terminal();
        Ok(())
    }

    fn handle_input(&mut self, game_boy: &mut GameBoy, pacer: &mut Pacer) -> bool {
        for action in self.input.poll() {
            match action {
                Action::Quit => return false,
                Action::Press(button) => game_boy.press(button),
                Action::Release(button) => game_boy.release(button),
                Action::ToggleTurbo(speed) => {
                    if pacer.toggle_turbo(speed, game_boy) {
                        self.video.set_title(game_boy, pacer.speed(), None);
                    }
                }
                Action::HoldTurbo(speed, held) => {
                    if pacer.hold_turbo(speed, held, game_boy) {
                        self.video.set_title(game_boy, pacer.speed(), None);
                    }
                }
                Action::LoadFile(filename) => {
                    if let Err(e) = game_boy.load_file(Some(&filename)) {
                        eprintln!("Failed to load {}: {}", filename, e);
                    }
                }
                Action::SaveState => {
                    let state = game_boy.save_state();
                    let file_name = format!("{}/{}_{}.state", STATE_DIR, timestamp(), game_boy.bus().cartridge().header().rom_name);
                    match fs::create_dir_all(STATE_DIR).and_then(|_| fs::write(&file_name, state)) {
                        Ok(_) => eprintln!("SaveState success full at: {}", file_name),
                        Err(e) => eprintln!("SaveState failed: {}", e),
                    }
                }
            }
        }
        true
    }

    fn process_commands(&mut self, game_boy: &mut GameBoy) {
        let Some(debug) = &mut self.debug else { return };
        while let Some(effect) = debug.next_command(game_boy) {
            match effect {
                Effect::None => {}
                Effect::RaiseWindow => self.video.raise(),
                Effect::Reset => {
                    game_boy.load_rom(game_boy.bus().cartridge().rom());
                    self.video.reset_fps();
                }
            }
        }
    }

    fn step(&mut self, game_boy: &mut GameBoy) {
        match &mut self.debug {
            Some(debug) => debug.step(game_boy),
            None => {
                game_boy.run_frame::<Debugger>(None);
            }
        }
    }

    fn crash(game_boy: &mut GameBoy) -> String {
        game_boy.bus().cartridge().save_ram();
        format!("emulator panicked at PC={:04X}, cycle={}", game_boy.pc(), game_boy.cycle_count())
    }
}
