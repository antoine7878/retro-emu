use crate::gameboy::cartridge::Cartridge;
use crate::gameboy::config::ConsoleType;
use crate::gameboy::io::interrupt::Interrupter;
use crate::gameboy::io::joypad::Joypad;
use crate::gameboy::io::link::Link;
use crate::gameboy::io::ppu::Ppu;
use crate::gameboy::io::timer::Timer;
use crate::gameboy::memory::io_reg::IOReg;
use crate::gameboy::memory::map;
use crate::gameboy::memory::wram::WRam;
use crate::gameboy::monitor::MemAccess;

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
    accesses: Vec<MemAccess>,
}

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
            accesses: Vec::new(),
        }
    }
}

impl Bus {
    pub fn reset(&mut self, mode: &ConsoleType, rom: Vec<u8>) {
        self.ppu.reset(mode);
        self.wram.reset(mode);
        self.interrupter.reset(mode);
        self.joypad.reset(mode);
        self.timer.reset(mode);
        self.link.init_reg(mode);
        self.hram = [0; map::HRAM_SIZE];
        self.cycle_count = 0;
        self.accesses.clear();
        self.cartridge = Cartridge::try_new(rom).unwrap_or_default();
    }

    pub fn read_internal(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.cartridge.read(addr),
            map::VRAM_START..map::VRAM_END => self.ppu.read(addr),
            map::SRAM_START..map::SRAM_END => self.cartridge.read(addr),
            map::WRAM_START..map::WRAM_END | map::SVBK => self.wram.read(addr),
            map::ECHO_START..map::ECHO_END => self.read_internal(addr - 0x2000),
            map::OAM_START..map::OAM_END => self.ppu.read(addr),
            map::NOT_START..map::NOT_END => 0xFF,
            map::PPU_START..map::PPU_END | map::VBK => self.ppu.read(addr),
            map::DMA_START..map::DMA_END | map::PAL_START..map::PAL_END => self.ppu.read(addr),
            map::HRAM_START..map::HRAM_END => self.hram[(addr - map::HRAM_START) as usize],
            map::TIMER_START..map::TIMER_END => self.timer.read(addr),
            // TODO:Audio
            map::AUDIO_START..map::AUDIO_END => 0xFF,
            map::WAVE_START..map::WAVE_END => 0xFF,
            // TODO:keypad
            map::KEY0 | map::KEY1 => 0xFF,
            map::BOOT_DISABLE => 0xFF,

            map::IE | map::IF => self.interrupter.read(addr),
            map::JOYPAD => self.joypad.read(),
            map::SB | map::SC => self.link.read(addr),
            map::U1
            | map::U2S..map::U2E
            | map::U3S..map::U3E
            | map::U5
            | map::U6S..map::U6E
            | map::U7S..map::U7E
            | map::U8S..map::U8E => 0xFF,
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
            map::ROM_START..map::ROM_END => self.cartridge.write(addr, value),
            map::VRAM_START..map::VRAM_END => self.ppu.write(addr, value),
            map::SRAM_START..map::SRAM_END => self.cartridge.write(addr, value),
            map::WRAM_START..map::WRAM_END | map::SVBK => self.wram.write(addr, value),
            map::ECHO_START..map::ECHO_END => self.write_internal(addr - 0x2000, value),
            map::OAM_START..map::OAM_END => self.ppu.write(addr, value),
            map::NOT_START..map::NOT_END => (),
            map::PPU_START..map::PPU_END | map::VBK => self.ppu.write(addr, value),
            map::DMA_START..map::DMA_END | map::PAL_START..map::PAL_END => self.ppu.write(addr, value),
            map::HRAM_START..map::HRAM_END => self.hram[(addr - map::HRAM_START) as usize] = value,
            map::TIMER_START..map::TIMER_END => self.timer.write(addr, value),
            // TODO: Audio
            map::AUDIO_START..map::AUDIO_END => (),
            map::WAVE_START..map::WAVE_END => (),
            // TODO: Keypad
            map::KEY0 | map::KEY1 => (),
            map::BOOT_DISABLE => (),

            map::IE | map::IF => self.interrupter.write(addr, value),
            map::JOYPAD => self.joypad.write(value),
            map::SB | map::SC => self.link.write(addr, value),
            map::U1
            | map::U2S..map::U2E
            | map::U3S..map::U3E
            | map::U5
            | map::U6S..map::U6E
            | map::U7S..map::U7E
            | map::U8S..map::U8E => (),
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

    pub fn set_serial_print(&mut self, on: bool) { self.link.set_print(on); }
    pub fn drain_accesses(&mut self) -> std::vec::Drain<'_, MemAccess> { self.accesses.drain(..) }
    pub fn cartridge(&self) -> &Cartridge { &self.cartridge }
}
