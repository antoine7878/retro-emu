use crate::gameboy::config::ConsoleType;
use crate::gameboy::memory::io_reg::IOReg;
use crate::gameboy::memory::{io_reg::IORegister, map};

#[derive(Default)]
pub struct DMA {
    is_delay: bool,
    is_active: bool,
    has_transferred: bool,
    current_byte: u8,
    src: u16,
    i: u16,
    dma: IORegister<{ map::DMA }, 0xFF>,
}

impl DMA {
    pub fn reset(&mut self, mode: &ConsoleType) { self.dma.reset(mode); }

    pub fn read(&self) -> u8 { self.dma.read() }

    pub fn write(&mut self, value: u8) {
        self.is_delay = true;
        let src = (value as u16) << 8;
        self.src = if src >= map::ECHO_START { src - 0x2000 } else { src };
        self.i = 0;
        self.dma.write(value);
    }

    pub fn tick(&mut self) -> Option<(u16, u16)> {
        if std::mem::take(&mut self.is_delay) {
            self.has_transferred = self.is_active;
            self.is_active = true;
            return None;
        }
        if !self.is_active {
            self.has_transferred = false;
            return None;
        }
        let ret = Some((self.src + self.i, self.i));
        self.i += 1;
        if self.i == 160 {
            self.is_active = false;
        }
        self.has_transferred = true;
        ret
    }

    pub fn conflict(&self, addr: u16) -> bool {
        (Self::on_vram_bus(self.src) && Self::on_vram_bus(addr))
            || (Self::on_ext_bus(addr) && Self::on_ext_bus(self.src))
            || Self::on_oam(addr)
    }
    fn on_oam(addr: u16) -> bool { matches!(addr, map::OAM_START..map::OAM_END) }
    fn on_vram_bus(addr: u16) -> bool { matches!(addr, map::VRAM_START..map::VRAM_END) }
    fn on_ext_bus(addr: u16) -> bool {
        matches!(addr,  map::ROM_START..map::ROM_END | map::SRAM_START..map::SRAM_END | map::WRAM_START..map::WRAM_END | map::ECHO_START..map::ECHO_END)
    }

    pub fn has_transferred(&self) -> bool { self.has_transferred }
    pub fn current_byte(&self) -> u8 { self.current_byte }
    pub fn set_current_byte(&mut self, byte: u8) { self.current_byte = byte }
}
