pub const WIDTH: u32 = 166;
pub const HEIGHT: u32 = 144;

type Frame = [[u8; 166]; 144];
type Tile = [u8; 16];
type TileColorMap = [[u8; 8]; 8];

fn tile_rows(tile: Tile) -> [[u8; 2]; 8] {
    let mut rows: [[u8; 2]; 8] = [[0; 2]; 8];
    for (i, chunk) in tile.chunks(2).enumerate() {
        rows[i].copy_from_slice(chunk);
    }
    rows
}

fn tile_color_map(tile: Tile) -> TileColorMap {
    let mut map: [[u8; 8]; 8] = [[0; 8]; 8];
    let rows = tile_rows(tile);
    for (line, row) in rows.iter().enumerate() {
        let p1 = row[0]; // a..h
        let p2 = row[1]; // i..p

        for px in 0..8 {
            let b1 = (p1 & (1 << 7 - px)) >> (7 - px);
            let b2 = (p2 & (1 << 7 - px)) >> (7 - px);
            let color = b2 << 1 | b1;
            map[line][px] = color;
        }
    }
    map
}

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

    /// LCDC.7
    fn lcd_enabled(&self) -> bool {
        self.lcdc & (1 << 7) > 0
    }

    /// LCDC.6
    fn win_tile_map_area(&self) -> u16 {
        if self.lcdc & (1 << 6) == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    /// LCDC.5
    fn win_enabled(&self) -> bool {
        self.lcdc & (1 << 5) > 0
    }

    /// LCDC.4
    /// True: 0x8000 based normal addressing.
    /// False: 0x9000 based signed addressing.
    fn bg_win_addr_mode(&self) -> bool {
        self.lcdc & (1 << 4) > 0
    }

    /// LCDC.3
    fn bg_tile_map_area(&self) -> u16 {
        if self.lcdc & (1 << 3) == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    /// LCDC.2
    fn obj_size(&self) -> u32 {
        if self.lcdc & (1 << 2) == 0 {
            8 * 8
        } else {
            8 * 16
        }
    }

    /// LCDC.1
    fn obj_enabled(&self) -> bool {
        self.lcdc & (1 << 1) > 0
    }

    /// LCDC.0
    fn bg_win_enabled(&self) -> bool {
        self.lcdc & 1 > 0
    }

    fn ppu_mode(&self) -> u8 {
        self.stat & 0b11
    }

    fn set_ppu_mode(&mut self, n: u8) {
        self.stat &= n & 0b11;
    }

    pub fn should_vblank_interrupt(&self) -> bool {
        self.ppu_mode() == 1
    }

    pub fn should_stat_interrupt(&self) -> bool {
        (self.stat & (1 << 6) > 0 && self.stat & (1 << 2) > 0)    // LYC int
            || (self.stat & (1 << 5) > 0 && self.ppu_mode() == 2) // Mode 2 int
            || (self.stat & (1 << 4) > 0 && self.ppu_mode() == 1) // Mode 1 int
            || (self.stat & (1 << 3) > 0 && self.ppu_mode() == 0) // Mode 0 int
    }

    fn object_tile(&self, id: u8) -> Tile {
        let start_addr = 0x8000 + (16 * id as u16);
        let mut tile = [0; 16];
        for i in 0..tile.len() {
            tile[i] = self.read_vram(start_addr + i as u16);
        }
        tile
    }

    fn bg_win_tile(&self, id: u8) -> Tile {
        let mut tile = [0; 16];
        let start_addr = if self.bg_win_addr_mode() {
            0x8000 + (16 * id as u16)
        } else {
            let id = if id > 127 { 127 - id as i8 } else { id as i8 };
            let offset = 16 * id as i32;
            (0x9000 + offset) as u16
        };

        for i in 0..tile.len() {
            tile[i] = self.read_vram(start_addr as u16 + i as u16);
        }
        tile
    }

    pub fn draw_frame(&mut self) -> Frame {
        let mut frame: Frame = [[0; 166]; 144];

        self.ly = 0;
        while self.ly <= 153 {
            self.set_ppu_mode(2);

            self.ly += 1;
        }
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_t() {
        let t: Tile = [
            0x3C, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x5E, 0x7E, 0x0A, 0x7C, 0x56,
            0x38, 0x7C,
        ];

        assert_eq!(
            tile_rows(t),
            [
                [60, 126],
                [66, 66],
                [66, 66],
                [66, 66],
                [126, 94],
                [126, 10],
                [124, 86],
                [56, 124],
            ]
        );
        assert_eq!(
            tile_color_map(t),
            [
                [0, 2, 3, 3, 3, 3, 2, 0],
                [0, 3, 0, 0, 0, 0, 3, 0],
                [0, 3, 0, 0, 0, 0, 3, 0],
                [0, 3, 0, 0, 0, 0, 3, 0],
                [0, 3, 1, 3, 3, 3, 3, 0],
                [0, 1, 1, 1, 3, 1, 3, 0],
                [0, 3, 1, 3, 1, 3, 2, 0],
                [0, 2, 3, 3, 3, 2, 0, 0],
            ]
        );
    }
}
