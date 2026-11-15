use std::fs;

use crate::gameboy::cartridge::{CartridgeHeader, MAGIC_SAVE};
use crate::gameboy::config::Config;
use crate::gameboy::cpu::Cpu;
use crate::gameboy::cpu::instruction::Instruction;
use crate::gameboy::io::joypad::Button;
use crate::gameboy::memory::bus::Bus;
use crate::gameboy::monitor::Monitor;
use crate::gameboy::roms;
use crate::gameboy::sram_output::SramOutput;
use crate::save_state_struct;

use super::serializer::{SaveState, SerdeError, StateReader, StateWriter};

pub const CYCLES_PER_FRAME: u64 = 17556;
pub const DOUBLE_CYCLES_PER_FRAME: u64 = 17556 * 2;

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
    sram_output: SramOutput,
    pub config: Config,
}

save_state_struct!(GameBoy { config, cpu, bus, sram_output });

const MAGIC_STATE: &[u8; 7] = b"GBSTATE";

impl GameBoy {
    pub fn new(config: Config) -> Self {
        let mut bus = Bus::default();
        bus.set_serial_print(config.serial_print);
        let cpu = Cpu::default();
        Self { cpu, bus, sram_output: SramOutput::default(), config }
    }

    pub fn load_file(&mut self, filename: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
        let Some(filename) = filename else {
            self.load_rom(roms::NO_GAME.to_vec());
            return Ok(());
        };
        let data = fs::read(filename)?;
        if data.starts_with(MAGIC_STATE) {
            self.load_state(&data)?;
        } else if data.starts_with(MAGIC_SAVE) {
            self.load_save(&data)?;
        } else if CartridgeHeader::is_rom(&data) {
            self.load_rom(data);
        } else {
            return Err(format!("{filename}: unknown file type").into());
        }
        Ok(())
    }

    pub fn load_save(&mut self, data: &[u8]) -> Result<(), SerdeError> {
        let ram = self.bus.cartridge().parse_save(data)?;
        let rom = self.bus.cartridge().rom();
        self.load_rom(rom);
        self.bus.cartridge_mut().load_ram(ram);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.bus.reset(self.config.model);
        self.cpu.reset();
        self.sram_output.reset();
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.bus.load_rom(self.config.model, rom);
        self.reset();
    }

    pub fn run_frame<M: Monitor>(&mut self, mut dbg: Option<&mut M>) -> Option<M::Stop> {
        let start = self.bus.get_cycle_count();
        let cycle_per_frame = if self.bus.double_speed() { DOUBLE_CYCLES_PER_FRAME } else { CYCLES_PER_FRAME };
        while self.bus.get_cycle_count() - start < cycle_per_frame {
            if self.cpu.stopped(&mut self.bus) {
                break;
            }
            if !self.cpu.halted(&mut self.bus) {
                if let Some(r) = dbg.as_deref_mut().and_then(|d| d.check(self.cpu.pc())) {
                    return Some(r);
                }
                self.cpu.dispatch_interrupt(&mut self.bus);
                self.cpu.step(&mut self.bus);
            }
            if let Some(r) = dbg.as_deref_mut().and_then(|d| d.check_accesses(self.bus.drain_accesses())) {
                return Some(r);
            }
        }
        if self.config.serial_print {
            self.sram_output.poll(&self.bus);
        }
        None
    }

    pub const fn pixels(&self) -> &[u8] { self.bus.get_pixels() }

    pub fn take_redraw(&mut self) -> bool { self.bus.take_redraw() }

    pub fn pc(&self) -> u16 { self.cpu.pc() }

    pub const fn cycle_count(&self) -> u64 { self.bus.get_cycle_count() }

    pub fn disassembly(&self, mut addr: u16, count: usize) -> Vec<(u16, String)> {
        (0..count)
            .map(|_| {
                let (text, len) = Instruction::disassemble(addr, |a| self.bus.read_internal(a));
                let r = (addr, text);
                addr = addr.wrapping_add(len);
                r
            })
            .collect()
    }

    pub fn dump_registers(&self) -> Vec<String> {
        let regs = self.cpu.registers();
        let flag = |bit, name| if bit == 1 { name } else { '-' };
        let flags = String::from_iter([
            flag(regs.get_z(), 'Z'),
            flag(regs.get_n(), 'N'),
            flag(regs.get_h(), 'H'),
            flag(regs.get_c(), 'C'),
        ]);
        vec![
            format!("AF {:04X}  {}", regs.get_af(), flags),
            format!("BC {:04X}", regs.get_bc()),
            format!("DE {:04X}", regs.get_de()),
            format!("HL {:04X}", regs.get_hl()),
            format!("SP {:04X}", regs.get_sp()),
            format!("PC {:04X}", regs.pc()),
        ]
    }

    pub fn set_trace(&mut self, on: bool) { self.bus.set_trace(on); }
    pub fn bus(&self) -> &Bus { &self.bus }
    pub fn press(&mut self, button: Button) { self.bus.press(button); }
    pub fn release(&mut self, button: Button) { self.bus.joypad_mut().release(button); }
    pub fn take_rumble(&mut self) -> f32 { self.bus.cartridge_mut().take_rumble() }

    pub fn save_state(&self) -> Vec<u8> {
        let mut w = StateWriter::default();
        w.write(MAGIC_STATE);
        self.bus.cartridge().rom().save(&mut w);
        self.save(&mut w);
        w.into_bytes()
    }

    pub fn load_state(&mut self, data: &[u8]) -> Result<(), SerdeError> {
        let mut r = StateReader::new(data);
        if r.take(MAGIC_STATE.len())? != MAGIC_STATE {
            return SerdeError::new("header", "not a state file");
        }
        let mut len = 0usize;
        len.load(&mut r)?;
        let rom = r.take(len)?.to_vec();

        let mut next = GameBoy::new(self.config.clone());
        next.load_rom(rom);
        next.load(&mut r)?;

        self.bus.cartridge().save_ram();
        *self = next;
        Ok(())
    }
}
