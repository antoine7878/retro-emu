mod argparser;
mod args;
mod debugger;
mod frontend;
mod gbc;
mod prompt;

use std::fs;
use std::sync::atomic::{AtomicU16, Ordering};

use frontend::Frontend;

use args::Args;
use gbc::GameBoy;

use crate::gbc::Config;

pub static PC: AtomicU16 = AtomicU16::new(0x0100);

fn main() {
    if let Err(e) = run() {
        let cycle = PC.load(Ordering::Relaxed);
        eprintln!("error at ${cycle:04X}: {e}");
        std::process::exit(1)
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse()?;
    let sdl = sdl2::init()?;
    let rom_path = args.infiles[0].as_str();
    let rom = fs::read(rom_path).map_err(|e| format!("{rom_path}: {e}"))?;
    let config = Config::new(&args, &rom)?;
    let mut gb = GameBoy::new(rom, config);
    gb.reset();
    let mut frontend = Frontend::new(args, sdl)?;
    frontend.run(&mut gb)
}
