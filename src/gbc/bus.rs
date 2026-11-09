use crate::debugger::MemAccess;

use super::cart::Cartridge;
use super::config::GCMode;
use super::interrupt::Interrupter;
use super::io_reg::IOReg;
use super::joypad::Joypad;
use super::link::Link;
use super::memory_map::{
    AUDIO_END, AUDIO_START, BOOT_DISABLE, CRAM_END, CRAM_START, DMA_END, DMA_START, ECHO_END, ECHO_START, HRAM_END,
    HRAM_SIZE, HRAM_START, IE, IF, JOYPAD, KEY0, KEY1, LY, NOT_END, NOT_START, OAM_END, OAM_START, PAL_END, PAL_START,
    PPU_END, PPU_START, ROM_END, ROM_START, SB, SC, SVBK, TIMER_END, TIMER_START, U1, U2E, U2S, U3E, U3S, U5, U6E, U6S,
    U7E, U7S, U8E, U8S, VBK, VRAM_END, VRAM_START, WAVE_END, WAVE_START, WRAM_END, WRAM_START,
};
use super::ppu::Ppu;
use super::timer::Timer;
use super::wram::WRam;

pub struct Bus {
    cart: Cartridge,
    ppu: Ppu,
    wram: WRam,
    hram: [u8; HRAM_SIZE],
    cycle_count: u64,
    interrupter: Interrupter,
    joypad: Joypad,
    timer: Timer,
    link: Link,
    trace: bool,
    doctor: bool,
    accesses: Vec<MemAccess>,
}

impl Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            cart: Cartridge::new(rom),
            ppu: Ppu::default(),
            wram: WRam::default(),
            hram: [0; HRAM_SIZE],
            cycle_count: 0,
            interrupter: Interrupter::default(),
            joypad: Joypad::default(),
            timer: Timer::default(),
            link: Link::default(),
            trace: false,
            doctor: false,
            accesses: Vec::new(),
        }
    }

    pub fn reset(&mut self, mode: &GCMode) {
        self.ppu.reset(mode);
        self.wram.reset(mode);
        self.interrupter.reset(mode);
        self.joypad.reset(mode);
        self.timer.reset(mode);
        self.link.init_reg(mode);
        self.cart.reset();
        self.hram = [0; HRAM_SIZE];
        self.cycle_count = 0;
        self.accesses.clear();
    }

    pub fn read_internal(&self, addr: u16) -> u8 {
        match addr {
            ROM_START..ROM_END => self.cart.read_rom(addr),
            VRAM_START..VRAM_END => self.ppu.read(addr),
            CRAM_START..CRAM_END => self.cart.read_ram(addr),
            WRAM_START..WRAM_END | SVBK => self.wram.read(addr),
            ECHO_START..ECHO_END => self.read_internal(addr - 0x2000),
            OAM_START..OAM_END => self.ppu.read(addr),
            NOT_START..NOT_END => 0xFF,
            LY if self.doctor => 0x90,
            PPU_START..PPU_END | VBK => self.ppu.read(addr),
            DMA_START..DMA_END | PAL_START..PAL_END => self.ppu.read(addr),
            HRAM_START..HRAM_END => self.hram[(addr - HRAM_START) as usize],
            TIMER_START..TIMER_END => self.timer.read(addr),
            // TODO:
            AUDIO_START..AUDIO_END => 0xFF,
            KEY0 | KEY1 => 0xFF,
            WAVE_START..WAVE_END => 0xFF,
            BOOT_DISABLE => 0xFF,

            IE | IF => self.interrupter.read(addr),
            JOYPAD => self.joypad.read(),
            SB | SC => self.link.read(addr),
            U1 | U2S..U2E | U3S..U3E | U5 | U6S..U6E | U7S..U7E | U8S..U8E => 0xFF,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.tick();
        let value = self.read_internal(addr);
        if self.trace {
            self.accesses.push(MemAccess::read(addr, value));
        }
        value
    }

    pub fn write_internal(&mut self, addr: u16, value: u8) {
        match addr {
            ROM_START..ROM_END => (),
            VRAM_START..VRAM_END => self.ppu.write(addr, value),
            CRAM_START..CRAM_END => self.cart.write_ram(addr, value),
            WRAM_START..WRAM_END | SVBK => self.wram.write(addr, value),
            ECHO_START..ECHO_END => self.write_internal(addr - 0x2000, value),
            OAM_START..OAM_END => self.ppu.write(addr, value),
            NOT_START..NOT_END => (),
            PPU_START..PPU_END | VBK => self.ppu.write(addr, value),
            DMA_START..DMA_END | PAL_START..PAL_END => self.ppu.write(addr, value),
            HRAM_START..HRAM_END => self.hram[(addr - HRAM_START) as usize] = value,
            TIMER_START..TIMER_END => self.timer.write(addr, value),
            // TODO:
            AUDIO_START..AUDIO_END => (),
            KEY0 | KEY1 => (),
            WAVE_START..WAVE_END => (),
            BOOT_DISABLE => (),

            IE | IF => self.interrupter.write(addr, value),
            JOYPAD => self.joypad.write(value),
            SB | SC => self.link.write(addr, value),
            U1 | U2S..U2E | U3S..U3E | U5 | U6S..U6E | U7S..U7E | U8S..U8E => (),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        self.tick();
        if self.trace {
            self.accesses.push(MemAccess::write(addr, value));
        }
        self.write_internal(addr, value);
    }

    pub fn ticks(&mut self, m_cycle: u8) {
        for _ in 0..m_cycle {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        let dots = 4;
        self.ppu.ticks(dots, self.interrupter.ifl_mut());
        self.timer.tick(self.interrupter.ifl_mut());
        self.cycle_count += 1;
    }

    pub fn pending_interrupt(&self) -> u8 { self.interrupter.pending() }
    pub fn ack_interrupt(&mut self, bit: u8) { self.interrupter.ack(bit); }
    pub const fn get_cycle_count(&self) -> u64 { self.cycle_count }
    pub const fn get_pixels(&self) -> &[u8] { self.ppu.get_pixels() }
    pub fn take_redraw(&mut self) -> bool { self.ppu.take_redraw() }
    pub fn set_trace(&mut self, on: bool) { self.trace = on; }
    pub fn set_doctor(&mut self, on: bool) { self.doctor = on; }
    pub fn drain_accesses(&mut self) -> std::vec::Drain<'_, MemAccess> { self.accesses.drain(..) }
}
