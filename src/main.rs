mod cli;
mod debugger;
mod frontend;
mod gbc;

use std::fs;

use frontend::Frontend;

use cli::Args;
use gbc::GameBoy;

use crate::gbc::Config;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1)
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse()?;
    let sdl = sdl2::init()?;
    let rom_path = args.infiles[0].as_str();
    let rom = fs::read(rom_path).map_err(|e| format!("{rom_path}: {e}"))?;
    let config = Config::new(&rom, args.max_fps)?;
    let mut gb = GameBoy::new(rom, config);
    gb.reset();
    let mut frontend = Frontend::new(args, sdl)?;
    frontend.run(&mut gb)
}
