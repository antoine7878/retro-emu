mod cli;
mod debugger;
mod frontend;
mod gameboy;

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
    sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");
    let sdl = sdl2::init()?;

    let config = Config::new(args.serial_print)?;
    let mut game_boy = GameBoy::new(config);

    game_boy.load_file(args.infiles.get(0))?;
    let mut front = Frontend::new(args, sdl)?;
    front.run(&mut game_boy)?;
    game_boy.bus().cartridge().save_ram();
    Ok(())
}
