use crate::gameboy::memory::map;
use crate::save_state_struct;

#[derive(Default)]
pub struct Hdma {
    src: u16,
    dst: u16,
    blocks: u8,
    active: bool,
    hblank: bool,
}

save_state_struct!(Hdma { src, dst, blocks, active, hblank });

impl Hdma {
    pub fn reset(&mut self) { *self = Self::default(); }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            map::HDMA5 => {
                let busy = if self.active || self.hblank { 0x00 } else { 0x80 };
                busy | (self.blocks.wrapping_sub(1) & 0x7F)
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8, in_hblank: bool) {
        match addr {
            map::HDMA1 => {
                self.src = (self.src & 0x00F0) | (value as u16) << 8;
                if self.src >= map::ECHO_START {
                    self.src |= 0xF000;
                }
            }
            map::HDMA2 => self.src = (self.src & 0xFF00) | (value & 0xF0) as u16,
            map::HDMA3 => self.dst = (self.dst & 0x00F0) | (value as u16) << 8,
            map::HDMA4 => self.dst = (self.dst & 0xFF00) | (value & 0xF0) as u16,
            map::HDMA5 => self.start(value, in_hblank),
            _ => unreachable!("Address not in HDMA"),
        }
    }

    fn start(&mut self, value: u8, in_hblank: bool) {
        let hblank = value & 0x80 != 0;
        self.blocks = (value & 0x7F) + 1;
        if !hblank && self.hblank {
            self.hblank = false;
            return;
        }
        self.hblank = hblank;
        self.active = !hblank || in_hblank;
    }

    pub fn hblank_started(&mut self) { self.active |= self.hblank; }

    pub fn skip_block(&mut self) {
        if self.hblank {
            self.active = false;
        }
    }

    pub fn next(&mut self) -> Option<(u16, u16)> {
        if !self.active {
            return None;
        }
        let src = self.src;
        let dst = map::VRAM_START | (self.dst & 0x1FFF);
        self.src = self.src.wrapping_add(1);
        self.dst = self.dst.wrapping_add(1);
        if self.dst & 0x0F == 0 {
            self.blocks -= 1;
            if self.blocks == 0 || self.dst == 0 {
                self.active = false;
                self.hblank = false;
            } else if self.hblank {
                self.active = false;
            }
        }
        Some((src, dst))
    }

    pub fn readable(src: u16) -> bool {
        matches!(src, map::ROM_START..map::ROM_END | map::SRAM_START..map::SRAM_END | map::WRAM_START..map::WRAM_END)
    }
}
