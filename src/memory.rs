use std::fs;

#[derive(Debug)]
pub struct Mem {
    ram: [u8; 65535],
}

impl Mem {
    pub fn new() -> Self {
        Mem { ram: [0; 65535] }
    }

    pub fn load_rom(&mut self, file_path: &String) {
        let data = fs::read(file_path).expect("failed to open rom file");

        for i in 0..0x7FFF {
            self.ram[i] = data[i];
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => 0x90, // Hardcode LCD
            _ => self.ram[addr as usize],
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            _ => self.ram[addr as usize] = val,
        };
    }
}
