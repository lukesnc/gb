use std::env;

mod cpu;
mod memory;
mod register;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
}
