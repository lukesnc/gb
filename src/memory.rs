use std::fs;

pub enum IFlagMask {
    VBlank = 0b00000001,
    Lcd = 0b00000010,
    Timer = 0b00000100,
    Serial = 0b00001000,
    Joypad = 0b00010000,
}

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

    pub fn iflag(&self, mask: IFlagMask) -> bool {
        self.iflag & (mask as u8) > 0
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
