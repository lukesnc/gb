use std::fs;

#[derive(Debug)]
pub struct Mem {
    ram: [u8; 65535],
    ie: u8,    // interrupt enable, seperate from the CPUs ime reg
    iflag: u8, // interrupt flag
}

impl Mem {
    pub fn new() -> Self {
        Mem {
            ram: [0; 65535],
            ie: 0,
            iflag: 0,
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
            0b00000001 => 0x40, // VBlank
            0b00000010 => 0x48, // Lcd
            0b00000100 => 0x50, // Timer
            0b00001000 => 0x58, // Serial
            0b00010000 => 0x60, // Joypad
            _ => return None,
        };
        self.iflag &= !requested;
        Some(addr)
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF0F => self.iflag,
            0xFF44 => 0x90, // Hardcode LCD
            0xFFFF => self.ie,
            _ => self.ram[addr as usize],
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF0F => self.iflag = val,
            0xFFFF => self.ie = val,
            _ => self.ram[addr as usize] = val,
        };
    }
}
