use cpu::Cpu;
use memory::load_rom;
use std::env;

mod cpu;
mod memory;
mod register;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut memory = [0; 0xFFFF];
    load_rom(&mut memory, file_path);

    let mut cpu = Cpu::new();

    loop {
        cpu.cycle(&mut memory);
    }
}
