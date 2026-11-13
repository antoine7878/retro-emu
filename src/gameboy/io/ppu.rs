use crate::gameboy::config::ConsoleType;
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

const fn palette(brightness: u8) -> [[u8; 3]; 4] {
    const BASE: [[u8; 3]; 4] = [[0x9B, 0xBC, 0x0F], [0x8B, 0xAC, 0x0F], [0x30, 0x62, 0x30], [0x0F, 0x38, 0x0F]];
    let mut out = [[0; 3]; 4];
    let mut i = 0;
    while i < 4 {
        let mut c = 0;
        while c < 3 {
            out[i][c] = (BASE[i][c] as u16 * brightness as u16 / 255) as u8;
            c += 1;
        }
        i += 1;
    }
    out
}

const PALETTE_ORIGINAL: [[u8; 3]; 4] = palette(255);

#[derive(PartialEq, Clone, Eq)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

pub struct Ppu {
    pixels: [[[u8; 3]; WIDTH]; HEIGHT],
    redraw: bool,
    dot: u16,

    stat_line: bool,
    mode: PpuMode,

    window_line: u16,
    wy_triggered: bool,

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
    opb0: IORegister<{ map::OBP0 }, 0xFF>,
    opb1: IORegister<{ map::OBP1 }, 0xFF>,
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
            lcdc: IORegister::default(),
            stat: IORegister::default(),
            scy: IORegister::default(),
            scx: IORegister::default(),
            ly: IORegister::default(),
            lyc: IORegister::default(),
            bgp: IORegister::default(),
            opb0: IORegister::default(),
            opb1: IORegister::default(),
            wy: IORegister::default(),
            wx: IORegister::default(),
            vbk: IORegister::default(),
            mode: PpuMode::HBlank,
        }
    }
}

impl Ppu {
    pub fn reset(&mut self, mode: &ConsoleType) {
        self.lcdc.reset(mode);
        self.stat.reset(mode);
        self.scy.reset(mode);
        self.scx.reset(mode);
        self.ly.reset(mode);
        self.lyc.reset(mode);
        self.bgp.reset(mode);
        self.opb0.reset(mode);
        self.opb1.reset(mode);
        self.wy.reset(mode);
        self.wx.reset(mode);
        self.vbk.reset(mode);
        self.pixels = [[[0; 3]; WIDTH]; HEIGHT];
        self.dot = 0;
        self.vram = [[0; map::VRAM_SIZE]; 2];
        self.oam = [0; map::OAM_SIZE];
        self.selected_objects = [[0; 4]; 10];
        self.selected_objects_len = 0;
        self.stat_line = false;
        self.redraw = false;
        self.window_line = 0;
        self.wy_triggered = false;
        self.mode = PpuMode::HBlank;
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
                    | self.mode.clone() as u8
            }
            map::SCY => self.scy.read(),
            map::SCX => self.scx.read(),
            map::LY => self.ly.read(),
            map::LYC => self.lyc.read(),
            map::BGP => self.bgp.read(),
            map::OBP0 => self.opb0.read(),
            map::OBP1 => self.opb1.read(),
            map::WY => self.wy.read(),
            map::WX => self.wx.read(),
            map::VBK => self.vbk.read(),
            _ => unreachable!("Address not in PPU"),
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        match self.mode {
            PpuMode::Drawing => 0xFF,
            _ => self.vram[self.vbk.read_raw() as usize & 0x01][(addr - map::VRAM_START) as usize],
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
            map::LCDC => self.lcdc.write(value),
            map::STAT => self.stat.write(value),
            map::SCY => self.scy.write(value),
            map::SCX => self.scx.write(value),
            map::LY => self.ly.write(value),
            map::LYC => self.lyc.write(value),
            map::BGP => self.bgp.write(value),
            map::OBP0 => self.opb0.write(value),
            map::OBP1 => self.opb1.write(value),
            map::WY => self.wy.write(value),
            map::WX => self.wx.write(value),
            map::VBK => self.vbk.write(value),
            _ => unreachable!("Address not in PPU"),
        }
    }

    pub fn write_vram(&mut self, addr: u16, value: u8) {
        match self.mode {
            PpuMode::Drawing => (),
            _ => self.vram[self.vbk.read_raw() as usize & 0x01][(addr - map::VRAM_START) as usize] = value,
        }
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        match self.mode {
            PpuMode::OamScan | PpuMode::Drawing => (),
            _ => self.oam[(addr - map::OAM_START) as usize] = value,
        }
    }

    pub fn read_vram_raw(&self, addr: u16) -> u8 { self.vram[0][(addr - map::VRAM_START) as usize] }

    pub fn read_vram_dma(&self, addr: u16) -> u8 {
        self.vram[self.vbk.read_raw() as usize & 0x01][(addr - map::VRAM_START) as usize]
    }

    pub fn write_oam_dma(&mut self, i: u16, value: u8) { self.oam[i as usize] = value; }

    fn tile_offset(&self, id: u8, object: bool) -> usize {
        let unsigned = object || self.lcdc.is_set(BG_WIN_TILE_DATA);
        if unsigned { id as usize * 16 } else { (0x1000 + (id as i8 as i32) * 16) as usize }
    }

    fn get_tile(&self, id: u8, object: bool) -> &[u8; 16] {
        let start = self.tile_offset(id, object);
        self.vram[0][start..start + 16].try_into().unwrap()
    }

    fn set_mode<const A: u16, const T: u8>(&mut self, mode: PpuMode, ifl: &mut IORegister<A, T>) {
        self.mode = mode;
        self.update_stat_line(ifl);
    }

    fn update_stat_line<const A: u16, const T: u8>(&mut self, ifl: &mut IORegister<A, T>) {
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

    fn tick<const A: u16, const T: u8>(&mut self, ifl: &mut IORegister<A, T>) {
        self.dot += 1;
        match self.mode {
            PpuMode::OamScan if self.dot == 80 => {
                self.scan_oam();
                self.set_mode(PpuMode::Drawing, ifl);
            }
            PpuMode::Drawing if self.dot == 252 => {
                self.render_scanline();
                self.set_mode(PpuMode::HBlank, ifl);
            }
            PpuMode::HBlank | PpuMode::VBlank if self.dot == 456 => {
                self.dot = 0;
                self.next_line(ifl);
            }
            _ => {}
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
                self.selected_objects[self.selected_objects_len] = [o[0], o[1], o[2], o[3]];
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
            self.pixels[ly][x] = PALETTE_ORIGINAL[shade as usize];
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
        let win_map: u16 = if self.lcdc.is_set(WINDOW_MAP) { 0x9C00 } else { 0x9800 };
        let bg_map: u16 = if self.lcdc.is_set(BG_MAP) { 0x9C00 } else { 0x9800 };
        let by = ly.wrapping_add(scy) as u16;
        let mut drew_window = false;

        for (x, pixel) in line_buffer.iter_mut().enumerate() {
            let x = x as i16;

            let color_idx = if window_visible && x >= win_x_start {
                let wx = (x - win_x_start) as u16;
                let wy = self.window_line;
                let tile = self.read_vram_raw(win_map + (wy / 8) * 32 + wx / 8);
                drew_window = true;
                self.get_tile_pixel(tile, wx % 8, wy % 8)
            } else {
                let bx = (x as u8).wrapping_add(scx) as u16;
                let tile = self.read_vram_raw(bg_map + (by / 8) * 32 + bx / 8);
                self.get_tile_pixel(tile, bx % 8, by % 8)
            };
            *pixel = color_idx;
        }

        if drew_window {
            self.window_line += 1;
        }
    }

    fn get_tile_pixel(&self, tile_index: u8, px: u16, py: u16) -> u8 {
        let tile_addr = if self.lcdc.is_set(BG_WIN_TILE_DATA) {
            0x8000 + tile_index as u16 * 16
        } else {
            0x9000u16.wrapping_add(((tile_index as i8) as i16 * 16) as u16)
        };
        let lo = self.read_vram_raw(tile_addr + py * 2);
        let hi = self.read_vram_raw(tile_addr + py * 2 + 1);
        let bit = 7 - px;
        (((hi >> bit) & 1) << 1) | ((lo >> bit) & 1)
    }

    fn render_objects(&mut self, line_buffer: &mut [u8; WIDTH]) {
        let screen_y = self.ly.read();
        if self.lcdc.is_clear(OBJ_ENABLE) {
            return;
        }
        let object_height = if self.lcdc.is_set(OBJ_SIZE) { 16 } else { 8 };
        let bg_enabled = self.lcdc.is_set(BG_WIN_ENABLE);

        for [tile_y, tile_x, tile_id, attr] in self.selected_objects[0..self.selected_objects_len].iter() {
            let bg_priority = attr & 0x80 != 0;
            let flip_y = attr & 0x40 != 0;
            let flip_x = attr & 0x20 != 0;
            let opb_1 = attr & 0x10 != 0;
            let mut row = screen_y + 16 - tile_y;
            if flip_y {
                row = object_height - 1 - row;
            }

            let id = if object_height == 16 { (tile_id & 0xFE) | (row >> 3) } else { *tile_id };
            let row_in_tile = (row % 8) as usize;
            let tile = self.get_tile(id, true);
            let palette = if opb_1 { self.opb1.read() } else { self.opb0.read() };

            let low_byte = tile[row_in_tile * 2];
            let high_byte = tile[row_in_tile * 2 + 1];

            let tile_x = *tile_x as usize;

            for lx in 0..8 {
                let Some(x) = (tile_x + lx).checked_sub(8).filter(|&x| x < WIDTH) else { continue };
                let bit_index = if flip_x { lx } else { 7 - lx };
                let color_id = (((high_byte >> bit_index) & 1) << 1) | ((low_byte >> bit_index) & 1);
                if color_id == 0 || line_buffer[x] & CLAIMED > 0 {
                    continue;
                }
                if bg_priority && bg_enabled && line_buffer[x] != 0 {
                    continue;
                }
                line_buffer[x] = CLAIMED | ((palette >> (color_id * 2)) & 0b11);
            }
        }
    }

    fn next_line<const A: u16, const T: u8>(&mut self, ifl: &mut IORegister<A, T>) {
        self.ly += 1;
        match self.ly.read() {
            144 => {
                self.set_mode(PpuMode::VBlank, ifl);
                ifl.set(0);
                self.redraw = true;
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

    pub fn ticks<const A: u16, const T: u8>(&mut self, dots: u8, ifl: &mut IORegister<A, T>) {
        if self.lcdc.is_clear(PPU_OFF) {
            self.dot = 4;
            self.ly.write_raw(0);
            self.mode = PpuMode::HBlank;
            return;
        }
        let next = match self.mode {
            PpuMode::OamScan => 80,
            PpuMode::Drawing => 252,
            PpuMode::HBlank | PpuMode::VBlank => 456,
        };
        if self.dot + (dots as u16) < next {
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
