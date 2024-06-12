pub const WIDTH: u32 = 166;
pub const HEIGHT: u32 = 144;

type Tile = [u8; 16];

pub struct Gpu {
    vram: [u8; 8192],
    pub lcdc: u8, // LCD control
    pub ly: u8,   // LCD Y coord
    pub lyc: u8,  // LY compare
    pub stat: u8, // LCD status
    pub scy: u8,  // Scroll Y
    pub scx: u8,  // Scroll X
    pub wy: u8,   // Window pos Y
    pub wx: u8,   // Window pox X
    pub bgp: u8,  // BG palette data
    pub obp0: u8, // Obj palette 0
    pub obp1: u8, // Obj palette 1
}

impl Gpu {
    pub fn new() -> Self {
        Gpu {
            vram: [0; 8192],
            lcdc: 0x91,
            ly: 0,
            lyc: 0,
            stat: 0x85,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 0,
            bgp: 0xFC,
            obp0: 0,
            obp1: 0,
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[addr as usize - 0x8000]
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        self.vram[addr as usize - 0x8000] = val;
    }

    // LCDC.7
    fn lcd_enable(&self) -> bool {
        self.lcdc & (1 << 7) > 0
    }

    // LCDC.6
    fn window_tile_map_area(&self) -> u16 {
        if self.lcdc & (1 << 6) == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    // LCDC.5
    fn window_enable(&self) -> bool {
        self.lcdc & (1 << 5) > 0
    }

    // LCDC.4
    fn bg_window_tile_data_area(&self) -> u16 {
        if self.lcdc & (1 << 4) == 0 {
            0x8800
        } else {
            0x8000
        }
    }

    // LCDC.3
    fn bg_tile_map_area(&self) -> u16 {
        if self.lcdc & (1 << 3) == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    // LCDC.2
    fn obj_size(&self) -> u32 {
        if self.lcdc & (1 << 2) == 0 {
            8 * 8
        } else {
            8 * 16
        }
    }

    // LCDC.1
    fn obj_enable(&self) -> bool {
        self.lcdc & (1 << 1) > 0
    }

    // LCDC.0
    fn bg_window_enable(&self) -> bool {
        self.lcdc & 1 > 0
    }

    //fn tile(&mut self, obj_num: usize) -> Tile {
    //    let base_addr = obj_num * 16;
    //    let tile_data = &self.vram[base_addr..base_addr + 16];
    //}
}
