use std::fs;

use crate::buttons::Btns;
use crate::graphics::Gpu;

struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
    running_div: u32,
    running_counter: u32,
}

impl Timer {
    fn enabled(&self) -> bool {
        self.tac & 0b100 > 0
    }

    fn step_size(&self) -> u32 {
        match self.tac & 0b11 {
            0b00 => 256,
            0b01 => 4,
            0b10 => 16,
            _ => 64,
        }
    }
}

pub struct Mmu {
    ram: [u8; 65535],
    ie: u8,    // interrupt enable, seperate from the CPUs ime reg
    iflag: u8, // interrupt flag
    timer: Timer,
    pub btns: Btns,
    pub gpu: Gpu,
}

impl Mmu {
    pub fn new() -> Self {
        Mmu {
            ram: [0; 65535],
            ie: 0xE1,
            iflag: 0,
            timer: Timer {
                div: 0xAB,
                tima: 0,
                tma: 0,
                tac: 0xF8,
                running_div: 0,
                running_counter: 0,
            },
            btns: Btns::new(),
            gpu: Gpu::new(),
        }
    }

    pub fn load_rom(&mut self, file_path: &String) {
        let data = fs::read(file_path).expect("failed to open rom file");

        for i in 0..0x7FFF {
            self.ram[i] = data[i];
        }
    }

    /// Get the address of the interrupt to be serviced (if there is one)
    pub fn interrupt_addr(&mut self) -> Option<u8> {
        let requested = self.iflag & self.ie;
        let addr = match requested {
            0b00001 => 0x40, // VBlank
            0b00010 => 0x48, // Lcd
            0b00100 => 0x50, // Timer
            0b01000 => 0x58, // Serial
            0b10000 => 0x60, // Joypad
            _ => return None,
        };
        self.iflag &= !requested;
        Some(addr)
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.gpu.read_vram(addr),
            0xFF00 => self.btns.data(),
            0xFF04 => self.timer.div,
            0xFF05 => self.timer.tima,
            0xFF06 => self.timer.tma,
            0xFF07 => self.timer.tac,
            0xFF0F => self.iflag,
            0xFF40 => self.gpu.lcdc,
            0xFF41 => self.gpu.stat,
            0xFF42 => self.gpu.scy,
            0xFF43 => self.gpu.scx,
            0xFF44 => self.gpu.ly,
            0xFF45 => self.gpu.lyc,
            0xFF46 => 0, // DMA transfer on write only
            0xFF47 => self.gpu.bgp,
            0xFF48 => self.gpu.obp0,
            0xFF49 => self.gpu.obp1,
            0xFF4A => self.gpu.wy,
            0xFF4B => self.gpu.wx,
            0xFFFF => self.ie,
            _ => self.ram[addr as usize],
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000..=0x9FFF => self.gpu.write_vram(addr, val),
            0xFF00 => self.btns.pick_row(val),
            0xFF04 => self.timer.div = 0,
            0xFF05 => self.timer.tima = val,
            0xFF06 => self.timer.tma = val,
            0xFF07 => self.timer.tac = val,
            0xFF0F => self.iflag = val,
            0xFF40 => self.gpu.lcdc = val,
            0xFF41 => self.gpu.stat = val,
            0xFF42 => self.gpu.scy = val,
            0xFF43 => self.gpu.scx = val,
            0xFF44 => {} // LY read only,
            0xFF45 => self.gpu.lyc = val,
            0xFF46 => self.dma_transfer(val),
            0xFF47 => self.gpu.bgp = val,
            0xFF48 => self.gpu.obp0 = val,
            0xFF49 => self.gpu.obp1 = val,
            0xFF4A => self.gpu.wy = val,
            0xFF4B => self.gpu.wx = val,
            0xFFFF => self.ie = val,
            _ => self.ram[addr as usize] = val,
        };
    }

    pub fn do_cycles(&mut self, m_cycles: u32) {
        // Timer routine
        self.timer.running_div += m_cycles;
        while self.timer.running_div >= 64 {
            self.timer.div = self.timer.div.wrapping_add(1);
            self.timer.running_div -= 64;
        }

        if self.timer.enabled() {
            self.timer.running_counter += m_cycles;
            let step_size = self.timer.step_size();
            while self.timer.running_counter >= step_size {
                self.timer.tima = self.timer.tima.wrapping_add(1);
                if self.timer.tima == 0 {
                    // Timer overflowed
                    self.timer.tima = self.timer.tma;
                    self.iflag |= 0b100;
                }
                self.timer.running_counter -= step_size;
            }
        }

        // Buttons routine (check lower nibble any button is pressed)
        if self.btns.data() & 0xF < 0xF {
            self.iflag |= 1 << 4;
        }

        // GPU routine
        if self.gpu.should_vblank_interrupt() {
            self.iflag |= 1;
        }
        if self.gpu.should_stat_interrupt() {
            self.iflag |= 0b10;
        }
    }

    fn dma_transfer(&mut self, val: u8) {
        let source = (val as usize) << 8;
        for offset in 0..0x9F {
            self.ram[0xFE00 + offset] = self.ram[source + offset];
        }
        self.do_cycles(160);
    }
}
