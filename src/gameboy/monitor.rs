use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AccessKind {
    Read,
    #[default]
    Write,
}

impl fmt::Display for AccessKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessKind::Read => write!(f, "read"),
            AccessKind::Write => write!(f, "write"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemAccess {
    pub addr: u16,
    pub value: u8,
    pub on: AccessKind,
}

impl MemAccess {
    pub fn read(addr: u16, value: u8) -> Self { Self { addr, value, on: AccessKind::Read } }
    pub fn write(addr: u16, value: u8) -> Self { Self { addr, value, on: AccessKind::Write } }
}

pub trait Monitor {
    type Stop;
    fn check(&mut self, pc: u16) -> Option<Self::Stop>;
    fn check_accesses(&mut self, accesses: impl Iterator<Item = MemAccess>) -> Option<Self::Stop>;
}
