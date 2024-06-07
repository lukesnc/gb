use std::fs;

#[derive(Debug)]
struct Timer {
    div: u8,
    counter: u8,
    modulo: u8,
    tac: u8,
}

impl Timer {
    fn enabled(&self) -> bool {
        self.tac & 0b100 == 0b100
    }

    fn step_size(&self) -> u32 {
        match self.tac & 0b11 {
            0b00 => 256,
            0b01 => 4,
            0b10 => 16,
            0b11 => 64,
            _ => panic!("you suck"),
        }
    }
}

#[derive(Debug)]
pub struct Mem {
    ram: [u8; 65535],
    ie: u8,    // interrupt enable, seperate from the CPUs ime reg
    iflag: u8, // interrupt flag
    timer: Timer,
}

impl Mem {
    pub fn new() -> Self {
        Mem {
            ram: [0; 65535],
            ie: 0xE1,
            iflag: 0,
            timer: Timer {
                div: 0xAB,
                counter: 0,
                modulo: 0,
                tac: 0xF8,
            },
        }
    }

    pub fn load_rom(&mut self, file_path: &String) {
        let data = fs::read(file_path).expect("failed to open rom file");

        for i in 0..0x7FFF {
            self.ram[i] = data[i];
        }
    }

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
            0xFF04 => self.timer.div,
            0xFF05 => self.timer.counter,
            0xFF06 => self.timer.modulo,
            0xFF07 => self.timer.tac,
            0xFF0F => self.iflag,
            0xFF44 => 0x90, // Hardcode LCD
            0xFFFF => self.ie,
            _ => self.ram[addr as usize],
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.timer.div = 0,
            0xFF05 => self.timer.counter = val,
            0xFF06 => self.timer.modulo = val,
            0xFF07 => self.timer.tac = val,
            0xFF0F => self.iflag = val,
            0xFFFF => self.ie = val,
            _ => self.ram[addr as usize] = val,
        };
    }

    pub fn do_cycles(&mut self, m_cycles: u8) {
        // Timer routine
        let mut div = self.timer.div as u32;
        div += m_cycles as u32 * 4;

        if self.timer.enabled() {
            let steps = div % self.timer.step_size();
            if self.timer.counter as u32 + steps > 0xFF {
                self.timer.counter = self.timer.modulo;
                self.iflag |= 0b100;
            } else {
                self.timer.counter = self.timer.counter.wrapping_add(steps as u8);
            }
        }

        self.timer.div = div as u8;
    }
}
