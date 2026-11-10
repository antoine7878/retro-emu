use crate::gameboy::config::Config;
use crate::gameboy::cpu::Cpu;
use crate::gameboy::cpu::instruction::Instruction;
use crate::gameboy::memory::bus::Bus;
use crate::gameboy::monitor::Monitor;

pub const CYCLES_PER_FRAME: u64 = 17556;

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
    pub config: Config,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>, config: Config) -> Self { Self { cpu: Cpu::default(), bus: Bus::new(rom), config } }

    pub fn reset(&mut self) {
        self.bus.reset(&self.config.mode);
        self.cpu.reset(&self.config.mode);
    }

    pub fn run_frame<M: Monitor>(&mut self, mut dbg: Option<&mut M>) -> Option<M::Stop> {
        let start = self.bus.get_cycle_count();
        while self.bus.get_cycle_count() - start < CYCLES_PER_FRAME {
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
}
