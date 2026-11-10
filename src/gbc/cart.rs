use std::collections::HashSet;

use super::config::CGMode;
use super::io_reg::{IOReg, IORegister};
use super::memory_map::{CART_RAM_BANK_SIZE, CART_ROM_BANK_SIZE, ROM_END, ROM_HALF, ROM_START, SRAM_END, SRAM_START};

const ROM_SIZE_FLAG: usize = 0x0148;
const RAM_SIZE_FLAG: usize = 0x149;

const TITLE_START: usize = 0x0134;
const TITLE_END: usize = 0x0144;
const MANU_START: usize = 0x013F;
const MANU_END: usize = 0x0143;
pub const CGB_FLAG: usize = 0x143;
const CARTIDGE_TYPE: usize = 0x0147;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbcKind {
    RomOnly,
    Mbc1,
    Mbc2,
    Mmm01,
    Mbc3,
    Mbc5,
    Mbc6,
    Mbc7,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Addon {
    Ram,
    Battery,
    Timer,
    Rumble,
    Sensor,
}

pub struct CartridgeHeader {
    rom_bank_count: u16,
    ram_bank_count: u8,
    rom_name: String,
    cgb_mode: CGMode,
    mbc_kind: MbcKind,
    addons: HashSet<Addon>,
}

impl CartridgeHeader {
    pub fn new(rom: &[u8]) -> Self {
        let rom_bank_count = Self::rom_bank_count(rom);
        let ram_bank_count = Self::ram_bank_count(rom);
        let rom_name = Self::rom_name(rom);
        let cgb_mode = Self::mode(rom);
        let (mbc_kind, addons) = Self::cartridge_type(rom);
        CartridgeHeader { rom_bank_count, ram_bank_count, rom_name, cgb_mode, mbc_kind, addons }
    }

    fn rom_name(rom: &[u8]) -> String { rom[TITLE_START..TITLE_END].iter().map(|&byte| byte as char).collect() }

    fn mode(rom: &[u8]) -> CGMode {
        match rom[CGB_FLAG] {
            0x80 => CGMode::Enhanced,
            0xC0 => CGMode::Cgb,
            _ => CGMode::Dmg,
        }
    }

    fn cartridge_type(rom: &[u8]) -> (MbcKind, HashSet<Addon>) {
        use Addon::*;
        let (kind, addons): (MbcKind, &[Addon]) = match rom[CARTIDGE_TYPE] {
            0x00 => (MbcKind::RomOnly, &[]),
            0x01 => (MbcKind::Mbc1, &[]),
            0x02 => (MbcKind::Mbc1, &[Ram]),
            0x03 => (MbcKind::Mbc1, &[Ram, Battery]),
            0x05 => (MbcKind::Mbc2, &[]),
            0x06 => (MbcKind::Mbc2, &[Battery]),
            0x08 => (MbcKind::RomOnly, &[Ram]),
            0x09 => (MbcKind::RomOnly, &[Ram, Battery]),
            0x0B => (MbcKind::Mmm01, &[]),
            0x0C => (MbcKind::Mmm01, &[Ram]),
            0x0D => (MbcKind::Mmm01, &[Ram, Battery]),
            0x0F => (MbcKind::Mbc3, &[Timer, Battery]),
            0x10 => (MbcKind::Mbc3, &[Timer, Ram, Battery]),
            0x11 => (MbcKind::Mbc3, &[]),
            0x12 => (MbcKind::Mbc3, &[Ram]),
            0x13 => (MbcKind::Mbc3, &[Ram, Battery]),
            0x19 => (MbcKind::Mbc5, &[]),
            0x1A => (MbcKind::Mbc5, &[Ram]),
            0x1B => (MbcKind::Mbc5, &[Ram, Battery]),
            0x1C => (MbcKind::Mbc5, &[Rumble]),
            0x1D => (MbcKind::Mbc5, &[Rumble, Ram]),
            0x1E => (MbcKind::Mbc5, &[Rumble, Ram, Battery]),
            0x20 => (MbcKind::Mbc6, &[]),
            0x22 => (MbcKind::Mbc7, &[Sensor, Rumble, Ram, Battery]),
            0xFC => (MbcKind::PocketCamera, &[]),
            0xFD => (MbcKind::BandaiTama5, &[]),
            0xFE => (MbcKind::HuC3, &[]),
            0xFF => (MbcKind::HuC1, &[Ram, Battery]),
            _ => unimplemented!("Invalid cartridge type"),
        };
        (kind, addons.iter().copied().collect())
    }

    fn rom_bank_count(rom: &[u8]) -> u16 {
        match rom[ROM_SIZE_FLAG] {
            s @ (0x00..0x08) => 1 << (s + 1),
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            _ => unimplemented!("Invalid rom size"),
        }
    }

    fn ram_bank_count(rom: &[u8]) -> u8 {
        match rom[RAM_SIZE_FLAG] {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => unimplemented!("Invalid ram size"),
        }
    }
}

pub struct Cartridge {
    mbc: Mbcs,
    header: CartridgeHeader,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Self {
        let header = CartridgeHeader::new(&rom);
        let mbc = Mbcs::from_header(rom, &header);
        Self { header, mbc }
    }

    pub fn reset(&mut self) { todo!() }

    pub fn read(&self, addr: u16) -> u8 { self.mbc.read(addr) }
    pub fn write(&mut self, addr: u16, value: u8) { self.mbc.write(addr, value) }
}

trait MBC {
    fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub enum Mbcs {
    RomOnly(RomOnly),
    Mbc1(Mbc1),
}

impl MBC for Mbcs {
    fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self {
        match header.mbc_kind {
            MbcKind::RomOnly => Mbcs::RomOnly(RomOnly::from_header(rom, &header)),
            MbcKind::Mbc1 => Mbcs::Mbc1(Mbc1::from_header(rom, &header)),
            _ => unimplemented!("mbc"),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match self {
            Mbcs::RomOnly(m) => m.read(addr),
            Mbcs::Mbc1(m) => m.read(addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match self {
            Mbcs::RomOnly(m) => m.write(addr, value),
            Mbcs::Mbc1(m) => m.write(addr, value),
        }
    }
}

pub struct RomOnly {
    rom: Vec<u8>,
    ram: Option<[u8; 1 << 13]>,
}

impl MBC for RomOnly {
    fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self {
        let ram = header.addons.contains(&Addon::Ram).then_some([0; 1 << 13]);
        Self { rom, ram }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            ROM_START..ROM_END => self.rom[(addr & 0x3FFF) as usize],
            SRAM_START..SRAM_END => self.ram.map_or_else(|| 0xFF, |r| r[(addr - SRAM_START) as usize]),
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        let Some(ram) = &mut self.ram else { return };
        ram[(addr - SRAM_START) as usize] = value;
    }
}

#[derive(Default)]
pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank_count: usize,
    ram_bank_count: usize,
    ram_size: usize,
    // rom_size: usize,
    bank1: IORegister<0x00, 0x1F>,
    bank2: IORegister<0x00, 0x03>,
    ram_enabled: bool,
    banking_mode: u8,
}

impl MBC for Mbc1 {
    fn from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Self {
        let rom_bank_count = header.rom_bank_count as usize;
        let rom_size = rom_bank_count * CART_ROM_BANK_SIZE;
        let mut rom = rom;
        rom.resize(rom_size, 0xFF);
        let ram_bank_count = header.ram_bank_count as usize;
        let ram_size = ram_bank_count * CART_RAM_BANK_SIZE;
        let ram = vec![0; ram_size];
        Self { rom, ram, rom_bank_count, ram_bank_count, ram_size, ..Default::default() }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            ROM_START..ROM_END => self.rom[self.rom_offset(addr)],
            SRAM_START..SRAM_END if self.ram_enabled => self.ram[self.ram_offset(addr)],
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000 => self.ram_enabled = value & 0x0F == 0x0A && self.ram_size > 0,
            0x2000..0x4000 => self.bank1.write(value),
            0x4000..0x6000 => self.bank2.write(value),
            0x6000..0x8000 => self.banking_mode = value & 1,
            SRAM_START..SRAM_END if self.ram_enabled => {
                let offset = self.ram_offset(addr);
                self.ram[offset] = value
            }
            _ => (),
        }
    }
}

impl Mbc1 {
    fn rom_offset(&self, addr: u16) -> usize {
        self.selected_rom_bank(addr) * CART_ROM_BANK_SIZE + (addr & 0x3FFF) as usize
    }

    fn ram_offset(&self, addr: u16) -> usize {
        self.selected_ram_bank() * CART_RAM_BANK_SIZE + (addr - SRAM_START) as usize
    }

    fn selected_rom_bank(&self, addr: u16) -> usize {
        let b1 = self.bank1.read();
        let b2 = self.bank2.read();
        let bank = match (self.banking_mode, addr) {
            (0, ROM_START..ROM_HALF) => 0,
            (1, ROM_START..ROM_HALF) => b2 << 5,
            (0 | 1, ROM_HALF..ROM_END) if b1 == 0 => (b2 << 5) | 1,
            (0 | 1, ROM_HALF..ROM_END) => (b2 << 5) | b1,
            _ => unreachable!(),
        };
        bank as usize & (self.rom_bank_count - 1)
    }

    fn selected_ram_bank(&self) -> usize {
        let bank = match self.banking_mode {
            0 => 0,
            1 => self.bank2.read(),
            _ => unreachable!(),
        };
        bank as usize & (self.ram_bank_count - 1)
    }
}
