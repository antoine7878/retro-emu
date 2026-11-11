use std::io::Write;

use crate::gameboy::memory::bus::Bus;
use crate::gameboy::memory::map;

const STATUS: u16 = map::SRAM_START;
const SIGNATURE_ADDR: u16 = map::SRAM_START + 1;
const SIGNATURE: [u8; 3] = [0xDE, 0xB0, 0x61];
const TEXT_START: u16 = map::SRAM_START + 4;
const RUNNING: u8 = 0x80;

#[derive(Default)]
pub struct SramOutput {
    printed: u16,
    done: bool,
}

impl SramOutput {
    pub fn reset(&mut self) { *self = Self::default(); }

    pub fn poll(&mut self, bus: &Bus) {
        if self.done || !Self::has_signature(bus) {
            return;
        }
        let mut text = Vec::new();
        let mut addr = TEXT_START + self.printed;
        while addr < map::SRAM_END {
            let byte = bus.read_internal(addr);
            if byte == 0 {
                break;
            }
            text.push(byte);
            addr += 1;
        }
        self.printed += text.len() as u16;
        if !text.is_empty() {
            let mut out = std::io::stdout();
            let _ = out.write_all(&text);
            let _ = out.flush();
        }
        self.done = bus.read_internal(STATUS) != RUNNING;
    }

    fn has_signature(bus: &Bus) -> bool {
        (0..SIGNATURE.len() as u16).all(|i| bus.read_internal(SIGNATURE_ADDR + i) == SIGNATURE[i as usize])
    }
}
