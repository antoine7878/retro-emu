use crate::gbc::config::CGMode;
use crate::gbc::memory::io_reg::{IOReg, IORegister};
use crate::gbc::memory::map::{
    BGP, DMA, DMA_END, DMA_START, LCDC, LY, LYC, OAM_END, OAM_SIZE, OAM_START, OBP0, OBP1, PAL_END, PAL_START, SCX,
    SCY, STAT, VBK, VRAM_END, VRAM_SIZE, VRAM_START, WX, WY,
};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

const LCDC_OFF: u8 = 7;

const PALETTE_ORIGNAL: [[u8; 3]; 4] = [[0x2A, 0x41, 0x39], [0x7B, 0x82, 0x0F], [0x5A, 0x79, 0x42], [0x39, 0x59, 0x4A]];

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

    selected_objects: [[u8; 4]; 10],
    selected_objects_len: usize,

    vram: [[u8; VRAM_SIZE]; 2],
    oam: [u8; OAM_SIZE],
    lcdc: IORegister<LCDC, 0xFF>,
    stat: IORegister<STAT, 0x78>,
    scy: IORegister<SCY, 0xFF>,
    scx: IORegister<SCX, 0xFF>,
    ly: IORegister<LY, 0x00, 0xFF>,
    lyc: IORegister<LYC, 0xFF>,
    dma: IORegister<DMA, 0xFF>,
    bgp: IORegister<BGP, 0xFF>,
    opb0: IORegister<OBP0, 0xFF>,
    opb1: IORegister<OBP1, 0xFF>,
    wy: IORegister<WY, 0xFF>,
    wx: IORegister<WX, 0xFF>,
    vbk: IORegister<VBK, 0x01>,
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            pixels: [[[0; 3]; WIDTH]; HEIGHT],
            dot: 0,
            vram: [[0; VRAM_SIZE]; 2],
            oam: [0; OAM_SIZE],
            selected_objects: [[0; 4]; 10],
            selected_objects_len: 0,
            stat_line: false,
            redraw: false,
            window_line: 0,
            lcdc: IORegister::default(),
            stat: IORegister::default(),
            scy: IORegister::default(),
            scx: IORegister::default(),
            ly: IORegister::default(),
            lyc: IORegister::default(),
            dma: IORegister::default(),
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
    pub fn reset(&mut self, mode: &CGMode) {
        self.lcdc.reset(mode);
        self.stat.reset(mode);
        self.scy.reset(mode);
        self.scx.reset(mode);
        self.ly.reset(mode);
        self.lyc.reset(mode);
        self.dma.reset(mode);
        self.bgp.reset(mode);
        self.opb0.reset(mode);
        self.opb1.reset(mode);
        self.wy.reset(mode);
        self.wx.reset(mode);
        self.vbk.reset(mode);

        self.pixels = [[[0; 3]; WIDTH]; HEIGHT];
        self.dot = 0;
        self.vram = [[0; VRAM_SIZE]; 2];
        self.oam = [0; OAM_SIZE];
        self.selected_objects = [[0; 4]; 10];
        self.selected_objects_len = 0;
        self.stat_line = false;
        self.redraw = false;
        self.window_line = 0;
        self.mode = PpuMode::HBlank;
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START..VRAM_END => self.read_vram(addr),
            OAM_START..OAM_END => self.read_oam(addr),
            // TODO:
            DMA_START..DMA_END => 0xFF,
            PAL_START..PAL_END => 0xFF,
            LCDC => self.lcdc.read(),
            STAT => {
                self.stat.read() & 0xF8
                    | ((self.ly.read_raw() == self.lyc.read_raw()) as u8) << 2
                    | self.mode.clone() as u8
            }
            SCY => self.scy.read(),
            SCX => self.scx.read(),
            LY => self.ly.read(),
            LYC => self.lyc.read(),
            DMA => self.dma.read(),
            BGP => self.bgp.read(),
            OBP0 => self.opb0.read(),
            OBP1 => self.opb1.read(),
            WY => self.wy.read(),
            WX => self.wx.read(),
            VBK => self.vbk.read(),
            _ => unreachable!("Address not in PPU"),
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        match self.mode {
            PpuMode::Drawing => 0xFF,
            _ => self.vram[self.vbk.read_raw() as usize & 0x01][(addr - VRAM_START) as usize],
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        match self.mode {
            PpuMode::OamScan | PpuMode::Drawing => 0xFF,
            _ => self.oam[(addr - OAM_START) as usize],
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            VRAM_START..VRAM_END => self.write_vram(addr, value),
            OAM_START..OAM_END => self.write_oam(addr, value),
            // TODO:
            DMA_START..DMA_END => (),
            PAL_START..PAL_END => (),
            LCDC => self.lcdc.write(value),
            STAT => self.stat.write(value),
            SCY => self.scy.write(value),
            SCX => self.scx.write(value),
            LY => self.ly.write(value),
            LYC => self.lyc.write(value),
            DMA => self.dma.write(value),
            BGP => self.bgp.write(value),
            OBP0 => self.opb0.write(value),
            OBP1 => self.opb1.write(value),
            WY => self.wy.write(value),
            WX => self.wx.write(value),
            VBK => self.vbk.write(value),
            _ => unreachable!("Address not in PPU"),
        }
    }

    pub fn write_vram(&mut self, addr: u16, value: u8) {
        match self.mode {
            PpuMode::Drawing => (),
            _ => self.vram[self.vbk.read_raw() as usize & 0x01][(addr - VRAM_START) as usize] = value,
        }
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        match self.mode {
            PpuMode::OamScan | PpuMode::Drawing => (),
            _ => self.oam[(addr - OAM_START) as usize] = value,
        }
    }

    fn tile_offset(&self, id: u8, object: bool) -> usize {
        let unsigned = object || self.lcdc.is_set(4);
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
        let height: i16 = if self.lcdc.is_set(2) { 16 } else { 8 };

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
    }

    fn render_scanline(&mut self) {
        let mut line_buffer = [0u8; WIDTH];
        self.render_background(&mut line_buffer);
        self.render_window(&mut line_buffer);
        self.render_objects(&mut line_buffer);
        for (x, id) in line_buffer.iter().enumerate() {
            self.pixels[self.ly.read() as usize][x] = PALETTE_ORIGNAL[*id as usize];
        }
    }

    fn render_background(&mut self, line_buffer: &mut [u8; WIDTH]) {
        let screen_y = self.ly.read();
        let pixel_row = &mut self.pixels[screen_y as usize];

        if self.lcdc.is_clear(0) {
            pixel_row.iter_mut().for_each(|pixel| *pixel = PALETTE_ORIGNAL[0]);
            return;
        }

        let tile_map_base: u16 = if self.lcdc.is_clear(3) { 0x9800 } else { 0x9C00 };
        let scx = self.scx.read();
        let scy = self.scy.read();
        let bg_palette = self.bgp.read();

        let map_y = scy.wrapping_add(screen_y);
        let tile_row = (map_y / 8) as u16;
        let row_in_tile = (map_y % 8) as usize;

        for (lx, buffer_pixel) in line_buffer.iter_mut().enumerate() {
            let map_x = scx.wrapping_add(lx as u8);
            let tile_col = (map_x / 8) as u16;

            let addr = tile_map_base + tile_row * 32 + tile_col;
            let tile_id = self.vram[0][(addr - VRAM_START) as usize];
            // opti: do it once every : x % 8 == 0
            let tile = self.get_tile(tile_id, false);
            let low_byte = tile[row_in_tile * 2];
            let high_byte = tile[row_in_tile * 2 + 1];

            let bit_index = 7 - (map_x % 8);
            let color_id = (((high_byte >> bit_index) & 1) << 1) | ((low_byte >> bit_index) & 1);
            let shade = (bg_palette >> (color_id * 2)) & 0b11;
            *buffer_pixel = shade;
        }
    }

    fn render_objects(&mut self, line_buffer: &mut [u8; WIDTH]) {
        let screen_y = self.ly.read();

        if self.lcdc.is_clear(1) {
            return;
        }
        // let object_size = if self.lcdc.is_set(2) { 8 } else { 16 };
        for [tile_y, tile_x, tile_id, attr] in self.selected_objects[0..self.selected_objects_len].iter() {
            let tile = self.get_tile(*tile_id, true);
            // let priority = attr & 0x80 > 0;
            // let flip_y = attr & 0x40 > 0;
            // let flip_x = attr & 0x20 > 0;
            let palette = if attr >> 4 & 1 == 0 { self.opb0.read() } else { self.opb1.read() };
            let row_in_tile = ((screen_y + 16 - tile_y) % 8) as usize;
            let tile_x = *tile_x as usize;
            for lx in 0..8 {
                let Some(x) = (tile_x + lx).checked_sub(8).filter(|&x| x < WIDTH) else { continue };
                let buffer_pixel = &mut line_buffer[x];
                let low_byte = tile[row_in_tile * 2];
                let high_byte = tile[row_in_tile * 2 + 1];
                let bit_index = 7 - lx;
                let color_id = (((high_byte >> bit_index) & 1) << 1) | ((low_byte >> bit_index) & 1);
                let shade = (palette >> (color_id * 2)) & 0b11;
                *buffer_pixel = shade;
            }
        }
    }

    fn render_window(&mut self, _line_buffer: &mut [u8; WIDTH]) {}

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
                self.set_mode(PpuMode::OamScan, ifl);
            }
            0..144 => self.set_mode(PpuMode::OamScan, ifl),
            _ => (),
        }
    }

    pub fn ticks<const A: u16, const T: u8>(&mut self, dots: u8, ifl: &mut IORegister<A, T>) {
        if self.lcdc.is_clear(LCDC_OFF) {
            self.dot = 0;
            self.ly.write_raw(0);
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
