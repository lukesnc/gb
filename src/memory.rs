use std::fs;

pub fn load_rom(memory: &mut [u8; 0xFFFF], file_path: &String) {
    let data = fs::read(file_path).unwrap();
    for i in 0..0x7FFF {
        memory[i] = data[i];
    }
}

pub fn init_ram(memory: &mut [u8; 0xFFFF]) {
    memory[0xFF05] = 0;
    memory[0xFF06] = 0;
    memory[0xFF07] = 0;
    memory[0xFF10] = 0x80;
    memory[0xFF11] = 0xBF;
    memory[0xFF12] = 0xF3;
    memory[0xFF14] = 0xBF;
    memory[0xFF16] = 0x3F;
    memory[0xFF16] = 0x3F;
    memory[0xFF17] = 0;
    memory[0xFF19] = 0xBF;
    memory[0xFF1A] = 0x7F;
    memory[0xFF1B] = 0xFF;
    memory[0xFF1C] = 0x9F;
    memory[0xFF1E] = 0xFF;
    memory[0xFF20] = 0xFF;
    memory[0xFF21] = 0;
    memory[0xFF22] = 0;
    memory[0xFF23] = 0xBF;
    memory[0xFF24] = 0x77;
    memory[0xFF25] = 0xF3;
    memory[0xFF26] = 0xF1;
    memory[0xFF40] = 0x91;
    memory[0xFF42] = 0;
    memory[0xFF43] = 0;
    memory[0xFF45] = 0;
    memory[0xFF47] = 0xFC;
    memory[0xFF48] = 0xFF;
    memory[0xFF49] = 0xFF;
    memory[0xFF4A] = 0;
    memory[0xFF4B] = 0;
}
