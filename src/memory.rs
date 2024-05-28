use std::fs;

pub fn load_rom(memory: &mut [u8; 0xFFFF], file_path: &String) {
    let data = fs::read(file_path).expect("failed to open rom");

    for i in 0..0x7FFF {
        memory[i] = data[i];
    }
}
