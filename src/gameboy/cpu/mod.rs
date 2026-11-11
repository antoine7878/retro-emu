mod alu;
pub mod flag;
pub mod instruction;
pub mod registers;
mod shift;

use crate::gameboy::memory::bus::Bus;
use crate::gameboy::memory::map;
use flag::{FlagSpec, Flags};
use instruction::{AluOp, Cond, Instruction, R8, R16, R16Mem, R16Stk, ShiftOp};
use registers::CpuRegisters;

use super::config::ConsoleType;

#[derive(Debug, PartialEq, Eq, Default)]
pub enum PowerMode {
    #[default]
    Normal,
    Halt,
    Stop,
    Illegal,
}

#[derive(Default)]
pub struct Cpu {
    power_mode: PowerMode,
    ime: bool,
    registers: CpuRegisters,
    delay_ie: bool,
    halt_bug: bool,
}

impl Cpu {
    pub fn reset(&mut self, mode: &ConsoleType) {
        self.registers.reset(mode);
        self.ime = false;
        self.delay_ie = false;
        self.halt_bug = false;
        self.power_mode = PowerMode::default();
    }

    pub fn step(&mut self, bus: &mut Bus) {
        let opcode = self.fetch_opcode(bus);
        let instruction = match opcode {
            0xCB => Instruction::prefix(self.take_pc8(bus)),
            _ => Instruction::decode(opcode),
        };
        self.execute(instruction, bus)
    }

    fn fetch_opcode(&mut self, bus: &mut Bus) -> u8 {
        let op = bus.read(self.pc());
        if !std::mem::take(&mut self.halt_bug) {
            *self.registers.pc_mut() = self.pc().wrapping_add(1);
        }
        op
    }

    pub fn halted(&mut self, bus: &mut Bus) -> bool {
        if self.power_mode != PowerMode::Halt {
            return false;
        }
        if bus.pending_interrupt() == 0 {
            bus.tick();
            return true;
        }
        self.power_mode = PowerMode::Normal;
        false
    }

    pub fn dispatch_interrupt(&mut self, bus: &mut Bus) {
        let pending = bus.pending_interrupt();
        if !self.ime || pending == 0 {
            return;
        }
        let int = pending.trailing_zeros() as u8;
        bus.ack_interrupt(int);
        self.ime = false;
        self.delay_ie = false;
        bus.tick();
        self.push_stack(bus, self.pc());
        *self.registers.pc_mut() = 0x40 + 8 * int as u16;
        bus.tick();
    }

    fn execute(&mut self, instruction: Instruction, bus: &mut Bus) {
        if self.power_mode == PowerMode::Illegal {
            return;
        }
        if std::mem::take(&mut self.delay_ie) {
            self.ime = true
        }
        match instruction {
            Instruction::Nop => (),
            Instruction::Stop => self.stop(bus),
            Instruction::Halt => self.halt(bus),
            Instruction::Di => self.ime = false,
            Instruction::Ei => self.delay_ie = true,

            Instruction::Ld(dst, src) => self.ld(bus, dst, src),
            Instruction::LdImm(r) => self.ldn8(bus, r),
            Instruction::LdhImmA => self.ldimma(bus),
            Instruction::LdhAImm => self.ldaimm(bus),
            Instruction::LdhCA => self.ldhca(bus),
            Instruction::LdhAC => self.ldhac(bus),

            Instruction::LdR16MemA(rm) => self.ldr16a(bus, rm),
            Instruction::LdAR16Mem(rm) => self.ldar16(bus, rm),
            Instruction::LdImm16(r) => self.ldn16(bus, r),
            Instruction::LdA16SP => self.lda16sp(bus),
            Instruction::LdSpHl => self.ldsphl(bus),
            Instruction::LdHlSp8 => self.ldhlsp8(bus),
            Instruction::LdhAbsA => self.ldhabsa(bus),
            Instruction::LdhAAbs => self.ldhaabs(bus),

            Instruction::Alu(op, r) => self.alu(bus, op, r),
            Instruction::AluN(op) => self.alun(bus, op),
            Instruction::AddHl(r) => self.addhl(bus, r),
            Instruction::AddSpImm => self.addspimm(bus),
            Instruction::Inc(r) => self.inc8(bus, r),
            Instruction::Dec(r) => self.dec8(bus, r),
            Instruction::Inc16(r) => self.inc16(bus, r),
            Instruction::Dec16(r) => self.dec16(bus, r),

            Instruction::Jp(c) => self.jp(bus, c),
            Instruction::Jr(c) => self.jr(bus, c),
            Instruction::Call(c) => self.call(bus, c),
            Instruction::Ret(c) => self.ret(bus, c),
            Instruction::JpHl => self.jphl(),
            Instruction::RetI => self.reti(bus),
            Instruction::Rst(vec) => self.rst(bus, vec as u16),

            Instruction::Push(r) => self.push(bus, r),
            Instruction::Pop(r) => self.pop(bus, r),

            Instruction::Rlca => self.rlca(),
            Instruction::Rrca => self.rrca(),
            Instruction::Rla => self.rla(),
            Instruction::Rra => self.rra(),
            Instruction::Daa => self.daa(),
            Instruction::Cpl => self.cpl(),
            Instruction::Scf => self.scf(),
            Instruction::Ccf => self.ccf(),

            Instruction::Shift(op, r) => self.shift(bus, op, r),
            Instruction::Bit(i, r) => self.bit(bus, i, r),
            Instruction::Res(i, r) => self.res(bus, i, r),
            Instruction::Set(i, r) => self.set(bus, i, r),

            Instruction::Illegal(_) => self.power_mode = PowerMode::Illegal,
        }
    }

    // ----- Utils ------------------------------

    pub fn take_pc8(&mut self, bus: &mut Bus) -> u8 {
        let res = bus.read(self.pc());
        *self.registers.pc_mut() = self.pc().wrapping_add(1);
        res
    }

    pub fn take_pc16(&mut self, bus: &mut Bus) -> u16 {
        let r1 = self.take_pc8(bus);
        let r2 = self.take_pc8(bus);
        (r2 as u16) << 8 | r1 as u16
    }

    fn map_high_page(addr: u8) -> u16 { 0xFF00 | addr as u16 }

    // ----- Registers Getter/Setter ------------------------------

    pub fn pc(&self) -> u16 { self.registers.pc() }

    fn get_r8(&self, bus: &mut Bus, r: R8) -> u8 {
        match r {
            R8::HlInd => bus.read(self.registers.get_hl()),
            _ => self.registers.get_r8(r),
        }
    }

    fn set_r8(&mut self, bus: &mut Bus, r: R8, value: u8) {
        match r {
            R8::HlInd => bus.write(self.registers.get_hl(), value),
            _ => self.registers.set_r8(r, value),
        }
    }
    const fn get_r16(&self, r: R16) -> u16 { self.registers.get_r16(r) }
    const fn set_r16(&mut self, r: R16, value: u16) { self.registers.set_r16(r, value); }

    const fn condition(&self, c: Option<Cond>) -> bool {
        match c {
            None => true,
            Some(Cond::Z) => self.registers.get_z() == 1,
            Some(Cond::NZ) => self.registers.get_z() == 0,
            Some(Cond::C) => self.registers.get_c() == 1,
            Some(Cond::NC) => self.registers.get_c() == 0,
        }
    }

    fn stop(&mut self, bus: &mut Bus) {
        self.take_pc8(bus);
        bus.write_internal(map::DIV, 0);
        self.power_mode = PowerMode::Stop
    }

    fn halt(&mut self, bus: &mut Bus) {
        if !self.ime && bus.pending_interrupt() != 0 {
            self.halt_bug = true;
        } else {
            self.power_mode = PowerMode::Halt;
        }
    }

    // ---- Load 8-bit ------------------------------

    fn ld(&mut self, bus: &mut Bus, dst: R8, src: R8) {
        let value = self.get_r8(bus, src);
        self.set_r8(bus, dst, value)
    }

    fn ldn8(&mut self, bus: &mut Bus, r: R8) {
        let n = self.take_pc8(bus);
        self.set_r8(bus, r, n);
    }

    // ----- Load 16-bit ------------------------------

    fn ldimma(&mut self, bus: &mut Bus) {
        let imm = self.take_pc8(bus);
        let addr = Self::map_high_page(imm);
        let value = self.get_r8(bus, R8::A);
        bus.write(addr, value);
    }

    fn ldaimm(&mut self, bus: &mut Bus) {
        let imm = self.take_pc8(bus);
        let addr = Self::map_high_page(imm);
        let value = bus.read(addr);
        self.set_r8(bus, R8::A, value)
    }

    fn ldhca(&mut self, bus: &mut Bus) {
        let c = self.get_r8(bus, R8::C);
        let addr = Self::map_high_page(c);
        let value = self.get_r8(bus, R8::A);
        bus.write(addr, value);
    }

    fn ldhac(&mut self, bus: &mut Bus) {
        let c = self.get_r8(bus, R8::C);
        let addr = Self::map_high_page(c);
        let value = bus.read(addr);
        self.set_r8(bus, R8::A, value);
    }

    fn ldr16a(&mut self, bus: &mut Bus, rm: R16Mem) {
        let r = R16::from(rm);
        let addr = self.get_r16(r);
        let value = self.get_r8(bus, R8::A);
        bus.write(addr, value);
        self.update_r16mem(rm);
    }

    fn ldar16(&mut self, bus: &mut Bus, rm: R16Mem) {
        let r = R16::from(rm);
        let addr = self.get_r16(r);
        let value = bus.read(addr);
        self.set_r8(bus, R8::A, value);
        self.update_r16mem(rm);
    }

    fn update_r16mem(&mut self, r: R16Mem) {
        match r {
            R16Mem::HlInc => self.registers.set_hl(self.registers.get_hl().wrapping_add(1)),
            R16Mem::HlDec => self.registers.set_hl(self.registers.get_hl().wrapping_sub(1)),
            _ => (),
        }
    }

    fn ldn16(&mut self, bus: &mut Bus, r: R16) {
        let n = self.take_pc16(bus);
        self.set_r16(r, n);
    }

    fn lda16sp(&mut self, bus: &mut Bus) {
        let sp = self.get_r16(R16::SP);
        let addr = self.take_pc16(bus);
        bus.write(addr, (sp & 0xFF) as u8);
        bus.write(addr.wrapping_add(1), (sp >> 8) as u8);
    }

    fn ldsphl(&mut self, bus: &mut Bus) {
        let value = self.get_r16(R16::HL);
        self.set_r16(R16::SP, value);
        bus.tick();
    }

    fn ldhlsp8(&mut self, bus: &mut Bus) {
        let sp = self.get_r16(R16::SP);
        let imm = self.take_pc8(bus) as i8;
        bus.tick();
        let (result, flags) = alu::add_sp_e8(sp, imm);
        self.set_r16(R16::HL, result);
        self.registers.set_flags(flags, ADD_SP);
    }

    fn ldhabsa(&mut self, bus: &mut Bus) {
        let addr = self.take_pc16(bus);
        let value = self.get_r8(bus, R8::A);
        bus.write(addr, value);
    }

    fn ldhaabs(&mut self, bus: &mut Bus) {
        let addr = self.take_pc16(bus);
        let value = bus.read(addr);
        self.set_r8(bus, R8::A, value);
    }

    // ----- Alu ------------------------------

    fn alu(&mut self, bus: &mut Bus, op: AluOp, r: R8) {
        let a = self.get_r8(bus, R8::A);
        let r = self.get_r8(bus, r);
        let (result, flags) = op.operation()(a, r, self.registers.get_c() == 1);
        self.set_r8(bus, R8::A, result);
        self.registers.set_flags(flags, op.flag_spec());
    }

    fn alun(&mut self, bus: &mut Bus, op: AluOp) {
        let a = self.get_r8(bus, R8::A);
        let imm = self.take_pc8(bus);
        let (result, flags) = op.operation()(a, imm, self.registers.get_c() == 1);
        self.set_r8(bus, R8::A, result);
        self.registers.set_flags(flags, op.flag_spec());
    }

    fn addhl(&mut self, bus: &mut Bus, r: R16) {
        let hl = self.get_r16(R16::HL);
        let r = self.get_r16(r);
        let (result, flags) = alu::add16(hl, r, false);
        self.set_r16(R16::HL, result);
        self.registers.set_flags(flags, ADD_HL);
        bus.tick()
    }

    fn addspimm(&mut self, bus: &mut Bus) {
        let sp = self.get_r16(R16::SP);
        let imm = self.take_pc8(bus) as i8;
        bus.ticks(2);
        let (result, flags) = alu::add_sp_e8(sp, imm);
        self.set_r16(R16::SP, result);
        self.registers.set_flags(flags, ADD_SP);
    }

    fn inc8(&mut self, bus: &mut Bus, r: R8) {
        let v = self.get_r8(bus, r);
        let (result, flags) = alu::add8(v, 1, false);
        self.set_r8(bus, r, result);
        self.registers.set_flags(flags, INC);
    }

    fn dec8(&mut self, bus: &mut Bus, r: R8) {
        let v = self.get_r8(bus, r);
        let (result, flags) = alu::sub8(v, 1, false);
        self.set_r8(bus, r, result);
        self.registers.set_flags(flags, DEC);
    }

    fn inc16(&mut self, bus: &mut Bus, r: R16) {
        let v = self.get_r16(r);
        let (result, flags) = alu::add16(v, 1, false);
        self.set_r16(r, result);
        self.registers.set_flags(flags, NOF);
        bus.tick();
    }

    fn dec16(&mut self, bus: &mut Bus, r: R16) {
        let v = self.get_r16(r);
        let (result, flags) = alu::sub16(v, 1, false);
        self.set_r16(r, result);
        self.registers.set_flags(flags, NOF);
        bus.tick();
    }

    // ----- Jump ------------------------------

    fn jp(&mut self, bus: &mut Bus, c: Option<Cond>) {
        let addr = self.take_pc16(bus);
        if !self.condition(c) {
            return;
        }
        *self.registers.pc_mut() = addr;
        bus.tick();
    }

    fn jr(&mut self, bus: &mut Bus, c: Option<Cond>) {
        let offset = self.take_pc8(bus) as i8 as i32;
        if !self.condition(c) {
            return;
        }
        *self.registers.pc_mut() = (self.pc() as i32).wrapping_add(offset) as u16;
        bus.tick();
    }

    const fn jphl(&mut self) { *self.registers.pc_mut() = self.get_r16(R16::HL); }

    fn call(&mut self, bus: &mut Bus, c: Option<Cond>) {
        let addr = self.take_pc16(bus);
        if !self.condition(c) {
            return;
        }
        self.rst(bus, addr);
    }

    fn ret(&mut self, bus: &mut Bus, c: Option<Cond>) {
        if c.is_some() {
            bus.tick()
        }
        if !self.condition(c) {
            return;
        }
        *self.registers.pc_mut() = self.pop_stack(bus);
        bus.tick();
    }

    fn reti(&mut self, bus: &mut Bus) {
        self.ime = true;
        self.ret(bus, None);
    }

    fn rst(&mut self, bus: &mut Bus, vec: u16) {
        self.push_stack(bus, self.pc());
        *self.registers.pc_mut() = vec
    }

    fn push(&mut self, bus: &mut Bus, r: R16Stk) {
        let value = self.registers.get_r16stck(r);
        self.push_stack(bus, value);
    }

    fn push_stack(&mut self, bus: &mut Bus, value: u16) {
        bus.tick();
        self.registers.dec_sp();
        bus.write(self.registers.get_sp(), (value >> 8) as u8);
        self.registers.dec_sp();
        bus.write(self.registers.get_sp(), (value & 0xFF) as u8);
    }

    fn pop(&mut self, bus: &mut Bus, r: R16Stk) {
        let value = self.pop_stack(bus);
        self.registers.set_r16stck(r, value);
    }

    fn pop_stack(&mut self, bus: &mut Bus) -> u16 {
        let lo = bus.read(self.registers.get_sp());
        self.registers.inc_sp();
        let hi = bus.read(self.registers.get_sp());
        self.registers.inc_sp();
        (hi as u16) << 8 | (lo as u16)
    }

    // ----- Rotate ------------------------------

    fn rlca(&mut self) {
        let a = self.registers.get_a();
        let (value, c) = shift::rlc(a);
        self.registers.set_a(value);
        let flags = Flags { c, ..Default::default() };
        self.registers.set_flags(flags, RLCA);
    }

    fn rrca(&mut self) {
        let a = self.registers.get_a();
        let (value, c) = shift::rrc(a);
        self.registers.set_a(value);
        let flags = Flags { c, ..Default::default() };
        self.registers.set_flags(flags, RLCA);
    }

    fn rla(&mut self) {
        let a = self.registers.get_a();
        let (value, c) = shift::rl(a, self.registers.get_c());
        self.registers.set_a(value);
        let flags = Flags { c, ..Default::default() };
        self.registers.set_flags(flags, RLCA);
    }

    fn rra(&mut self) {
        let a = self.registers.get_a();
        let (value, c) = shift::rr(a, self.registers.get_c());
        self.registers.set_a(value);
        let flags = Flags { c, ..Default::default() };
        self.registers.set_flags(flags, RLCA);
    }

    fn cpl(&mut self) {
        let value = !self.registers.get_a();
        self.registers.set_a(value);
        let flags = Flags { n: true, h: true, ..Default::default() };
        self.registers.set_flags(flags, CPL);
    }

    // ----- Accumulator/Flag ------------------------------

    #[rustfmt::skip]
    fn daa(&mut self) {
        let mut adj = 0;
        let Flags { n, h, mut c, .. } = self.registers.get_flags();
        let a = self.registers.get_a();
        let value = if n {
            if h { adj += 0x06 }
            if c { adj += 0x60 }
            a.wrapping_sub(adj)
        } else {
            if h || a & 0x0F > 0x09 { adj += 0x06 }
            if c || a > 0x99 { adj += 0x60; c = true;}
            a.wrapping_add(adj)
        };
        self.registers.set_a(value);
        let flags = Flags { z: value == 0, c, ..Default::default() };
        self.registers.set_flags(flags, DAA);
    }

    fn scf(&mut self) {
        self.registers.clear_n();
        self.registers.clear_h();
        self.registers.set_c();
    }

    fn ccf(&mut self) {
        self.registers.clear_n();
        self.registers.clear_h();
        self.registers.toogle_c();
    }

    // ----- Prefix ------------------------------

    fn shift(&mut self, bus: &mut Bus, op: ShiftOp, r: R8) {
        let val = self.get_r8(bus, r);
        let (value, c) = shift::shift(op, val, self.registers.get_c());
        self.set_r8(bus, r, value);
        let flags = Flags { z: value == 0, c, ..Default::default() };
        self.registers.set_flags(flags, SHIFT);
    }

    fn bit(&mut self, bus: &mut Bus, i: u8, r: R8) {
        let r = self.get_r8(bus, r);
        let value = (r >> i) & 1;
        let flags = Flags { z: value == 0, n: false, h: true, ..Default::default() };
        self.registers.set_flags(flags, BIT);
    }

    fn res(&mut self, bus: &mut Bus, i: u8, r: R8) {
        let value = self.get_r8(bus, r) & !(1 << i);
        self.set_r8(bus, r, value);
    }

    fn set(&mut self, bus: &mut Bus, i: u8, r: R8) {
        let value = self.get_r8(bus, r) | (1 << i);
        self.set_r8(bus, r, value);
    }

    pub const fn registers(&self) -> &CpuRegisters { &self.registers }
}

const DAA: FlagSpec = FlagSpec::znhc("R-0R");
const CPL: FlagSpec = FlagSpec::znhc("-RR-");
const DEC: FlagSpec = FlagSpec::znhc("R1R-");
const INC: FlagSpec = FlagSpec::znhc("R0R-");

const ADD_HL: FlagSpec = FlagSpec::znhc("-0RR");

const ADD_SP: FlagSpec = FlagSpec::znhc("00RR");
const RLCA: FlagSpec = FlagSpec::znhc("000R");
// const CB_RLC: FlagSpec = FlagSpec::znhc("R00R");
const NOF: FlagSpec = FlagSpec::znhc("----");
const SHIFT: FlagSpec = FlagSpec::znhc("R00R");
const BIT: FlagSpec = FlagSpec::znhc("R01-");
