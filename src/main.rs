mod buttons;
mod cpu;
mod graphics;
mod memory;
mod register;

use std::env;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use buttons::{Button::*, DpadDirection::*, GbKeyEvent};
use cpu::Cpu;
use graphics::{HEIGHT, WIDTH};
use memory::Mmu;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    // Init SDL
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Game Boy", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();

    // Init Gb
    let mut mem = Mmu::new();
    mem.load_rom(file_path);
    let mut cpu = Cpu::from(mem);

    // Game loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                // Quit
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                // Controls (Buttons)
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Button(A)),
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Button(B)),
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Button(Start)),
                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Button(Select)),
                Event::KeyUp {
                    keycode: Some(Keycode::X),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Button(A)),
                Event::KeyUp {
                    keycode: Some(Keycode::Z),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Button(B)),
                Event::KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Button(Start)),
                Event::KeyUp {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Button(Select)),
                // Dpad
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Dpad(Down)),
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Dpad(Up)),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Dpad(Left)),
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => cpu.membus.btns.press(GbKeyEvent::Dpad(Right)),
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Dpad(Down)),
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Dpad(Up)),
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Dpad(Left)),
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => cpu.membus.btns.release(GbKeyEvent::Dpad(Right)),
                _ => {}
            }
        }

        // Cycle device
        cpu.cycle();

        canvas.present();
        std::thread::sleep(Duration::from_nanos(238));
    }
}
