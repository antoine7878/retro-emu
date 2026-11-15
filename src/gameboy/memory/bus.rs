use std::vec::Drain;

use crate::gameboy::cartridge::Cartridge;
use crate::gameboy::config::Model;
use crate::gameboy::io::hdma::Hdma;
use crate::gameboy::io::interrupt::Interrupter;
use crate::gameboy::io::joypad::{Button, Joypad};
use crate::gameboy::io::link::Link;
use crate::gameboy::io::oam_dma::OamDma;
use crate::gameboy::io::ppu::Ppu;
use crate::gameboy::io::timer::Timer;
use crate::gameboy::memory::io_reg::IOReg;
use crate::gameboy::memory::io_reg::IORegister;
use crate::gameboy::memory::map;
use crate::gameboy::memory::wram::WRam;
use crate::gameboy::monitor::MemAccess;
use crate::save_state_struct;

use super::map::{OAM_END, OAM_START, VRAM_END, VRAM_START};

pub struct Bus {
    cartridge: Cartridge,
    ppu: Ppu,
    wram: WRam,
    hram: [u8; map::HRAM_SIZE],
    cycle_count: u64,
    interrupter: Interrupter,
    joypad: Joypad,
    timer: Timer,
    link: Link,
    trace: bool,
    oam_dma: OamDma,
    hdma: Hdma,
    accesses: Vec<MemAccess>,
    double_speed: bool,
    key0: IORegister<0x04>,
    key1: IORegister<0x01, 0x81>,
    boot_enabled: bool,
    model: Model,
}

#[rustfmt::skip]
save_state_struct!(Bus {cartridge, ppu, wram, hram, cycle_count, interrupter, joypad, timer, link, trace, oam_dma, hdma, double_speed, key0, key1, boot_enabled, model});

impl Default for Bus {
    fn default() -> Self {
        Self {
            cartridge: Cartridge::default(),
            ppu: Ppu::default(),
            wram: WRam::default(),
            hram: [0; map::HRAM_SIZE],
            cycle_count: 0,
            interrupter: Interrupter::default(),
            joypad: Joypad::default(),
            timer: Timer::default(),
            link: Link::default(),
            trace: false,
            oam_dma: OamDma::default(),
            hdma: Hdma::default(),
            accesses: Vec::new(),
            double_speed: false,
            key0: IORegister::default(),
            key1: IORegister::default(),
            boot_enabled: false,
            model: Model::Dmg,
        }
    }
}

impl Bus {
    pub fn reset(&mut self, model: Model) {
        self.oam_dma.reset();
        self.hdma.reset();
        self.ppu.reset(model);
        self.wram.reset();
        self.interrupter.reset();
        self.joypad.reset();
        self.timer.reset();
        self.link.reset(self.cgb_mode());
        self.boot_enabled = true;
        self.hram = [0; map::HRAM_SIZE];
        self.cycle_count = 0;
        self.accesses.clear();
        self.double_speed = false;
        self.model = model;
        self.key0.reset();
        self.key1.reset();
    }

    pub fn load_rom(&mut self, model: Model, rom: Vec<u8>) {
        self.cartridge = Cartridge::try_new(model, rom).unwrap_or_default();
    }

    fn read_boot_rom(&self, addr: u16) -> Option<u8> {
        if !self.boot_enabled || matches!(addr, map::BOOT_HEADER_START..map::BOOT_HEADER_END) {
            return None;
        }
        self.model.boot_rom().get(addr as usize).copied()
    }

    pub fn read_internal(&self, addr: u16) -> u8 {
        use map::*;

        match addr {
            ROM_START..ROM_END => self.read_boot_rom(addr).unwrap_or_else(|| self.cartridge.read(addr)),
            VRAM_START..VRAM_END => self.ppu.read(addr),
            SRAM_START..SRAM_END => self.cartridge.read(addr),
            WRAM_START..WRAM_END => self.wram.read(addr),
            ECHO_START..ECHO_END => self.read_internal(addr - 0x2000),
            OAM_START..OAM_END => self.ppu.read(addr),
            OAM_DMA => self.oam_dma.read(),
            PPU_START..PPU_END => self.ppu.read(addr),
            HRAM_START..HRAM_END => self.hram[(addr - HRAM_START) as usize],
            TIMER_START..TIMER_END => self.timer.read(addr),
            BOOT_DISABLE => 0xFF,
            IE | IF => self.interrupter.read(addr),
            JOYPAD => self.joypad.read(),
            SB | SC => self.link.read(addr),
            // TODO:Audio
            AUDIO_START..AUDIO_END => 0xFF,
            WAVE_START..WAVE_END => 0xFF,

            _ if !self.cgb_mode() => 0xFF,

            SVBK => self.wram.read(addr),
            VBK => self.ppu.read(addr),
            HDMA_START..HDMA_END => self.hdma.read(addr),
            PAL_START..PAL_END => self.ppu.read(addr),
            KEY0 => self.key0.read(),
            KEY1 => self.key1.read(),
            _ => 0xFF,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.tick();
        let value = if !(self.oam_dma.has_transferred() && self.oam_dma.conflict(addr)) {
            self.read_internal(addr)
        } else if matches!(addr, OAM_START..OAM_END) {
            0xFF
        } else {
            self.oam_dma.current_byte()
        };
        if self.trace {
            self.accesses.push(MemAccess::read(addr, value));
        }
        value
    }

    pub fn write_internal(&mut self, addr: u16, value: u8) {
        use map::*;

        match addr {
            ROM_START..ROM_END => self.cartridge.write(addr, value),
            VRAM_START..VRAM_END => self.ppu.write(addr, value),
            SRAM_START..SRAM_END => self.cartridge.write(addr, value),
            WRAM_START..WRAM_END => self.wram.write(addr, value),
            ECHO_START..ECHO_END => self.write_internal(addr - 0x2000, value),
            OAM_START..OAM_END => self.ppu.write(addr, value),
            OAM_DMA => self.oam_dma.write(value),
            STAT => self.ppu.write_stat(value, self.interrupter.ifl_mut()),
            PPU_START..PPU_END => self.ppu.write(addr, value),
            HRAM_START..HRAM_END => self.hram[(addr - HRAM_START) as usize] = value,
            TIMER_START..TIMER_END => self.timer.write(addr, value, self.interrupter.ifl_mut()),
            BOOT_DISABLE if value != 0 => self.boot_enabled = false,
            IE | IF => self.interrupter.write(addr, value),
            JOYPAD => self.joypad.write(value),
            SB | SC => self.link.write(addr, value),
            // TODO: Audio
            AUDIO_START..AUDIO_END => (),
            WAVE_START..WAVE_END => (),

            _ if !self.cgb_mode() => (),

            SVBK => self.wram.write(addr, value),
            VBK => self.ppu.write(addr, value),
            HDMA_START..HDMA_END => self.hdma.write(addr, value, self.ppu.in_hblank()),
            PAL_START..PAL_END => self.ppu.write(addr, value),
            KEY0 if self.boot_enabled => self.key0.write(value),
            KEY1 => self.key1.write(value),
            _ => (),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.tick();
        if self.trace {
            self.accesses.push(MemAccess::write(addr, value));
        }
        if self.oam_dma.has_transferred() && self.oam_dma.conflict(addr) {
            return;
        }
        self.write_internal(addr, value);
    }

    pub fn ticks(&mut self, m_cycle: u8) {
        for _ in 0..m_cycle {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        let dots = if self.double_speed { 2 } else { 4 };
        self.ppu.ticks(dots, self.interrupter.ifl_mut());
        if self.ppu.take_hblank_started() {
            self.hdma.hblank_started();
        }
        self.timer.tick(self.interrupter.ifl_mut());
        self.link.tick(self.interrupter.ifl_mut());
        self.cartridge.tick(dots);
        if let Some((src, dst)) = self.oam_dma.tick() {
            let byte = match src {
                VRAM_START..VRAM_END => self.ppu.read_vram_dma(src),
                _ => self.read_internal(src),
            };
            self.ppu.write_oam_dma(dst, byte);
            self.oam_dma.set_current_byte(byte);
        }
        self.cycle_count += 1;
    }

    pub fn run_hdma(&mut self) {
        if !self.ppu.in_hblank() {
            self.hdma.skip_block();
        }
        while let Some((src, dst)) = self.hdma.next() {
            let byte = if Hdma::readable(src) { self.read_internal(src) } else { 0xFF };
            self.ppu.write_vram_dma(dst, byte);
            if self.double_speed || dst & 1 == 1 {
                self.tick();
            }
        }
    }

    pub fn toggle_double_speed(&mut self) -> bool {
        if !self.key1.is_set(0) {
            return false;
        }
        self.double_speed = !self.double_speed;
        self.key1.clear(0);
        self.key1.write_bit(7, self.double_speed);
        return true;
    }

    pub fn press(&mut self, button: Button) {
        if self.joypad.press(button) {
            self.interrupter.ifl_mut().set(4);
        }
    }

    fn cgb_mode(&self) -> bool { self.model == Model::Cgb && (self.boot_enabled || self.key0.is_clear(2)) }
    pub fn take_redraw(&mut self) -> bool { self.ppu.take_redraw() }
    pub fn pending_interrupt(&self) -> u8 { self.interrupter.pending() }
    pub fn ack_interrupt(&mut self, bit: u8) { self.interrupter.ack(bit); }
    pub fn set_serial_print(&mut self, on: bool) { self.link.set_print(on); }
    pub fn drain_accesses(&mut self) -> Drain<'_, MemAccess> { self.accesses.drain(..) }
    pub const fn joypad(&self) -> &Joypad { &self.joypad }
    pub const fn double_speed(&self) -> bool { self.double_speed }
    pub const fn get_cycle_count(&self) -> u64 { self.cycle_count }
    pub const fn cartridge(&self) -> &Cartridge { &self.cartridge }
    pub const fn set_trace(&mut self, on: bool) { self.trace = on; }
    pub const fn get_pixels(&self) -> &[u8] { self.ppu.get_pixels() }
    pub const fn joypad_mut(&mut self) -> &mut Joypad { &mut self.joypad }
    pub const fn cartridge_mut(&mut self) -> &mut Cartridge { &mut self.cartridge }
}
