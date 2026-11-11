mod cli;
mod debugger;
mod frontend;
mod gameboy;

use std::fs;

use frontend::Frontend;

use cli::Args;
use gameboy::GameBoy;

use crate::gameboy::Config;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1)
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse()?;
    let sdl = sdl2::init()?;

    let config = Config::new(args.max_fps, args.serial_print)?;
    let mut gb = GameBoy::new(config);

    let rom = match args.infiles.get(0) {
        Some(file) => fs::read(file)?,
        None => include_bytes!("./../rscs/no_game.gb").to_vec(),
    };
    gb.load(rom);

    Frontend::new(args, sdl)?.run(&mut gb)
}
