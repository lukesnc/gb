use crate::register::Flag::{C, H, N, Z};
use crate::register::Registers;
use std::{thread, time};

const M_CYCLE: time::Duration = time::Duration::from_nanos(238 * 4);

pub struct CPU {
    reg: Registers,
    ime: bool,
    ime_scheduled: bool,
    halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            reg: Registers::new(),
            ime: false,
            ime_scheduled: false,
            halted: false,
        }
    }

    // CPU cycle
    pub fn cycle(&mut self, memory: &mut [u8; 0xFFFF]) {
        if self.halted {
            return;
        }
        let cost = self.exec(memory);
        thread::sleep(M_CYCLE * cost);
    }

    fn exec(&mut self, memory: &mut [u8; 0xFFFF]) -> u32 {
        let opcode = self.read_byte(memory);

        println!("op: {:02X}", opcode);
        // println!("pc: {:02X}", self.reg.pc);

        match opcode {
            0x00 => 1,
            0x01 => {
                let nn = self.read_word(memory);
                self.reg.set_bc(nn);
                3
            }
            0x02 => {
                memory[self.reg.bc() as usize] = self.reg.a;
                2
            }
            0x03 => {
                self.reg.set_bc(self.reg.bc().wrapping_add(1));
                2
            }
            0x04 => {
                self.reg.b = self.alu_inc(self.reg.b);
                1
            }
            0x05 => {
                self.reg.b = self.alu_dec(self.reg.b);
                1
            }
            0x06 => {
                self.reg.b = self.read_byte(memory);
                2
            }
            // 0x07 => {}
            0x08 => {
                let nn = self.read_word(memory);
                memory[nn as usize] = self.reg.sp as u8;
                memory[(nn + 1) as usize] = (self.reg.sp >> 8) as u8;
                5
            }
            0x09 => {
                self.alu_add16(self.reg.bc());
                2
            }
            0x0A => {
                self.reg.a = memory[self.reg.bc() as usize];
                2
            }
            0x0B => {
                self.reg.set_bc(self.reg.bc().wrapping_sub(1));
                2
            }
            0x0C => {
                self.reg.c = self.alu_inc(self.reg.c);
                1
            }
            0x0D => {
                self.reg.c = self.alu_dec(self.reg.c);
                1
            }
            0x0E => {
                self.reg.c = self.read_byte(memory);
                2
            }
            // 0x0F => {}
            // 0x10 => {}
            0x11 => {
                let nn = self.read_word(memory);
                self.reg.set_de(nn);
                3
            }
            0x12 => {
                memory[self.reg.de() as usize] = self.reg.a;
                2
            }
            0x13 => {
                self.reg.set_de(self.reg.de().wrapping_add(1));
                2
            }
            0x14 => {
                self.reg.d = self.alu_inc(self.reg.d);
                1
            }
            0x15 => {
                self.reg.d = self.alu_dec(self.reg.d);
                1
            }
            0x16 => {
                self.reg.d = self.read_byte(memory);
                2
            }
            // 0x17 => {}
            0x18 => {
                let e = self.read_byte(memory) as i8;
                self.reg.pc += e as i16 as u16;
                3
            }
            0x19 => {
                self.alu_add16(self.reg.de());
                2
            }
            0x1A => {
                self.reg.a = memory[self.reg.de() as usize];
                2
            }
            0x1B => {
                self.reg.set_de(self.reg.de().wrapping_sub(1));
                2
            }
            0x1C => {
                self.reg.e = self.alu_inc(self.reg.e);
                1
            }
            0x1D => {
                self.reg.e = self.alu_dec(self.reg.e);
                1
            }
            0x1E => {
                self.reg.e = self.read_byte(memory);
                2
            }
            // 0x1F => {}
            0x20 => {
                let e = self.read_byte(memory);
                if !self.reg.flag(Z) {
                    self.reg.pc = self.reg.pc.wrapping_add(e as i16 as u16);
                    3
                } else {
                    2
                }
            }
            0x21 => {
                let nn = self.read_word(memory);
                self.reg.set_hl(nn);
                3
            }
            0x22 => {
                memory[self.reg.hl() as usize] = self.reg.a;
                memory[self.reg.hl() as usize] = self.alu_inc(memory[self.reg.hl() as usize]);
                2
            }
            0x23 => {
                self.reg.set_hl(self.reg.hl().wrapping_add(1));
                2
            }
            0x24 => {
                self.reg.h = self.alu_inc(self.reg.h);
                1
            }
            0x25 => {
                self.reg.h = self.alu_dec(self.reg.h);
                1
            }
            0x26 => {
                self.reg.h = self.read_byte(memory);
                2
            }
            0x27 => {
                // DAA instruction credit: https://forums.nesdev.org/viewtopic.php?t=15944
                if !self.reg.flag(N) {
                    if self.reg.flag(C) || self.reg.a > 0x99 {
                        self.reg.a += 0x60;
                        self.reg.set_flag(C, true);
                    }
                    if self.reg.flag(H) || (self.reg.a & 0x0f) > 0x09 {
                        self.reg.a += 0x6;
                    }
                } else {
                    if self.reg.flag(C) {
                        self.reg.a -= 0x60;
                    }
                    if self.reg.flag(H) {
                        self.reg.a -= 0x6;
                    }
                }
                self.reg.set_flag(Z, self.reg.a == 0);
                self.reg.set_flag(H, false);
                1
            }
            0x28 => {
                let e = self.read_byte(memory);
                if self.reg.flag(Z) {
                    self.reg.pc = self.reg.pc.wrapping_add(e as i16 as u16);
                    3
                } else {
                    2
                }
            }
            0x29 => {
                self.alu_add16(self.reg.hl());
                2
            }
            0x2A => {
                self.reg.a = memory[self.reg.hl() as usize];
                memory[self.reg.hl() as usize] = self.alu_inc(memory[self.reg.hl() as usize]);
                2
            }
            0x2B => {
                self.reg.set_hl(self.reg.hl().wrapping_sub(1));
                2
            }
            0x2C => {
                self.reg.l = self.alu_inc(self.reg.l);
                1
            }
            0x2D => {
                self.reg.l = self.alu_dec(self.reg.l);
                1
            }
            0x2E => {
                self.reg.l = self.read_byte(memory);
                2
            }
            0x2F => {
                self.reg.a = !self.reg.a;
                self.reg.set_flag(N, true);
                self.reg.set_flag(H, true);
                1
            }
            0x30 => {
                let e = self.read_byte(memory);
                if !self.reg.flag(C) {
                    self.reg.pc = self.reg.pc.wrapping_add(e as i16 as u16);
                    3
                } else {
                    2
                }
            }
            0x31 => {
                let nn = self.read_word(memory);
                self.reg.sp = nn;
                3
            }
            0x32 => {
                memory[self.reg.hl() as usize] = self.reg.a;
                memory[self.reg.hl() as usize] = self.alu_dec(memory[self.reg.hl() as usize]);
                2
            }
            0x33 => {
                self.reg.sp = self.reg.sp.wrapping_add(1);
                2
            }
            0x34 => {
                memory[self.reg.hl() as usize] = self.alu_inc(memory[self.reg.hl() as usize]);
                3
            }
            0x35 => {
                memory[self.reg.hl() as usize] = self.alu_dec(memory[self.reg.hl() as usize]);
                3
            }
            0x36 => {
                memory[self.reg.hl() as usize] = self.read_byte(memory);
                3
            }
            0x37 => {
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
                self.reg.set_flag(C, true);
                1
            }
            0x38 => {
                let e = self.read_byte(memory);
                if self.reg.flag(C) {
                    self.reg.pc = self.reg.pc.wrapping_add(e as i16 as u16);
                    3
                } else {
                    2
                }
            }
            0x39 => {
                self.alu_add16(self.reg.sp);
                2
            }
            0x3A => {
                self.reg.a = memory[self.reg.hl() as usize];
                memory[self.reg.hl() as usize] = self.alu_dec(memory[self.reg.hl() as usize]);
                2
            }
            0x3B => {
                self.reg.sp = self.reg.sp.wrapping_sub(1);
                2
            }
            0x3C => {
                self.reg.a = self.alu_inc(self.reg.a);
                1
            }
            0x3D => {
                self.reg.a = self.alu_dec(self.reg.a);
                1
            }
            0x3E => {
                self.reg.a = self.read_byte(memory);
                2
            }
            0x3F => {
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
                self.reg.set_flag(C, !self.reg.flag(C));
                1
            }
            0x40 => 1,
            0x41 => {
                self.reg.b = self.reg.c;
                1
            }
            0x42 => {
                self.reg.b = self.reg.d;
                1
            }
            0x43 => {
                self.reg.b = self.reg.e;
                1
            }
            0x44 => {
                self.reg.b = self.reg.h;
                1
            }
            0x45 => {
                self.reg.b = self.reg.l;
                1
            }
            0x46 => {
                self.reg.b = memory[self.reg.hl() as usize];
                2
            }
            0x47 => {
                self.reg.b = self.reg.a;
                1
            }
            0x48 => {
                self.reg.c = self.reg.b;
                1
            }
            0x49 => 1,
            0x4A => {
                self.reg.c = self.reg.d;
                1
            }
            0x4B => {
                self.reg.c = self.reg.e;
                1
            }
            0x4C => {
                self.reg.c = self.reg.h;
                1
            }
            0x4D => {
                self.reg.c = self.reg.l;
                1
            }
            0x4E => {
                self.reg.c = memory[self.reg.hl() as usize];
                2
            }
            0x4F => {
                self.reg.c = self.reg.a;
                1
            }
            0x50 => {
                self.reg.d = self.reg.b;
                1
            }
            0x51 => {
                self.reg.d = self.reg.c;
                1
            }
            0x52 => 1,
            0x53 => {
                self.reg.d = self.reg.e;
                1
            }
            0x54 => {
                self.reg.d = self.reg.h;
                1
            }
            0x55 => {
                self.reg.d = self.reg.l;
                1
            }
            0x56 => {
                self.reg.d = memory[self.reg.hl() as usize];
                2
            }
            0x57 => {
                self.reg.d = self.reg.a;
                1
            }
            0x58 => {
                self.reg.e = self.reg.b;
                1
            }
            0x59 => {
                self.reg.e = self.reg.c;
                1
            }
            0x5A => {
                self.reg.e = self.reg.d;
                1
            }
            0x5B => 1,
            0x5C => {
                self.reg.e = self.reg.h;
                1
            }
            0x5D => {
                self.reg.e = self.reg.l;
                1
            }
            0x5E => {
                self.reg.e = memory[self.reg.hl() as usize];
                2
            }
            0x5F => {
                self.reg.e = self.reg.a;
                1
            }
            0x60 => {
                self.reg.h = self.reg.b;
                1
            }
            0x61 => {
                self.reg.h = self.reg.c;
                1
            }
            0x62 => {
                self.reg.h = self.reg.d;
                1
            }
            0x63 => {
                self.reg.h = self.reg.e;
                1
            }
            0x64 => 1,
            0x65 => {
                self.reg.h = self.reg.l;
                1
            }
            0x66 => {
                self.reg.h = memory[self.reg.hl() as usize];
                2
            }
            0x67 => {
                self.reg.h = self.reg.a;
                1
            }
            0x68 => {
                self.reg.l = self.reg.b;
                1
            }
            0x69 => {
                self.reg.l = self.reg.c;
                1
            }
            0x6A => {
                self.reg.l = self.reg.d;
                1
            }
            0x6B => {
                self.reg.l = self.reg.e;
                1
            }
            0x6C => {
                self.reg.l = self.reg.h;
                1
            }
            0x6D => 1,
            0x6E => {
                self.reg.l = memory[self.reg.hl() as usize];
                2
            }
            0x6F => {
                self.reg.l = self.reg.a;
                1
            }
            0x70 => {
                memory[self.reg.hl() as usize] = self.reg.b;
                2
            }
            0x71 => {
                memory[self.reg.hl() as usize] = self.reg.c;
                2
            }
            0x72 => {
                memory[self.reg.hl() as usize] = self.reg.d;
                2
            }
            0x73 => {
                memory[self.reg.hl() as usize] = self.reg.e;
                2
            }
            0x74 => {
                memory[self.reg.hl() as usize] = self.reg.h;
                2
            }
            0x75 => {
                memory[self.reg.hl() as usize] = self.reg.l;
                2
            }
            // 0x76 => {}
            0x77 => {
                memory[self.reg.hl() as usize] = self.reg.a;
                2
            }
            0x78 => {
                self.reg.a = self.reg.b;
                1
            }
            0x79 => {
                self.reg.a = self.reg.c;
                1
            }
            0x7A => {
                self.reg.a = self.reg.d;
                1
            }
            0x7B => {
                self.reg.a = self.reg.e;
                1
            }
            0x7C => {
                self.reg.a = self.reg.h;
                1
            }
            0x7D => {
                self.reg.a = self.reg.l;
                1
            }
            0x7E => {
                self.reg.a = memory[self.reg.hl() as usize];
                2
            }
            0x7F => 1,
            0x80 => {
                self.alu_add(self.reg.b);
                1
            }
            0x81 => {
                self.alu_add(self.reg.c);
                1
            }
            0x82 => {
                self.alu_add(self.reg.d);
                1
            }
            0x83 => {
                self.alu_add(self.reg.e);
                1
            }
            0x84 => {
                self.alu_add(self.reg.h);
                1
            }
            0x85 => {
                self.alu_add(self.reg.l);
                1
            }
            0x86 => {
                self.alu_add(memory[self.reg.hl() as usize]);
                2
            }
            0x87 => {
                self.alu_add(self.reg.a);
                1
            }
            0x88 => {
                self.alu_adc(self.reg.b);
                1
            }
            0x89 => {
                self.alu_adc(self.reg.c);
                1
            }
            0x8A => {
                self.alu_adc(self.reg.d);
                1
            }
            0x8B => {
                self.alu_adc(self.reg.e);
                1
            }
            0x8C => {
                self.alu_adc(self.reg.h);
                1
            }
            0x8D => {
                self.alu_adc(self.reg.l);
                1
            }
            0x8E => {
                self.alu_adc(memory[self.reg.hl() as usize]);
                2
            }
            0x8F => {
                self.alu_adc(self.reg.a);
                1
            }
            0x90 => {
                self.alu_sub(self.reg.b);
                1
            }
            0x91 => {
                self.alu_sub(self.reg.c);
                1
            }
            0x92 => {
                self.alu_sub(self.reg.d);
                1
            }
            0x93 => {
                self.alu_sub(self.reg.e);
                1
            }
            0x94 => {
                self.alu_sub(self.reg.h);
                1
            }
            0x95 => {
                self.alu_sub(self.reg.l);
                1
            }
            0x96 => {
                self.alu_sub(memory[self.reg.hl() as usize]);
                2
            }
            0x97 => {
                self.alu_sub(self.reg.a);
                1
            }
            0x98 => {
                self.alu_sbc(self.reg.b);
                1
            }
            0x99 => {
                self.alu_sbc(self.reg.c);
                1
            }
            0x9A => {
                self.alu_sbc(self.reg.d);
                1
            }
            0x9B => {
                self.alu_sbc(self.reg.e);
                1
            }
            0x9C => {
                self.alu_sbc(self.reg.h);
                1
            }
            0x9D => {
                self.alu_sbc(self.reg.l);
                1
            }
            0x9E => {
                self.alu_sbc(memory[self.reg.hl() as usize]);
                2
            }
            0x9F => {
                self.alu_sbc(self.reg.a);
                1
            }
            0xA0 => {
                self.alu_and(self.reg.b);
                1
            }
            0xA1 => {
                self.alu_and(self.reg.c);
                1
            }
            0xA2 => {
                self.alu_and(self.reg.d);
                1
            }
            0xA3 => {
                self.alu_and(self.reg.e);
                1
            }
            0xA4 => {
                self.alu_and(self.reg.h);
                1
            }
            0xA5 => {
                self.alu_and(self.reg.l);
                1
            }
            0xA6 => {
                self.alu_and(memory[self.reg.hl() as usize]);
                2
            }
            0xA7 => {
                self.alu_and(self.reg.a);
                1
            }
            0xA9 => {
                self.alu_xor(self.reg.c);
                1
            }
            0xAA => {
                self.alu_xor(self.reg.d);
                1
            }
            0xAB => {
                self.alu_xor(self.reg.e);
                1
            }
            0xAC => {
                self.alu_xor(self.reg.h);
                1
            }
            0xAD => {
                self.alu_xor(self.reg.l);
                1
            }
            0xAE => {
                self.alu_xor(memory[self.reg.hl() as usize]);
                2
            }
            0xAF => {
                self.alu_xor(self.reg.a);
                1
            }
            0xB0 => {
                self.alu_or(self.reg.b);
                1
            }
            0xB1 => {
                self.alu_or(self.reg.c);
                1
            }
            0xB2 => {
                self.alu_or(self.reg.d);
                1
            }
            0xB3 => {
                self.alu_or(self.reg.e);
                1
            }
            0xB4 => {
                self.alu_or(self.reg.h);
                1
            }
            0xB5 => {
                self.alu_or(self.reg.l);
                1
            }
            0xB6 => {
                self.alu_or(memory[self.reg.hl() as usize]);
                2
            }
            0xB7 => {
                self.alu_or(self.reg.a);
                1
            }
            0xB8 => {
                self.alu_cp(self.reg.b);
                1
            }
            0xB9 => {
                self.alu_cp(self.reg.c);
                1
            }
            0xBA => {
                self.alu_cp(self.reg.d);
                1
            }
            0xBB => {
                self.alu_cp(self.reg.e);
                1
            }
            0xBC => {
                self.alu_cp(self.reg.h);
                1
            }
            0xBD => {
                self.alu_cp(self.reg.l);
                1
            }
            0xBE => {
                self.alu_cp(memory[self.reg.hl() as usize]);
                2
            }
            0xBF => {
                self.alu_cp(self.reg.a);
                1
            }
            0xC0 => {
                if !self.reg.flag(Z) {
                    self.reg.pc = self.pop_stack(memory);
                    5
                } else {
                    2
                }
            }
            0xC1 => {
                let res = self.pop_stack(memory);
                self.reg.set_bc(res);
                3
            }
            0xC2 => {
                let nn = self.read_word(memory);
                if !self.reg.flag(Z) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xC3 => {
                let nn = self.read_word(memory);
                self.reg.pc = nn;
                4
            }
            0xC4 => {
                let nn = self.read_word(memory);
                if !self.reg.flag(Z) {
                    self.push_stack(memory, self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xC5 => {
                let rr = self.reg.bc();
                self.push_stack(memory, rr);
                4
            }
            0xC6 => {
                let n = self.read_byte(memory);
                self.alu_add(n);
                2
            }
            // 0xC7 => {}
            0xC8 => {
                if self.reg.flag(Z) {
                    self.reg.pc = self.pop_stack(memory);
                    5
                } else {
                    2
                }
            }
            0xC9 => {
                self.reg.pc = self.pop_stack(memory);
                4
            }
            0xCA => {
                let nn = self.read_word(memory);
                if self.reg.flag(Z) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xCB => {
                let op = self.read_byte(memory);
                self.exec_cb(op, memory)
            }
            0xCC => {
                let nn = self.read_word(memory);
                if self.reg.flag(Z) {
                    self.push_stack(memory, self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xCD => {
                let nn = self.read_word(memory);
                self.push_stack(memory, self.reg.pc);
                self.reg.pc = nn;
                6
            }
            0xCE => {
                let n = self.read_byte(memory);
                self.alu_adc(n);
                2
            }
            // 0xCF => {}
            0xD0 => {
                if !self.reg.flag(C) {
                    self.reg.pc = self.pop_stack(memory);
                    5
                } else {
                    2
                }
            }
            0xD1 => {
                let res = self.pop_stack(memory);
                self.reg.set_de(res);
                3
            }
            0xD2 => {
                let nn = self.read_word(memory);
                if !self.reg.flag(C) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xD4 => {
                let nn = self.read_word(memory);
                if !self.reg.flag(C) {
                    self.push_stack(memory, self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xD5 => {
                let rr = self.reg.de();
                self.push_stack(memory, rr);
                4
            }
            0xD6 => {
                let n = self.read_byte(memory);
                self.alu_sub(n);
                2
            }
            // 0xD7 => {}
            0xD8 => {
                if self.reg.flag(C) {
                    self.reg.pc = self.pop_stack(memory);
                    5
                } else {
                    2
                }
            }
            0xD9 => {
                self.reg.pc = self.pop_stack(memory);
                self.ime = true;
                4
            }
            0xDA => {
                let nn = self.read_word(memory);
                if self.reg.flag(C) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xDC => {
                let nn = self.read_word(memory);
                if self.reg.flag(C) {
                    self.push_stack(memory, self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xDE => {
                let n = self.read_byte(memory);
                self.alu_sbc(n);
                2
            }
            // 0xDF => {}
            0xE0 => {
                let n = self.read_byte(memory);
                let addr = (0xFF << 8) | (n as u16);
                memory[addr as usize] = self.reg.a;
                3
            }
            0xE1 => {
                let res = self.pop_stack(memory);
                self.reg.set_hl(res);
                3
            }
            0xE2 => {
                let addr = (0xFF << 8) | (self.reg.c as u16);
                memory[addr as usize] = self.reg.a;
                2
            }
            0xE5 => {
                let rr = self.reg.hl();
                self.push_stack(memory, rr);
                4
            }
            0xE6 => {
                let n = self.read_byte(memory);
                self.alu_and(n);
                2
            }
            // 0xE7 => {}
            0xE8 => {
                let e = self.read_byte(memory) as i8 as i16 as u16;
                let res = self.reg.sp.wrapping_add(e);
                self.reg.set_flag(Z, false);
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, (res & 0x1000) == 0x1000);
                self.reg
                    .set_flag(C, (self.reg.sp as u32) + (e as u32) > 0xFFFF);
                self.reg.sp = res;
                4
            }
            0xE9 => {
                self.reg.pc = self.reg.hl();
                1
            }
            0xEA => {
                let nn = self.read_word(memory);
                memory[nn as usize] = self.reg.a;
                4
            }
            0xEE => {
                let n = self.read_byte(memory);
                self.alu_xor(n);
                2
            }
            0xEF => {
                self.push_stack(memory, self.reg.pc);
                self.reg.pc = 0x28;
                4
            }
            0xF0 => {
                let n = self.read_byte(memory);
                let addr = (0xFF << 8) | (n as u16);
                self.reg.a = memory[addr as usize];
                3
            }
            0xF1 => {
                let res = self.pop_stack(memory);
                self.reg.set_af(res);
                3
            }
            0xF2 => {
                let addr = (0xFF << 8) | (self.reg.c as u16);
                self.reg.a = memory[addr as usize];
                2
            }
            0xF3 => {
                self.ime = false;
                1
            }
            0xF5 => {
                let rr = self.reg.af();
                self.push_stack(memory, rr);
                4
            }
            0xF6 => {
                let n = self.read_byte(memory);
                self.alu_or(n);
                2
            }
            0xF7 => {
                self.push_stack(memory, self.reg.pc);
                self.reg.pc = 0x30;
                4
            }
            0xF8 => {
                let e = self.read_byte(memory) as i8 as i16 as u16;
                let res = self.reg.sp + e;
                self.reg.set_hl(res);
                3
            }
            0xF9 => {
                self.reg.sp = self.reg.hl();
                2
            }
            0xFA => {
                let nn = self.read_word(memory);
                self.reg.a = memory[nn as usize];
                4
            }
            0xFB => {
                self.ime_scheduled = true;
                1
            }
            0xFE => {
                let n = self.read_byte(memory);
                self.alu_cp(n);
                2
            }
            0xFF => {
                self.push_stack(memory, self.reg.pc);
                self.reg.pc = 0x38;
                4
            }
            _ => panic!("Unimplemented opcode: 0x{:02X}", opcode),
        }
    }

    fn exec_cb(&mut self, op: u8, memory: &mut [u8; 0xFFFF]) -> u32 {
        match op {
            // 0x00 => {}
            // 0x01 => {}
            // 0x02 => {}
            // 0x03 => {}
            // 0x04 => {}
            // 0x05 => {}
            // 0x06 => {}
            // 0x07 => {}
            // 0x08 => {}
            // 0x09 => {}
            // 0x0A => {}
            // 0x0B => {}
            // 0x0C => {}
            // 0x0D => {}
            // 0x0E => {}
            // 0x0F => {}
            // 0x10 => {}
            // 0x11 => {}
            // 0x12 => {}
            // 0x13 => {}
            // 0x14 => {}
            // 0x15 => {}
            // 0x16 => {}
            // 0x17 => {}
            // 0x18 => {}
            // 0x19 => {}
            // 0x1A => {}
            // 0x1B => {}
            // 0x1C => {}
            // 0x1D => {}
            // 0x1E => {}
            // 0x1F => {}
            // 0x20 => {}
            // 0x21 => {}
            // 0x22 => {}
            // 0x23 => {}
            // 0x24 => {}
            // 0x25 => {}
            // 0x26 => {}
            // 0x27 => {}
            // 0x28 => {}
            // 0x29 => {}
            // 0x2A => {}
            // 0x2B => {}
            // 0x2C => {}
            // 0x2D => {}
            // 0x2E => {}
            // 0x2F => {}
            0x30 => {
                self.reg.b = self.alu_swap(self.reg.b);
                2
            }
            0x31 => {
                self.reg.c = self.alu_swap(self.reg.c);
                2
            }
            0x32 => {
                self.reg.d = self.alu_swap(self.reg.d);
                2
            }
            0x33 => {
                self.reg.e = self.alu_swap(self.reg.e);
                2
            }
            0x34 => {
                self.reg.h = self.alu_swap(self.reg.h);
                2
            }
            0x35 => {
                self.reg.l = self.alu_swap(self.reg.l);
                2
            }
            0x36 => {
                memory[self.reg.hl() as usize] = self.alu_swap(memory[self.reg.hl() as usize]);
                4
            }
            0x37 => {
                self.reg.a = self.alu_swap(self.reg.a);
                2
            }
            // 0x38 => {}
            // 0x39 => {}
            // 0x3A => {}
            // 0x3B => {}
            // 0x3C => {}
            // 0x3D => {}
            // 0x3E => {}
            0x3F => {
                self.reg.a = self.alu_srl(self.reg.a);
                2
            }
            0x40 => {
                self.alu_bit(0, self.reg.b);
                2
            }
            0x41 => {
                self.alu_bit(0, self.reg.c);
                2
            }
            0x42 => {
                self.alu_bit(0, self.reg.d);
                2
            }
            0x43 => {
                self.alu_bit(0, self.reg.e);
                2
            }
            0x44 => {
                self.alu_bit(0, self.reg.h);
                2
            }
            0x45 => {
                self.alu_bit(0, self.reg.l);
                2
            }
            0x46 => {
                self.alu_bit(0, memory[self.reg.hl() as usize]);
                3
            }
            0x47 => {
                self.alu_bit(0, self.reg.a);
                2
            }
            0x48 => {
                self.alu_bit(1, self.reg.b);
                2
            }
            0x49 => {
                self.alu_bit(1, self.reg.c);
                2
            }
            0x4A => {
                self.alu_bit(1, self.reg.d);
                2
            }
            0x4B => {
                self.alu_bit(1, self.reg.e);
                2
            }
            0x4C => {
                self.alu_bit(1, self.reg.h);
                2
            }
            0x4D => {
                self.alu_bit(1, self.reg.l);
                2
            }
            0x4E => {
                self.alu_bit(1, memory[self.reg.hl() as usize]);
                3
            }
            0x4F => {
                self.alu_bit(1, self.reg.a);
                2
            }
            0x50 => {
                self.alu_bit(2, self.reg.b);
                2
            }
            0x51 => {
                self.alu_bit(2, self.reg.c);
                2
            }
            0x52 => {
                self.alu_bit(2, self.reg.d);
                2
            }
            0x53 => {
                self.alu_bit(2, self.reg.e);
                2
            }
            0x54 => {
                self.alu_bit(2, self.reg.h);
                2
            }
            0x55 => {
                self.alu_bit(2, self.reg.l);
                2
            }
            0x56 => {
                self.alu_bit(2, memory[self.reg.hl() as usize]);
                3
            }
            0x57 => {
                self.alu_bit(2, self.reg.a);
                2
            }
            0x58 => {
                self.alu_bit(3, self.reg.b);
                2
            }
            0x59 => {
                self.alu_bit(3, self.reg.c);
                2
            }
            0x5A => {
                self.alu_bit(3, self.reg.d);
                2
            }
            0x5B => {
                self.alu_bit(3, self.reg.e);
                2
            }
            0x5C => {
                self.alu_bit(3, self.reg.h);
                2
            }
            0x5D => {
                self.alu_bit(3, self.reg.l);
                2
            }
            0x5E => {
                self.alu_bit(3, memory[self.reg.hl() as usize]);
                3
            }
            0x5F => {
                self.alu_bit(3, self.reg.a);
                2
            }
            0x60 => {
                self.alu_bit(4, self.reg.b);
                2
            }
            0x61 => {
                self.alu_bit(4, self.reg.c);
                2
            }
            0x62 => {
                self.alu_bit(4, self.reg.d);
                2
            }
            0x63 => {
                self.alu_bit(4, self.reg.e);
                2
            }
            0x64 => {
                self.alu_bit(4, self.reg.h);
                2
            }
            0x65 => {
                self.alu_bit(4, self.reg.l);
                2
            }
            0x66 => {
                self.alu_bit(4, memory[self.reg.hl() as usize]);
                3
            }
            0x67 => {
                self.alu_bit(4, self.reg.a);
                2
            }
            0x68 => {
                self.alu_bit(5, self.reg.b);
                2
            }
            0x69 => {
                self.alu_bit(5, self.reg.c);
                2
            }
            0x6A => {
                self.alu_bit(5, self.reg.d);
                2
            }
            0x6B => {
                self.alu_bit(5, self.reg.e);
                2
            }
            0x6C => {
                self.alu_bit(5, self.reg.h);
                2
            }
            0x6D => {
                self.alu_bit(5, self.reg.l);
                2
            }
            0x6E => {
                self.alu_bit(5, memory[self.reg.hl() as usize]);
                3
            }
            0x6F => {
                self.alu_bit(5, self.reg.a);
                2
            }
            0x70 => {
                self.alu_bit(6, self.reg.b);
                2
            }
            0x71 => {
                self.alu_bit(6, self.reg.c);
                2
            }
            0x72 => {
                self.alu_bit(6, self.reg.d);
                2
            }
            0x73 => {
                self.alu_bit(6, self.reg.e);
                2
            }
            0x74 => {
                self.alu_bit(6, self.reg.h);
                2
            }
            0x75 => {
                self.alu_bit(6, self.reg.l);
                2
            }
            0x76 => {
                self.alu_bit(6, memory[self.reg.hl() as usize]);
                3
            }
            0x77 => {
                self.alu_bit(6, self.reg.a);
                2
            }
            0x78 => {
                self.alu_bit(7, self.reg.b);
                2
            }
            0x79 => {
                self.alu_bit(7, self.reg.c);
                2
            }
            0x7A => {
                self.alu_bit(7, self.reg.d);
                2
            }
            0x7B => {
                self.alu_bit(7, self.reg.e);
                2
            }
            0x7C => {
                self.alu_bit(7, self.reg.h);
                2
            }
            0x7D => {
                self.alu_bit(7, self.reg.l);
                2
            }
            0x7E => {
                self.alu_bit(7, memory[self.reg.hl() as usize]);
                3
            }
            0x7F => {
                self.alu_bit(7, self.reg.a);
                2
            }
            // 0x80 => {}
            // 0x81 => {}
            // 0x82 => {}
            // 0x83 => {}
            // 0x84 => {}
            // 0x85 => {}
            // 0x86 => {}
            // 0x87 => {}
            // 0x88 => {}
            // 0x89 => {}
            // 0x8A => {}
            // 0x8B => {}
            // 0x8C => {}
            // 0x8D => {}
            // 0x8E => {}
            // 0x8F => {}
            // 0x90 => {}
            // 0x91 => {}
            // 0x92 => {}
            // 0x93 => {}
            // 0x94 => {}
            // 0x95 => {}
            // 0x96 => {}
            // 0x97 => {}
            // 0x98 => {}
            // 0x99 => {}
            // 0x9A => {}
            // 0x9B => {}
            // 0x9C => {}
            // 0x9D => {}
            // 0x9E => {}
            // 0x9F => {}
            // 0xA0 => {}
            // 0xA1 => {}
            // 0xA2 => {}
            // 0xA3 => {}
            // 0xA4 => {}
            // 0xA5 => {}
            // 0xA6 => {}
            // 0xA7 => {}
            // 0xA8 => {}
            // 0xA9 => {}
            // 0xAA => {}
            // 0xAB => {}
            // 0xAC => {}
            // 0xAD => {}
            // 0xAE => {}
            // 0xAF => {}
            // 0xB0 => {}
            // 0xB1 => {}
            // 0xB2 => {}
            // 0xB3 => {}
            // 0xB4 => {}
            // 0xB5 => {}
            // 0xB6 => {}
            // 0xB7 => {}
            // 0xB8 => {}
            // 0xB9 => {}
            // 0xBA => {}
            // 0xBB => {}
            // 0xBC => {}
            // 0xBD => {}
            // 0xBE => {}
            // 0xBF => {}
            // 0xC0 => {}
            // 0xC1 => {}
            // 0xC2 => {}
            // 0xC3 => {}
            // 0xC4 => {}
            // 0xC5 => {}
            // 0xC6 => {}
            // 0xC7 => {}
            // 0xC8 => {}
            // 0xC9 => {}
            // 0xCA => {}
            // 0xCB => {}
            // 0xCC => {}
            // 0xCD => {}
            // 0xCE => {}
            // 0xCF => {}
            // 0xD0 => {}
            // 0xD1 => {}
            // 0xD2 => {}
            // 0xD3 => {}
            // 0xD4 => {}
            // 0xD5 => {}
            // 0xD6 => {}
            // 0xD7 => {}
            // 0xD8 => {}
            // 0xD9 => {}
            // 0xDA => {}
            // 0xDB => {}
            // 0xDC => {}
            // 0xDD => {}
            // 0xDE => {}
            // 0xDF => {}
            // 0xE0 => {}
            // 0xE1 => {}
            // 0xE2 => {}
            // 0xE3 => {}
            // 0xE4 => {}
            // 0xE5 => {}
            // 0xE6 => {}
            // 0xE7 => {}
            // 0xE8 => {}
            // 0xE9 => {}
            // 0xEA => {}
            // 0xEB => {}
            // 0xEC => {}
            // 0xED => {}
            // 0xEE => {}
            // 0xEF => {}
            // 0xF0 => {}
            // 0xF1 => {}
            // 0xF2 => {}
            // 0xF3 => {}
            // 0xF4 => {}
            // 0xF5 => {}
            // 0xF6 => {}
            // 0xF7 => {}
            // 0xF8 => {}
            // 0xF9 => {}
            // 0xFA => {}
            // 0xFB => {}
            // 0xFC => {}
            // 0xFD => {}
            // 0xFE => {}
            // 0xFF => {}
            _ => panic!("Unimplemented opcode: 0xCB{:02X}", op),
        }
    }

    // Read/write ops
    fn read_byte(&mut self, memory: &[u8; 0xFFFF]) -> u8 {
        self.reg.pc = self.reg.pc.wrapping_add(1);
        memory[self.reg.pc as usize]
    }

    fn read_word(&mut self, memory: &[u8; 0xFFFF]) -> u16 {
        let lsb = self.read_byte(memory);
        let msb = self.read_byte(memory);
        ((msb as u16) << 8) | (lsb as u16)
    }

    fn push_stack(&mut self, memory: &mut [u8; 0xFFFF], value: u16) {
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        memory[self.reg.sp as usize] = (value >> 8) as u8;
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        memory[self.reg.sp as usize] = value as u8;
    }

    fn pop_stack(&mut self, memory: &[u8; 0xFFFF]) -> u16 {
        self.reg.sp += 1;
        let lsb = memory[self.reg.sp as usize];
        self.reg.sp += 1;
        let msb = memory[self.reg.sp as usize];
        ((msb as u16) << 8) | (lsb as u16)
    }

    // 8-bit ALU ops
    fn alu_add(&mut self, val: u8) {
        let res = self.reg.a.wrapping_add(val);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (res & 0x10) == 0x10);
        self.reg
            .set_flag(C, (self.reg.a as u16) + (val as u16) > 0xFF);
        self.reg.a = res;
    }

    fn alu_adc(&mut self, val: u8) {
        let c = if self.reg.flag(C) { 1 } else { 0 };
        let res = self.reg.a.wrapping_add(c).wrapping_add(val);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (res & 0x10) == 0x10);
        self.reg
            .set_flag(C, (self.reg.a as u16) + (val as u16) + (c as u16) > 0xFF);
        self.reg.a = res;
    }

    fn alu_sub(&mut self, val: u8) {
        let res = self.reg.a.wrapping_sub(val);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (res & 0x10) == 0x10);
        self.reg.set_flag(C, (self.reg.a as u16) < (val as u16));
        self.reg.a = res;
    }

    fn alu_sbc(&mut self, val: u8) {
        let c = if self.reg.flag(C) { 1 } else { 0 };
        let res = self.reg.a.wrapping_sub(c).wrapping_sub(val);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (res & 0x10) == 0x10);
        self.reg
            .set_flag(C, (self.reg.a as u16) < (val as u16) + (c as u16));
        self.reg.a = res;
    }

    fn alu_and(&mut self, val: u8) {
        let res = self.reg.a & val;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    fn alu_xor(&mut self, val: u8) {
        let res = self.reg.a ^ val;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    fn alu_or(&mut self, val: u8) {
        let res = self.reg.a | val;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    fn alu_cp(&mut self, val: u8) {
        let a = self.reg.a;
        self.alu_sub(val);
        self.reg.a = a;
    }

    fn alu_inc(&mut self, val: u8) -> u8 {
        let res = val.wrapping_add(1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (res & 0x10) == 0x10);
        res
    }

    fn alu_dec(&mut self, val: u8) -> u8 {
        let res = val.wrapping_sub(1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (res & 0x10) == 0x10);
        res
    }

    // 8-bit ALU 0xCBXX ops
    fn alu_srl(&mut self, val: u8) -> u8 {
        self.reg.set_flag(C, (val & 1) == 1);
        let res = val >> 1;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    fn alu_swap(&mut self, val: u8) -> u8 {
        self.reg.set_flag(Z, val == 0);
        self.reg.set_flag(C, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        (val << 4) | (val >> 4)
    }

    fn alu_bit(&mut self, bit: u8, val: u8) {
        self.reg.set_flag(Z, (val >> bit) & 1 == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
    }

    // 16-bit ALU ops
    fn alu_add16(&mut self, val: u16) {
        let res = self.reg.hl().wrapping_add(val);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (res & 0x1000) == 0x1000);
        self.reg
            .set_flag(C, (self.reg.hl() as u32) + (val as u32) > 0xFFFF);
        self.reg.set_hl(res);
    }
}

#[cfg(test)]
mod test {
    use super::{C, CPU, H, N, Z};
    #[test]
    fn test_alu() {
        let mut cpu = CPU::new();
        cpu.alu_add(4);
        assert_eq!(cpu.reg.a, 5);
        cpu.reg.set_flag(C, true);
        cpu.alu_adc(5);
        assert_eq!(cpu.reg.a, 11);
        cpu.reg.set_flag(Z, false);
        cpu.alu_bit(1, 0b00000010);
        assert_eq!(cpu.reg.flag(Z), false);
    }
}
