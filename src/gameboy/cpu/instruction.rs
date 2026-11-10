use std::fmt;

#[rustfmt::skip] #[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R8 { B, C, D, E, H, L, HlInd, A }

impl R8 {
    const ALL: [Self; 8] = [Self::B, Self::C, Self::D, Self::E, Self::H, Self::L, Self::HlInd, Self::A];
    const FMT: [&str; 8] = ["b", "c", "d", "e", "h", "l", "[hl]", "a"];
    const fn oct(o: u8) -> Self { Self::ALL[(o & 0b111) as usize] }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R16 { BC, DE, HL, SP }

impl R16 {
    const ALL: [Self; 4] = [Self::BC, Self::DE, Self::HL, Self::SP];
    const FMT: [&str; 4] = ["bc", "de", "hl", "sp"];
    const fn oct(o: u8) -> Self { Self::ALL[(o & 0b11) as usize] }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R16Stk { BC, DE , HL , AF }

impl R16Stk {
    const ALL: [Self; 4] = [Self::BC, Self::DE, Self::HL, Self::AF];
    const FMT: [&str; 4] = ["bc", "de", "hl", "af"];
    const fn oct(o: u8) -> Self { Self::ALL[(o & 0b11) as usize] }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R16Mem { BC, DE, HlInc, HlDec }

impl R16Mem {
    const ALL: [Self; 4] = [Self::BC, Self::DE, Self::HlInc, Self::HlDec];
    const FMT: [&str; 4] = ["[bc]", "[de]", "[hl+]", "[hl-]"];
    const fn oct(o: u8) -> Self { Self::ALL[(o & 0b11) as usize] }
}

impl From<R16Mem> for R16 {
    fn from(value: R16Mem) -> Self {
        match value {
            R16Mem::BC => R16::BC,
            R16Mem::DE => R16::DE,
            R16Mem::HlDec | R16Mem::HlInc => R16::HL,
        }
    }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cond { NZ, Z, NC, C }

impl Cond {
    const ALL: [Self; 4] = [Self::NZ, Self::Z, Self::NC, Self::C];
    const FMT: [&str; 4] = ["nz", "z", "nc", "c"];
    const fn oct(o: u8) -> Self { Self::ALL[(o & 0b11) as usize] }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AluOp { Add, Adc, Sub, Sbc, And, Xor, Or, Cp }

impl AluOp {
    const ALL: [Self; 8] = [Self::Add, Self::Adc, Self::Sub, Self::Sbc, Self::And, Self::Xor, Self::Or, Self::Cp];
    const FMT: [&str; 8] = ["add", "adc", "sub", "sbc", "and", "xor", "or", "cp"];
    const fn oct(o: u8) -> Self { Self::ALL[(o & 0b111) as usize] }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftOp { Rlc, Rrc, Rl, Rr, Sla, Sra, Swap, Srl }

impl ShiftOp {
    const ALL: [Self; 8] = [Self::Rlc, Self::Rrc, Self::Rl, Self::Rr, Self::Sla, Self::Sra, Self::Swap, Self::Srl];
    const FMT: [&str; 8] = ["rlc", "rrc", "rl", "rr", "sla", "sra", "swap", "srl"];
    const fn oct(o: u8) -> Self { Self::ALL[(o & 0b111) as usize] }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    // Control
    Nop,
    Stop,
    Halt,
    Di,
    Ei,

    // 8-bit loads
    Ld(R8, R8),
    LdImm(R8),
    LdhImmA,
    LdhAImm,
    LdhCA,
    LdhAC,
    LdhAbsA,
    LdhAAbs,

    // 16-bit loads
    LdAR16Mem(R16Mem),
    LdR16MemA(R16Mem),
    LdImm16(R16),
    LdA16SP,
    LdSpHl,
    LdHlSp8,

    // Arithmetic
    Alu(AluOp, R8),
    AluN(AluOp),
    Inc(R8),
    Dec(R8),
    Inc16(R16),
    Dec16(R16),
    AddHl(R16),
    AddSpImm,

    // Jump
    Jr(Option<Cond>),
    Jp(Option<Cond>),
    Call(Option<Cond>),
    Ret(Option<Cond>),
    JpHl,
    RetI,
    Rst(u8),

    // Stack
    Pop(R16Stk),
    Push(R16Stk),

    // Accumulator/Flag
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,

    // Prefix
    Shift(ShiftOp, R8),
    Bit(u8, R8),
    Res(u8, R8),
    Set(u8, R8),

    Illegal(u8),
}

impl Instruction {
    pub const fn decode(op_code: u8) -> Self {
        let (mi, lo) = Self::milo(op_code);
        match op_code {
            0o000 => Instruction::Nop,
            0o010 => Instruction::LdA16SP,
            0o020 => Instruction::Stop,
            0o030 => Instruction::Jr(None),
            0o040 | 0o050 | 0o060 | 0o070 => Instruction::Jr(Some(Cond::oct(mi))),

            0o001 | 0o021 | 0o041 | 0o061 => Instruction::LdImm16(R16::oct(mi / 2)),
            0o011 | 0o031 | 0o051 | 0o071 => Instruction::AddHl(R16::oct(mi / 2)),

            0o002 | 0o022 | 0o042 | 0o062 => Instruction::LdR16MemA(R16Mem::oct(mi / 2)),
            0o012 | 0o032 | 0o052 | 0o072 => Instruction::LdAR16Mem(R16Mem::oct(mi / 2)),

            0o003 | 0o023 | 0o043 | 0o063 => Instruction::Inc16(R16::oct(mi / 2)),
            0o013 | 0o033 | 0o053 | 0o073 => Instruction::Dec16(R16::oct(mi / 2)),

            0o004 | 0o014 | 0o024 | 0o034 | 0o044 | 0o054 | 0o064 | 0o074 => Instruction::Inc(R8::oct(mi)),
            0o005 | 0o015 | 0o025 | 0o035 | 0o045 | 0o055 | 0o065 | 0o075 => Instruction::Dec(R8::oct(mi)),
            0o006 | 0o016 | 0o026 | 0o036 | 0o046 | 0o056 | 0o066 | 0o076 => Instruction::LdImm(R8::oct(mi)),

            0o007 => Instruction::Rlca,
            0o017 => Instruction::Rrca,
            0o027 => Instruction::Rla,
            0o037 => Instruction::Rra,
            0o047 => Instruction::Daa,
            0o057 => Instruction::Cpl,
            0o067 => Instruction::Scf,
            0o077 => Instruction::Ccf,

            0o166 => Instruction::Halt,
            0o100..0o166 | 0o167..0o200 => Instruction::Ld(R8::oct(mi), R8::oct(lo)),

            0o200..0o300 => Instruction::Alu(AluOp::oct(mi), R8::oct(lo)),

            0o300 | 0o310 | 0o320 | 0o330 => Instruction::Ret(Some(Cond::oct(mi))),
            0o340 => Instruction::LdhImmA,
            0o350 => Instruction::AddSpImm,
            0o360 => Instruction::LdhAImm,
            0o370 => Instruction::LdHlSp8,

            0o301 | 0o321 | 0o341 | 0o361 => Instruction::Pop(R16Stk::oct(mi / 2)),
            0o311 => Instruction::Ret(None),
            0o331 => Instruction::RetI,
            0o351 => Instruction::JpHl,
            0o371 => Instruction::LdSpHl,

            0o302 | 0o312 | 0o322 | 0o332 => Instruction::Jp(Some(Cond::oct(mi))),
            0o342 => Instruction::LdhCA,
            0o352 => Instruction::LdhAbsA,
            0o362 => Instruction::LdhAC,
            0o372 => Instruction::LdhAAbs,

            0o303 => Instruction::Jp(None),
            0o313 => Self::prefix(op_code),
            0o363 => Instruction::Di,
            0o373 => Instruction::Ei,

            0o304 | 0o314 | 0o324 | 0o334 => Instruction::Call(Some(Cond::oct(mi))),
            0o305 | 0o325 | 0o345 | 0o365 => Instruction::Push(R16Stk::oct(mi / 2)),
            0o315 => Instruction::Call(None),

            0o306 | 0o316 | 0o326 | 0o336 | 0o346 | 0o356 | 0o366 | 0o376 => Instruction::AluN(AluOp::oct(mi)),
            0o307 | 0o317 | 0o327 | 0o337 | 0o347 | 0o357 | 0o367 | 0o377 => Instruction::Rst(op_code & 0o070),

            0o323 | 0o333 | 0o343 | 0o353 | 0o344 | 0o354 | 0o364 | 0o374 | 0o335 | 0o355 | 0o375 => {
                Instruction::Illegal(op_code)
            }
        }
    }

    const fn milo(op_code: u8) -> (u8, u8) { ((op_code & 0o070) >> 3, op_code & 0o007) }

    pub const fn prefix(op_code: u8) -> Self {
        let (mi, lo) = Self::milo(op_code);
        match op_code {
            0o000..0o100 => Instruction::Shift(ShiftOp::oct(mi), R8::oct(lo)),
            0o100..0o200 => Instruction::Bit(mi, R8::oct(lo)),
            0o200..0o300 => Instruction::Res(mi, R8::oct(lo)),
            0o300.. => Instruction::Set(mi, R8::oct(lo)),
        }
    }

    pub const fn shape(&self) -> (u8, u8) {
        match self {
            Instruction::Nop => (1, 4),
            Instruction::Stop => (2, 4),
            Instruction::Halt => (1, 4),
            Instruction::Di => (1, 4),
            Instruction::Ei => (1, 4),
            Instruction::Ld(R8::HlInd, _) | Instruction::Ld(_, R8::HlInd) => (1, 8),
            Instruction::Ld(_, _) => (1, 4),
            Instruction::LdImm(R8::HlInd) => (2, 12),
            Instruction::LdImm(_) => (2, 8),
            Instruction::LdhImmA => (2, 12),
            Instruction::LdhAImm => (2, 12),
            Instruction::LdhCA => (1, 8),
            Instruction::LdhAC => (1, 8),
            Instruction::LdhAbsA => (3, 16),
            Instruction::LdhAAbs => (3, 16),
            Instruction::LdImm16(_) => (3, 12),
            Instruction::LdA16SP => (3, 20),
            Instruction::LdR16MemA(_) => (1, 8),
            Instruction::LdAR16Mem(_) => (1, 8),
            Instruction::LdSpHl => (1, 8),
            Instruction::LdHlSp8 => (2, 12),
            Instruction::Alu(_, R8::HlInd) => (1, 8),
            Instruction::Alu(_, _) => (1, 4),
            Instruction::AluN(_) => (2, 8),
            Instruction::Inc(R8::HlInd) => (1, 12),
            Instruction::Inc(_) => (1, 4),
            Instruction::Dec(R8::HlInd) => (1, 12),
            Instruction::Dec(_) => (1, 4),
            Instruction::Inc16(_) => (1, 8),
            Instruction::Dec16(_) => (1, 8),
            Instruction::AddHl(_) => (1, 8),
            Instruction::AddSpImm => (2, 16),
            Instruction::Jr(None) => (2, 12),
            Instruction::Jr(Some(_)) => (2, (8 << 4) | 12),
            Instruction::Jp(None) => (3, 16),
            Instruction::Jp(Some(_)) => (3, (12 << 4) | 16),
            Instruction::JpHl => (1, 4),
            Instruction::Call(None) => (3, 24),
            Instruction::Call(Some(_)) => (3, (12 << 4) | 24),
            Instruction::Ret(None) => (1, 16),
            Instruction::Ret(Some(_)) => (1, (8 << 4) | 20),
            Instruction::RetI => (1, 16),
            Instruction::Rst(_) => (1, 16),
            Instruction::Pop(_) => (1, 12),
            Instruction::Push(_) => (1, 16),
            Instruction::Rlca => (1, 4),
            Instruction::Rrca => (1, 4),
            Instruction::Rla => (1, 4),
            Instruction::Rra => (1, 4),
            Instruction::Daa => (1, 4),
            Instruction::Cpl => (1, 4),
            Instruction::Scf => (1, 4),
            Instruction::Ccf => (1, 4),
            Instruction::Shift(_, R8::HlInd) => (2, 16),
            Instruction::Shift(_, _) => (2, 8),
            Instruction::Bit(_, R8::HlInd) => (2, 12),
            Instruction::Bit(_, _) => (2, 8),
            Instruction::Res(_, R8::HlInd) => (2, 16),
            Instruction::Res(_, _) => (2, 8),
            Instruction::Set(_, R8::HlInd) => (2, 16),
            Instruction::Set(_, _) => (2, 8),
            Instruction::Illegal(_) => (1, 4),
        }
    }

    pub fn disassemble(addr: u16, read: impl Fn(u16) -> u8) -> (String, u16) {
        let opcode = read(addr);
        let instr = match opcode {
            0xCB => Self::prefix(read(addr.wrapping_add(1))),
            _ => Self::decode(opcode),
        };
        let instruction_len = instr.shape().0 as u16;
        let n8 = read(addr.wrapping_add(1));
        let n16 = u16::from_le_bytes([n8, read(addr.wrapping_add(2))]);
        let e8 = match instr {
            Instruction::Jr(_) => format!("${:04X}", addr.wrapping_add(2).wrapping_add_signed(n8 as i8 as i16)),
            _ => format!("{}", n8 as i8),
        };
        let text = instr
            .to_string()
            .replace("n16", &format!("${n16:04X}"))
            .replace("a16", &format!("${n16:04X}"))
            .replace("n8", &format!("${n8:02X}"))
            .replace("a8", &format!("$FF{n8:02X}"))
            .replace("e8", &e8);
        (text, instruction_len)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Nop => write!(f, "nop"),
            Instruction::Stop => write!(f, "stop"),
            Instruction::Halt => write!(f, "halt"),
            Instruction::Di => write!(f, "di"),
            Instruction::Ei => write!(f, "ei"),
            Instruction::Ld(dst, src) => write!(f, "ld {dst}, {src}"),
            Instruction::LdImm(r) => write!(f, "ld {r}, n8"),
            Instruction::LdhImmA => write!(f, "ldh [a8], a"),
            Instruction::LdhAImm => write!(f, "ldh a, [a8]"),
            Instruction::LdhCA => write!(f, "ldh [c], a"),
            Instruction::LdhAC => write!(f, "ldh a, [c]"),
            Instruction::LdhAbsA => write!(f, "ld [a16], a"),
            Instruction::LdhAAbs => write!(f, "ld a, [a16]"),
            Instruction::LdImm16(r) => write!(f, "ld {r}, n16"),
            Instruction::LdA16SP => write!(f, "ld [a16], sp"),
            Instruction::LdR16MemA(r) => write!(f, "ld {r}, a"),
            Instruction::LdAR16Mem(r) => write!(f, "ld a, {r}"),
            Instruction::LdSpHl => write!(f, "ld sp, hl"),
            Instruction::LdHlSp8 => write!(f, "ld hl, sp + e8"),
            Instruction::Alu(op, r) => write!(f, "{op} a, {r}"),
            Instruction::AluN(op) => write!(f, "{op} a, n8"),
            Instruction::Inc(r) => write!(f, "inc {r}"),
            Instruction::Dec(r) => write!(f, "dec {r}"),
            Instruction::Inc16(r) => write!(f, "inc {r}"),
            Instruction::Dec16(r) => write!(f, "dec {r}"),
            Instruction::AddHl(r) => write!(f, "add hl, {r}"),
            Instruction::AddSpImm => write!(f, "add sp, e8"),
            Instruction::Jr(None) => write!(f, "jr e8"),
            Instruction::Jr(Some(c)) => write!(f, "jr {c}, e8"),
            Instruction::Jp(None) => write!(f, "jp a16"),
            Instruction::Jp(Some(c)) => write!(f, "jp {c}, a16"),
            Instruction::JpHl => write!(f, "jp hl"),
            Instruction::Call(None) => write!(f, "call a16"),
            Instruction::Call(Some(c)) => write!(f, "call {c}, a16"),
            Instruction::Ret(None) => write!(f, "ret"),
            Instruction::Ret(Some(c)) => write!(f, "ret {c}"),
            Instruction::RetI => write!(f, "reti"),
            Instruction::Rst(i) => write!(f, "rst {i:02X}"),
            Instruction::Pop(r) => write!(f, "pop {r}"),
            Instruction::Push(r) => write!(f, "push {r}"),
            Instruction::Rlca => write!(f, "rlca"),
            Instruction::Rrca => write!(f, "rrca"),
            Instruction::Rla => write!(f, "rla"),
            Instruction::Rra => write!(f, "rra"),
            Instruction::Daa => write!(f, "daa"),
            Instruction::Cpl => write!(f, "cpl"),
            Instruction::Scf => write!(f, "scf"),
            Instruction::Ccf => write!(f, "ccf"),
            Instruction::Shift(op, r) => write!(f, "{op} {r}"),
            Instruction::Bit(i, r) => write!(f, "bit {i}, {r}"),
            Instruction::Res(i, r) => write!(f, "res {i}, {r}"),
            Instruction::Set(i, r) => write!(f, "set {i}, {r}"),
            Instruction::Illegal(opc) => write!(f, "ILLEGAL opcode {opc:#o}"),
        }
    }
}

macro_rules! impl_display_from_fmt {
    ($($ty:ty),* $(,)?) => {
        $(
            impl fmt::Display for $ty {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(Self::FMT[*self as usize])
                }
            }
        )*
    };
}

impl_display_from_fmt!(R8, R16, R16Stk, R16Mem, Cond, AluOp, ShiftOp,);
