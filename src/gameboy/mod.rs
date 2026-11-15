mod cartridge;
mod config;
mod console;
mod cpu;
mod io;
mod memory;
mod monitor;
mod serializer;
mod sram_output;

pub use config::Config;
pub use console::{CYCLES_PER_FRAME, GameBoy};
pub use io::joypad::Button;
pub use io::ppu::{HEIGHT, WIDTH};
pub use memory::map::{memory_location, memory_range};
pub use monitor::{AccessKind, MemAccess, Monitor};

#[macro_export]
macro_rules! save_state_struct {
    ($ty:ty { $($field:ident),* $(,)? }) => {
        impl $crate::gameboy::serializer::SaveState for $ty {
            fn save(&self, w: &mut $crate::gameboy::serializer::StateWriter) {
                $( self.$field.save(w); )*
            }
            fn load(&mut self, r: &mut $crate::gameboy::serializer::StateReader)
                -> Result<(), $crate::gameboy::serializer::SerdeError> {
                $( self.$field.load(r)?; )*
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! save_state_enum {
      ($ty:ident { $($variant:ident),* $(,)? }) => {
          impl $crate::gameboy::serializer::SaveState for $ty {
              fn save(&self, w: &mut $crate::gameboy::serializer::StateWriter) {
                  let tag = match self {
                      $( $ty::$variant => $ty::$variant as u8, )*
                  };
                  $crate::gameboy::serializer::SaveState::save(&tag, w);
              }

          fn load(
              &mut self,
              r: &mut $crate::gameboy::serializer::StateReader,
          ) -> Result<(), $crate::gameboy::serializer::SerdeError> {
              let tag = r.take(1)?[0];
              $(
                  if tag == $ty::$variant as u8 {
                      *self = $ty::$variant;
                      return Ok(());
                  }
              )*
              $crate::gameboy::serializer::SerdeError::new(stringify!($ty), "invalid tag")
          }
      }
  };
  }
