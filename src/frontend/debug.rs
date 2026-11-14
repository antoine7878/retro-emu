use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::cli::Args;
use crate::debugger::prompt::Prompt;
use crate::debugger::{DebugAction, Debugger, PROMPT, StopReason};
use crate::gameboy::GameBoy;

pub enum Effect {
    None,
    RaiseWindow,
    Reset,
}

enum Mode {
    Running,
    Tracing(u32),
    Paused,
}

pub struct DebugSession {
    dbg: Debugger,
    mode: Mode,
    commands: Receiver<String>,
    prompt_ready: Sender<()>,
    prompt_pending: bool,
    last_cmd: String,
    terminal: Option<String>,
}

impl DebugSession {
    pub fn new(args: &Args) -> Option<Self> {
        let dbg = Debugger::new(args)?;
        let (commands, prompt_ready) = spawn_prompt_thread();
        Some(Self {
            dbg,
            mode: Mode::Running,
            commands,
            prompt_ready,
            prompt_pending: false,
            last_cmd: String::new(),
            terminal: terminal_id(),
        })
    }

    fn reset(&mut self) {
        self.mode = Mode::Running;
        self.last_cmd.clear();
        self.terminal = terminal_id();
    }

    pub fn request_pause(&mut self, game_boy: &GameBoy) {
        if !matches!(self.mode, Mode::Paused) {
            self.stop(StopReason::Pause, game_boy);
        }
    }

    pub fn next_command(&mut self, game_boy: &GameBoy) -> Option<Effect> {
        let cmd = self.commands.try_recv().ok()?;
        self.prompt_pending = false;
        if cmd.is_empty() && matches!(self.mode, Mode::Tracing(_)) {
            self.pause(game_boy);
            return Some(Effect::None);
        }
        if !cmd.is_empty() {
            self.last_cmd = cmd;
        }
        Some(self.handle_command(game_boy))
    }

    fn handle_command(&mut self, game_boy: &GameBoy) -> Effect {
        match self.dbg.handle_command(&self.last_cmd) {
            DebugAction::Continue => {
                self.mode = Mode::Running;
                return Effect::RaiseWindow;
            }
            DebugAction::Step => self.mode = Mode::Running,
            DebugAction::Trace(n) => {
                self.mode = Mode::Tracing(n);
                self.show_prompt(game_boy);
            }
            DebugAction::Stay => self.pause(game_boy),
            DebugAction::Reset => {
                self.reset();
                return Effect::Reset;
            }
        }
        Effect::None
    }

    pub fn step(&mut self, game_boy: &mut GameBoy) {
        match self.mode {
            Mode::Running => {
                if let Some(reason) = game_boy.run_frame(Some(&mut self.dbg)) {
                    self.stop(reason, game_boy);
                }
            }
            Mode::Tracing(budget) => {
                self.dbg.set_step_budget(budget);
                match game_boy.run_frame(Some(&mut self.dbg)) {
                    None | Some(StopReason::Step) => self.dbg.draw_live(game_boy),
                    Some(reason) => self.stop(reason, game_boy),
                }
            }
            Mode::Paused => {}
        }
    }

    fn stop(&mut self, reason: StopReason, game_boy: &GameBoy) {
        self.dbg.log(reason.to_string());
        if let Some(id) = &self.terminal {
            let _ = Command::new("open").args(["-b", id]).status();
        }
        self.pause(game_boy);
    }

    fn pause(&mut self, game_boy: &GameBoy) {
        self.mode = Mode::Paused;
        self.show_prompt(game_boy);
    }

    fn show_prompt(&mut self, game_boy: &GameBoy) {
        if self.prompt_pending {
            self.dbg.draw_live(game_boy);
        } else {
            self.dbg.draw(game_boy);
            let _ = self.prompt_ready.send(());
            self.prompt_pending = true;
        }
    }
}

fn terminal_id() -> Option<String> { std::env::var("__CFBundleIdentifier").ok() }

fn spawn_prompt_thread() -> (Receiver<String>, Sender<()>) {
    let (tx, commands) = mpsc::channel();
    let (prompt_ready, ready) = mpsc::channel();
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
