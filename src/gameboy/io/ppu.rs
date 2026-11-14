use crate::gameboy::config::ConsoleType;
use crate::gameboy::io::interrupt::InterruptFlag;
use crate::gameboy::memory::io_reg::{IOReg, IORegister};
use crate::gameboy::memory::map;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const PPU_OFF: u8 = 7;
const WINDOW_MAP: u8 = 6;
const WINDOW_ENABLE: u8 = 5;
const BG_WIN_TILE_DATA: u8 = 4;
const BG_MAP: u8 = 3;
const OBJ_SIZE: u8 = 2;
const OBJ_ENABLE: u8 = 1;
const BG_WIN_ENABLE: u8 = 0;
const CLAIMED: u8 = 0x80;

const OBJ_BG_PRIORITY: u8 = 0x80;
const OBJ_FLIP_Y: u8 = 0x40;
const OBJ_FLIP_X: u8 = 0x20;
const OBJ_PALETTE: u8 = 0x10;

const PALETTE: [[u8; 3]; 4] = [[0x9B, 0xBC, 0x0F], [0x8B, 0xAC, 0x0F], [0x30, 0x62, 0x30], [0x0F, 0x38, 0x0F]];

const fn color_index(lo: u8, hi: u8, bit: u8) -> u8 { (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1) }

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

impl PpuMode {
    const fn end_dot(self) -> u16 {
        match self {
            Self::OamScan => 80,
            Self::Drawing => 252,
            Self::HBlank | Self::VBlank => 456,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum TileKind {
    Obj,
    BgWin,
}

pub struct Ppu {
    pixels: [[[u8; 3]; WIDTH]; HEIGHT],
    redraw: bool,
    dot: u16,

    stat_line: bool,
    mode: PpuMode,
    dmg: bool,

    window_line: u16,
    wy_triggered: bool,

    first_line: bool,
    skip_frame: bool,

    selected_objects: [[u8; 4]; 10],
    selected_objects_len: usize,

    vram: [[u8; map::VRAM_SIZE]; 2],
    oam: [u8; map::OAM_SIZE],
    lcdc: IORegister<{ map::LCDC }, 0xFF>,
    stat: IORegister<{ map::STAT }, 0x78>,
    scy: IORegister<{ map::SCY }, 0xFF>,
    scx: IORegister<{ map::SCX }, 0xFF>,
    ly: IORegister<{ map::LY }, 0x00, 0xFF>,
    lyc: IORegister<{ map::LYC }, 0xFF>,
    bgp: IORegister<{ map::BGP }, 0xFF>,
    obp0: IORegister<{ map::OBP0 }, 0xFF>,
    obp1: IORegister<{ map::OBP1 }, 0xFF>,
    wy: IORegister<{ map::WY }, 0xFF>,
    wx: IORegister<{ map::WX }, 0xFF>,
    vbk: IORegister<{ map::VBK }, 0x01>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            pixels: [[[0; 3]; WIDTH]; HEIGHT],
            dot: 0,
            vram: [[0; map::VRAM_SIZE]; 2],
            oam: [0; map::OAM_SIZE],
            selected_objects: [[0; 4]; 10],
            selected_objects_len: 0,
            stat_line: false,
            redraw: false,
            window_line: 0,
            wy_triggered: false,
            first_line: false,
            skip_frame: false,
            lcdc: IORegister::default(),
            stat: IORegister::default(),
            scy: IORegister::default(),
            scx: IORegister::default(),
            ly: IORegister::default(),
            lyc: IORegister::default(),
            bgp: IORegister::default(),
            obp0: IORegister::default(),
            obp1: IORegister::default(),
            wy: IORegister::default(),
            wx: IORegister::default(),
            vbk: IORegister::default(),
            mode: PpuMode::HBlank,
            dmg: true,
        }
    }
}

impl Ppu {
    pub fn reset(&mut self, mode: &ConsoleType) {
        *self = Self::default();
        self.dmg = matches!(mode, ConsoleType::Dmg);
        self.lcdc.reset(mode);
        self.stat.reset(mode);
        self.scy.reset(mode);
        self.scx.reset(mode);
        self.ly.reset(mode);
        self.lyc.reset(mode);
        self.bgp.reset(mode);
        self.obp0.reset(mode);
        self.obp1.reset(mode);
        self.wy.reset(mode);
        self.wx.reset(mode);
        self.vbk.reset(mode);
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            map::VRAM_START..map::VRAM_END => self.read_vram(addr),
            map::OAM_START..map::OAM_END => self.read_oam(addr),
            // TODO: DMA CGB
            map::DMA_START..map::DMA_END => 0xFF,
            map::PAL_START..map::PAL_END => 0xFF,
            map::LCDC => self.lcdc.read(),
            map::STAT => {
                self.stat.read() & 0xF8
                    | ((self.ly.read_raw() == self.lyc.read_raw()) as u8) << 2
                    | self.mode as u8
            }
            map::SCY => self.scy.read(),
            map::SCX => self.scx.read(),
            map::LY => self.ly.read(),
            map::LYC => self.lyc.read(),
            map::BGP => self.bgp.read(),
            map::OBP0 => self.obp0.read(),
            map::OBP1 => self.obp1.read(),
            map::WY => self.wy.read(),
            map::WX => self.wx.read(),
            map::VBK => self.vbk.read(),
            _ => unreachable!("Address not in PPU"),
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        match self.mode {
            PpuMode::Drawing => 0xFF,
            _ => self.read_vram_dma(addr),
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        match self.mode {
            PpuMode::OamScan | PpuMode::Drawing => 0xFF,
            _ => self.oam[(addr - map::OAM_START) as usize],
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            map::VRAM_START..map::VRAM_END => self.write_vram(addr, value),
            map::OAM_START..map::OAM_END => self.write_oam(addr, value),
            // TODO: DMA CGB
            map::DMA_START..map::DMA_END => (),
            map::PAL_START..map::PAL_END => (),
            map::LCDC => self.write_lcdc(value),
            map::STAT => self.stat.write(value),
            map::SCY => self.scy.write(value),
            map::SCX => self.scx.write(value),
            map::LY => self.ly.write(value),
            map::LYC => self.lyc.write(value),
            map::BGP => self.bgp.write(value),
            map::OBP0 => self.obp0.write(value),
            map::OBP1 => self.obp1.write(value),
            map::WY => self.wy.write(value),
            map::WX => self.wx.write(value),
            map::VBK => self.vbk.write(value),
            _ => unreachable!("Address not in PPU"),
        }
    }

    pub fn write_stat(&mut self, value: u8, ifl: &mut InterruptFlag) {
        if self.lcdc.is_clear(PPU_OFF) {
            self.stat.write(value);
            return;
        }
        if self.dmg {
            let lyc_match = self.ly.read_raw() == self.lyc.read_raw();
            let line = matches!(self.mode, PpuMode::HBlank | PpuMode::VBlank) || lyc_match;
            if line && !self.stat_line {
                ifl.set(1);
            }
            self.stat_line = line;
        }
        self.stat.write(value);
        self.update_stat_line(ifl);
    }

    fn write_lcdc(&mut self, value: u8) {
        let was_on = self.lcdc.is_set(PPU_OFF);
        self.lcdc.write(value);
        match (was_on, self.lcdc.is_set(PPU_OFF)) {
            (true, false) => self.lcd_off(),
            (false, true) => self.lcd_on(),
            _ => (),
        }
    }

    fn lcd_off(&mut self) {
        self.dot = 0;
        self.ly.write_raw(0);
        self.mode = PpuMode::HBlank;
        self.stat_line = false;
        self.window_line = 0;
        self.wy_triggered = false;
        self.first_line = false;
        self.skip_frame = false;
        self.pixels = [[PALETTE[0]; WIDTH]; HEIGHT];
        self.redraw = true;
    }

    fn lcd_on(&mut self) {
        self.dot = 4;
        self.ly.write_raw(0);
        self.mode = PpuMode::HBlank;
        self.first_line = true;
        self.skip_frame = true;
    }

    const fn end_dot(&self) -> u16 { if self.first_line { PpuMode::OamScan.end_dot() } else { self.mode.end_dot() } }

    pub fn write_vram(&mut self, addr: u16, value: u8) {
        if self.mode != PpuMode::Drawing {
            let bank = self.vram_bank();
            self.vram[bank][(addr - map::VRAM_START) as usize] = value;
        }
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        match self.mode {
            PpuMode::OamScan | PpuMode::Drawing => (),
            _ => self.oam[(addr - map::OAM_START) as usize] = value,
        }
    }

    pub fn read_vram_raw(&self, addr: u16) -> u8 { self.vram[0][(addr - map::VRAM_START) as usize] }

    pub fn read_vram_dma(&self, addr: u16) -> u8 { self.vram[self.vram_bank()][(addr - map::VRAM_START) as usize] }

    pub fn write_oam_dma(&mut self, i: u16, value: u8) { self.oam[i as usize] = value; }

    fn vram_bank(&self) -> usize { self.vbk.read_raw() as usize & 0x01 }

    fn tile_map(&self, bit: u8) -> u16 { if self.lcdc.is_set(bit) { 0x9C00 } else { 0x9800 } }

    fn tile_row(&self, id: u8, kind: TileKind, row: u8) -> (u8, u8) {
        let unsigned = kind == TileKind::Obj || self.lcdc.is_set(BG_WIN_TILE_DATA);
        let tile = if unsigned { id as usize * 16 } else { (0x1000 + (id as i8 as i32) * 16) as usize };
        let start = tile + row as usize * 2;
        (self.vram[0][start], self.vram[0][start + 1])
    }

    fn set_mode(&mut self, mode: PpuMode, ifl: &mut InterruptFlag) {
        self.mode = mode;
        self.update_stat_line(ifl);
    }

    fn update_stat_line(&mut self, ifl: &mut InterruptFlag) {
        let lyc_match = self.ly.read_raw() == self.lyc.read_raw();

        let stat = self.stat.read();
        let line = (stat & 0x08 != 0 && self.mode == PpuMode::HBlank)
            || (stat & 0x10 != 0 && self.mode == PpuMode::VBlank)
            || (stat & 0x20 != 0 && self.mode == PpuMode::OamScan)
            || (stat & 0x40 != 0 && lyc_match);

        if line && !self.stat_line {
            ifl.set(1);
        }
        self.stat_line = line;
    }

    fn tick(&mut self, ifl: &mut InterruptFlag) {
        self.dot += 1;
        if self.dot == self.end_dot() {
            match self.mode {
                PpuMode::HBlank if self.first_line => {
                    self.first_line = false;
                    self.scan_oam();
                    self.set_mode(PpuMode::Drawing, ifl);
                }
                PpuMode::OamScan => {
                    self.scan_oam();
                    self.set_mode(PpuMode::Drawing, ifl);
                }
                PpuMode::Drawing => {
                    self.render_scanline();
                    self.set_mode(PpuMode::HBlank, ifl);
                }
                PpuMode::HBlank | PpuMode::VBlank => {
                    self.dot = 0;
                    self.next_line(ifl);
                }
            }
        }
        self.update_stat_line(ifl);
    }

    fn scan_oam(&mut self) {
        let line = self.ly.read() as i16 + 16;
        let height: i16 = if self.lcdc.is_set(OBJ_SIZE) { 16 } else { 8 };

        self.selected_objects_len = 0;
        for o in self.oam.as_chunks::<4>().0 {
            let y = o[0] as i16;
            if y <= line && line < y + height {
                self.selected_objects[self.selected_objects_len] = *o;
                self.selected_objects_len += 1;
                if self.selected_objects_len == 10 {
                    break;
                }
            }
        }
        self.selected_objects[..self.selected_objects_len].sort_by_key(|o| o[1]);
    }

    fn render_scanline(&mut self) {
        let mut line_buffer = [0u8; WIDTH];
        self.render_background(&mut line_buffer);
        self.render_objects(&mut line_buffer);
        let bgp = if self.lcdc.is_set(BG_WIN_ENABLE) { self.bgp.read() } else { 0 };
        let ly = self.ly.read() as usize;
        for (x, v) in line_buffer.iter().enumerate() {
            let shade = if v & CLAIMED != 0 { v & 0b11 } else { (bgp >> ((v & 0b11) * 2)) & 0b11 };
            self.pixels[ly][x] = PALETTE[shade as usize];
        }
    }

    fn render_background(&mut self, line_buffer: &mut [u8; WIDTH]) {
        let ly = self.ly.read();
        let scx = self.scx.read();
        let scy = self.scy.read();
        let wx = self.wx.read();

        if self.wy.read() == ly {
            self.wy_triggered = true;
        }

        if !self.lcdc.is_set(BG_WIN_ENABLE) {
            line_buffer.fill(0);
            return;
        }

        let window_visible = self.lcdc.is_set(WINDOW_ENABLE) && self.wy_triggered && wx <= 166;
        let win_x_start = wx as i16 - 7;
        let win_map = self.tile_map(WINDOW_MAP);
        let bg_map = self.tile_map(BG_MAP);
        let by = ly.wrapping_add(scy) as u16;
        let mut drew_window = false;

        for (x, pixel) in line_buffer.iter_mut().enumerate() {
            let x = x as i16;

            *pixel = if window_visible && x >= win_x_start {
                let wx = (x - win_x_start) as u16;
                let wy = self.window_line;
                let tile = self.read_vram_raw(win_map + (wy / 8) * 32 + wx / 8);
                drew_window = true;
                self.bg_pixel(tile, wx % 8, wy % 8)
            } else {
                let bx = (x as u8).wrapping_add(scx) as u16;
                let tile = self.read_vram_raw(bg_map + (by / 8) * 32 + bx / 8);
                self.bg_pixel(tile, bx % 8, by % 8)
            };
        }

        if drew_window {
            self.window_line += 1;
        }
    }

    fn bg_pixel(&self, tile: u8, px: u16, py: u16) -> u8 {
        let (lo, hi) = self.tile_row(tile, TileKind::BgWin, py as u8);
        color_index(lo, hi, 7 - px as u8)
    }

    fn render_objects(&self, line_buffer: &mut [u8; WIDTH]) {
        if self.lcdc.is_clear(OBJ_ENABLE) {
            return;
        }
        let height = if self.lcdc.is_set(OBJ_SIZE) { 16 } else { 8 };
        let ly = self.ly.read();

        for &[y, x, tile_id, attr] in &self.selected_objects[..self.selected_objects_len] {
            let row = ly + 16 - y;
            let row = if attr & OBJ_FLIP_Y != 0 { height - 1 - row } else { row };
            let id = if height == 16 { (tile_id & 0xFE) | (row >> 3) } else { tile_id };
            let (lo, hi) = self.tile_row(id, TileKind::Obj, row % 8);
            let palette = if attr & OBJ_PALETTE != 0 { self.obp1.read() } else { self.obp0.read() };

            for lx in 0..8u8 {
                let Some(sx) = (x as usize + lx as usize).checked_sub(8).filter(|&sx| sx < WIDTH) else { continue };
                let bit = if attr & OBJ_FLIP_X != 0 { lx } else { 7 - lx };
                let color = color_index(lo, hi, bit);
                let pixel = &mut line_buffer[sx];
                let hidden = color == 0 || *pixel & CLAIMED != 0 || (attr & OBJ_BG_PRIORITY != 0 && *pixel != 0);
                if !hidden {
                    *pixel = CLAIMED | ((palette >> (color * 2)) & 0b11);
                }
            }
        }
    }

    fn next_line(&mut self, ifl: &mut InterruptFlag) {
        self.ly += 1;
        match self.ly.read() {
            144 => {
                self.set_mode(PpuMode::VBlank, ifl);
                ifl.set(0);
                self.redraw = !std::mem::take(&mut self.skip_frame);
            }
            154 => {
                self.ly.write_raw(0);
                self.window_line = 0;
                self.wy_triggered = false;
                self.set_mode(PpuMode::OamScan, ifl);
            }
            0..144 => self.set_mode(PpuMode::OamScan, ifl),
            _ => (),
        }
    }

    pub fn ticks(&mut self, dots: u8, ifl: &mut InterruptFlag) {
        if self.lcdc.is_clear(PPU_OFF) {
            return;
        }
        if self.dot + (dots as u16) < self.end_dot() {
            self.dot += dots as u16;
            self.update_stat_line(ifl);
            return;
        }
        for _ in 0..dots {
            self.tick(ifl);
        }
    }

    pub fn take_redraw(&mut self) -> bool { std::mem::take(&mut self.redraw) }
    pub const fn get_pixels(&self) -> &[u8] { self.pixels.as_flattened().as_flattened() }
}
