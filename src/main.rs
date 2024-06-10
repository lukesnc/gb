mod buttons;
mod cpu;
mod graphics;
mod memory;
mod register;

use std::env;

use cpu::Cpu;
use memory::Mmu;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let mut mem = Mmu::new();
    mem.load_rom(file_path);
    let mut cpu = Cpu::from(mem);

    loop {
        cpu.cycle();
    }
}
