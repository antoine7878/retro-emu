use std::error::Error;
use std::time::Duration;

use crate::cli::ArgError;
use crate::frontend::FRAME_RATE;
use crate::{save_state_enum, save_state_struct};

use super::roms;

#[derive(Debug, Clone)]
pub struct Config {
    pub model: Model,
    pub frame_time: Option<Duration>,
    pub serial_print: bool,
}

save_state_struct!(Config { model });

impl Config {
    pub fn new(mode: Model, serial_print: bool) -> Result<Self, Box<dyn Error>> {
        let frame_time = (!serial_print).then(|| Duration::from_secs(1) / FRAME_RATE);
        Ok(Self { model: mode, frame_time, serial_print })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    Dmg,
    Cgb,
}

save_state_enum!(Model { Dmg, Cgb });

impl Model {
    pub fn boot_rom(&self) -> &'static [u8] {
        match self {
            Model::Dmg => roms::DMG_BOOT,
            Model::Cgb => roms::CGB_BOOT,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CgbFlag {
    #[default]
    Dmg,
    Enhanced,
    Cgb,
}

save_state_enum!(CgbFlag { Dmg, Enhanced, Cgb });

impl TryFrom<String> for Model {
    type Error = ArgError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "dmg" => Ok(Model::Dmg),
            "cgb" | "gbc" => Ok(Model::Cgb),
            _ => Err(ArgError::WrongValue('m', value)),
        }
    }
}
