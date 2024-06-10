use std::env;

use cpu::Cpu;
use memory::Mem;

mod buttons;
mod cpu;
mod graphics;
mod memory;
mod register;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    // gameboy doctor
    let mut mem = Mem::new();
    mem.load_rom(file_path);
    let mut cpu = Cpu::from(mem);

    loop {
        cpu.cycle();
    }
}
