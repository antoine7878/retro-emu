use std::time::{SystemTime, UNIX_EPOCH};

use crate::gameboy::cartridge::header::CartridgeHeader;
use crate::gameboy::memory::map;
use crate::save_state_struct;

use super::{Banks, Mbc};

pub struct Mbc3 {
    mem: Banks,
    selected_rom_bank: usize,
    selected_ram_bank: usize,
    prev_latch_reg: u8,
    ram_enabled: bool,
    has_rtc: bool,

    time: [u8; 5],
    latch: [u8; 5],

    rtc_clock: u32,
}

#[rustfmt::skip]
save_state_struct!(Mbc3 { mem, selected_rom_bank, selected_ram_bank, prev_latch_reg, ram_enabled, has_rtc, time, latch, rtc_clock });

const TIME_MASKS: [u8; 5] = [0x3F, 0x3F, 0x1F, 0xFF, 0xC1];

impl Mbc for Mbc3 {
    fn try_from_header(rom: Vec<u8>, header: &CartridgeHeader) -> Option<Self> {
        Some(Self {
            mem: Banks::from_header(rom, header),
            has_rtc: header.has_rtc(),
            selected_rom_bank: 1,
            selected_ram_bank: 0,
            prev_latch_reg: 0xFF,
            ram_enabled: false,
            time: [0; 5],
            latch: [0; 5],
            rtc_clock: 0,
        })
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            map::ROM_START..map::ROM_END => self.mem.read_rom(self.selected_rom_bank(addr), addr),
            map::SRAM_START..map::SRAM_END
                if self.ram_enabled && self.has_rtc && (0x08..=0x0C).contains(&self.selected_ram_bank) =>
            {
                self.latch[self.selected_ram_bank - 0x08]
            }
            map::SRAM_START..map::SRAM_END
                if self.ram_enabled && self.mem.has_ram() && (0x00..=0x07).contains(&self.selected_ram_bank) =>
            {
                self.mem.read_ram(self.selected_ram_bank, addr)
            }
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..0x2000 => self.ram_enabled = value & 0x0F == 0x0A && (self.mem.has_ram() || self.has_rtc),
            0x2000..0x4000 => self.selected_rom_bank = usize::max(value as usize & 0x7F, 1),
            0x4000..0x6000 => self.selected_ram_bank = value as usize & 0x0F,
            0x6000..0x8000 => {
                if self.prev_latch_reg == 0x00 && value == 0x01 {
                    self.latch = self.time;
                }
                self.prev_latch_reg = value;
            }
            map::SRAM_START..map::SRAM_END
                if self.ram_enabled && self.has_rtc && (0x08..=0x0C).contains(&self.selected_ram_bank) =>
            {
                let idx = self.selected_ram_bank - 0x08;
                self.time[idx] = value & TIME_MASKS[idx];
                if idx == 0 {
                    self.rtc_clock = 0;
                }
            }
            map::SRAM_START..map::SRAM_END
                if self.ram_enabled && self.mem.has_ram() && (0x00..=0x07).contains(&self.selected_ram_bank) =>
            {
                self.mem.write_ram(self.selected_ram_bank, addr, value)
            }
            _ => (),
        }
    }

    fn rom(&self) -> Vec<u8> { self.mem.rom() }

    fn ram(&self) -> Vec<u8> {
        let ram = self.mem.ram();
        let mut out = vec![0u8; 8 + 5 + ram.len()];
        let save_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs().to_be_bytes();
        out[..8].copy_from_slice(&save_time);
        out[8..13].copy_from_slice(&self.time);
        out[13..].copy_from_slice(&ram);
        out
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        let mut ram = ram;
        ram.resize(ram.len().max(8 + 5), 0);
        let time_of_save = u64::from_be_bytes(ram[..8].try_into().unwrap_or_default());
        let saved_rtc = self.seconds_of_reg(&ram[8..13]);

        let halted = ram[12] & 0x40 != 0;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let elapsed = if halted { 0 } else { now.saturating_sub(time_of_save) };
        let seconds = saved_rtc + elapsed;
        self.init_reg_with(seconds, ram[12]);
        self.mem.load_ram(ram[13..].to_vec());
    }

    fn tick(&mut self, dots: u8) {
        if self.time[4] & 0x40 != 0 {
            return;
        };
        self.rtc_clock += dots as u32;
        if self.rtc_clock >= 4_194_304 {
            self.inc_seconds();
            self.rtc_clock = 0;
        }
    }
}

impl Mbc3 {
    fn inc_seconds(&mut self) {
        self.time[0] = (self.time[0] + 1) & TIME_MASKS[0];
        if self.time[0] == 60 {
            self.time[0] = 0;
            self.inc_minutes()
        }
    }

    fn inc_minutes(&mut self) {
        self.time[1] = (self.time[1] + 1) & TIME_MASKS[1];
        if self.time[1] == 60 {
            self.time[1] = 0;
            self.inc_hours()
        }
    }

    fn inc_hours(&mut self) {
        self.time[2] = (self.time[2] + 1) & TIME_MASKS[2];
        if self.time[2] == 24 {
            self.time[2] = 0;
            self.inc_days()
        }
    }

    fn inc_days(&mut self) {
        let day = (((self.time[4] as u16 & 0x01) << 8) | self.time[3] as u16) + 1;
        self.time[3] = day as u8;
        self.time[4] ^= (self.time[4] ^ ((day >> 8) as u8)) & 0x01;
        self.time[4] |= ((day >= 512) as u8) << 7;
    }

    fn selected_rom_bank(&self, addr: u16) -> usize {
        match addr {
            map::ROM_START..map::ROM_HALF => 0,
            map::ROM_HALF..map::ROM_END => self.selected_rom_bank,
            _ => unreachable!(),
        }
    }

    fn seconds_of_reg(&self, time: &[u8]) -> u64 {
        let [sec, min, hour, day, flags] = *time else { return 0 };
        let day = ((flags as u64 & 0x01) << 8) | (day as u64);
        (sec as u64) + (min as u64 * 60) + (hour as u64 * 3600) + (day * 86400)
    }

    fn init_reg_with(&mut self, seconds: u64, f: u8) {
        let [sec, min, hour, day, flags] = &mut self.time;
        *sec = (seconds % 60) as u8;
        *min = ((seconds / 60) % 60) as u8;
        *hour = ((seconds / 3600) % 24) as u8;
        let d = (seconds / 86400) as u16;
        *day = (d & 0xFF) as u8;
        *flags = (((d > 511) as u8) << 7) | ((d >> 8) as u8 & 1) | (f & 0xC0);
    }
}
