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
        match op_code {
            0o000..0o100 => Self::block0(op_code),
            0o100..0o200 => Self::block1(op_code),
            0o200..0o300 => Self::block2(op_code),
            0o300.. => Self::block3(op_code),
        }
    }

    const fn milo(op_code: u8) -> (u8, u8) { ((op_code & 0o070) >> 3, op_code & 0o007) }

    const fn block0(op_code: u8) -> Self {
        match Self::milo(op_code) {
            (0, 0) => Instruction::Nop,
            (1, 0) => Instruction::LdA16SP,
            (2, 0) => Instruction::Stop,
            (3, 0) => Instruction::Jr(None),
            (c, 0) => Instruction::Jr(Some(Cond::oct(c))),

            (r @ (0 | 2 | 4 | 6), 1) => Instruction::LdImm16(R16::oct(r >> 1)),
            (r @ (1 | 3 | 5 | 7), 1) => Instruction::AddHl(R16::oct((r - 1) >> 1)),

            (r @ (0 | 2 | 4 | 6), 2) => Instruction::LdR16MemA(R16Mem::oct(r >> 1)),
            (r @ (1 | 3 | 5 | 7), 2) => Instruction::LdAR16Mem(R16Mem::oct((r - 1) >> 1)),

            (r @ (0 | 2 | 4 | 6), 3) => Instruction::Inc16(R16::oct(r >> 1)),
            (r @ (1 | 3 | 5 | 7), 3) => Instruction::Dec16(R16::oct((r - 1) >> 1)),

            (r, 4) => Instruction::Inc(R8::oct(r)),
            (r, 5) => Instruction::Dec(R8::oct(r)),
            (r, 6) => Instruction::LdImm(R8::oct(r)),
            (0, 7) => Instruction::Rlca,
            (1, 7) => Instruction::Rrca,
            (2, 7) => Instruction::Rla,
            (3, 7) => Instruction::Rra,
            (4, 7) => Instruction::Daa,
            (5, 7) => Instruction::Cpl,
            (6, 7) => Instruction::Scf,
            (7, 7) => Instruction::Ccf,
            _ => Instruction::Illegal(op_code),
        }
    }

    const fn block1(op_code: u8) -> Self {
        match Self::milo(op_code) {
            (6, 6) => Instruction::Halt,
            (dst @ 0..8, src @ 0..8) => Instruction::Ld(R8::oct(dst), R8::oct(src)),
            _ => Instruction::Illegal(op_code),
        }
    }

    const fn block2(op_code: u8) -> Self {
        match Self::milo(op_code) {
            (op @ 0..8, r @ 0..8) => Instruction::Alu(AluOp::oct(op), R8::oct(r)),
            _ => Instruction::Illegal(op_code),
        }
    }

    const fn block3(op_code: u8) -> Self {
        match Self::milo(op_code) {
            (c @ 0..4, 0) => Instruction::Ret(Some(Cond::oct(c))),
            (4, 0) => Instruction::LdhImmA,
            (5, 0) => Instruction::AddSpImm,
            (6, 0) => Instruction::LdhAImm,
            (7, 0) => Instruction::LdHlSp8,

            (1, 1) => Instruction::Ret(None),
            (3, 1) => Instruction::RetI,
            (5, 1) => Instruction::JpHl,
            (7, 1) => Instruction::LdSpHl,
            (r @ (0 | 2 | 4 | 6), 1) => Instruction::Pop(R16Stk::oct(r >> 1)),

            (o @ 0..4, 2) => Instruction::Jp(Some(Cond::oct(o))),
            (4, 2) => Instruction::LdhCA,
            (5, 2) => Instruction::LdhAbsA,
            (6, 2) => Instruction::LdhAC,
            (7, 2) => Instruction::LdhAAbs,

            (0, 3) => Instruction::Jp(None),
            (1, 3) => Self::prefix(op_code),
            (6, 3) => Instruction::Di,
            (7, 3) => Instruction::Ei,

            (c @ 0..4, 4) => Instruction::Call(Some(Cond::oct(c))),
            (r @ (0 | 2 | 4 | 6), 5) => Instruction::Push(R16Stk::oct(r >> 1)),
            (1, 5) => Instruction::Call(None),

            (op, 6) => Instruction::AluN(AluOp::oct(op)),

            (j, 7) => Instruction::Rst(j << 3),
            _ => Instruction::Illegal(op_code),
        }
    }

    pub const fn prefix(op_code: u8) -> Self {
        let (mi, lo) = Self::milo(op_code);
        match (op_code >> 6, mi, lo) {
            (0, o @ 0..8, r @ 0..8) => Instruction::Shift(ShiftOp::oct(o), R8::oct(r)),
            (1, i @ 0..8, r @ 0..8) => Instruction::Bit(i, R8::oct(r)),
            (2, i @ 0..8, r @ 0..8) => Instruction::Res(i, R8::oct(r)),
            (3, i @ 0..8, r @ 0..8) => Instruction::Set(i, R8::oct(r)),
            _ => Instruction::Illegal(op_code),
        }
    }

    const MAIN: [Self; 256] = {
        let mut table = [Self::Nop; 256];
        let mut op = 0;
        while op < 256 {
            table[op] = Self::decode(op as u8);
            op += 1;
        }
        table
    };

    const PREFIX: [Self; 256] = {
        let mut table = [Self::Nop; 256];
        let mut op = 0;
        while op < 256 {
            table[op] = Self::prefix(op as u8);
            op += 1;
        }
        table
    };

    pub const fn lookup(op_code: u8) -> Self { Self::MAIN[op_code as usize] }
    pub const fn lookup_prefix(op_code: u8) -> Self { Self::PREFIX[op_code as usize] }

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
