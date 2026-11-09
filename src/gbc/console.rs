use std::fs::File;
use std::io::{self, BufWriter, Write};

use crate::debugger::{Debugger, StopReason};

use super::bus::Bus;
use super::config::Config;
use super::constants::CYCLES_PER_FRAME;
use super::cpu::Cpu;
use super::instruction::Instruction;

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
    pub config: Config,
    doctor: Option<BufWriter<File>>,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>, config: Config) -> Self {
        Self { cpu: Cpu::default(), bus: Bus::new(rom), config, doctor: None }
    }

    pub fn reset(&mut self) {
        self.bus.reset(&self.config.mode);
        self.cpu.reset(&self.config.mode);
    }

    pub fn run_frame(&mut self, mut dbg: Option<&mut Debugger>) -> Option<StopReason> {
        let start = self.bus.get_cycle_count();
        while self.bus.get_cycle_count() - start < CYCLES_PER_FRAME {
            if !self.cpu.halted(&mut self.bus) {
                if let Some(r) = dbg.as_deref_mut().and_then(|d| d.check(self.cpu.pc())) {
                    return Some(r);
                }
                self.log_doctor();
                self.cpu.dispatch_interrupt(&mut self.bus);
                self.cpu.step(&mut self.bus);
            }
            if let Some(r) = dbg.as_deref_mut().and_then(|d| d.check_accesses(self.bus.drain_accesses())) {
                return Some(r);
            }
        }
        None
    }

    pub fn set_doctor(&mut self, path: &str) -> io::Result<()> {
        self.doctor = Some(BufWriter::new(File::create(path)?));
        self.bus.set_doctor(true);
        Ok(())
    }

    fn log_doctor(&mut self) {
        let Some(w) = self.doctor.as_mut() else { return };
        let pc = self.cpu.pc();
        let m = |o: u16| self.bus.read_internal(pc.wrapping_add(o));
        let line =
            format!("{} PCMEM:{:02X},{:02X},{:02X},{:02X}", self.cpu.registers().doctor(), m(0), m(1), m(2), m(3));
        if let Err(e) = writeln!(w, "{line}") {
            eprintln!("doctor log: {e}");
            self.doctor = None;
        }
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
}
