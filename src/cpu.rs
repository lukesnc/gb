use crate::memory::Mmu;
use crate::register::Flag::*;
use crate::register::Reg;

pub struct Cpu {
    reg: Reg,
    pub membus: Mmu,
    ime: bool,
    ime_next: bool,
    halted: bool,
}

impl Cpu {
    /// Init CPU from existing memory
    pub fn from(mem: Mmu) -> Self {
        Cpu {
            reg: Reg::new(),
            membus: mem,
            ime: false,
            ime_next: false,
            halted: false,
        }
    }

    // CPU cycle
    pub fn cycle(&mut self) {
        // blarggs test - serial output
        if self.membus.read(0xff02) == 0x81 {
            let c = self.membus.read(0xff01);
            if let Some(c) = char::from_u32(c as u32) {
                print!("{}", c);
            }
            self.membus.write(0xff02, 0x0);
        }

        // Check should leave HALT
        if self.halted && self.membus.interrupt_addr().is_some() {
            self.halted = false;
            if !self.ime {
                //println!("halt bug");
            }
        }

        // Handle interrupt
        if self.ime {
            if let Some(addr) = self.membus.interrupt_addr() {
                self.ime = false;
                self.ime_next = false;

                self.push_stack(self.reg.pc);
                self.reg.pc = addr as u16;
                self.membus.do_cycles(5);
            }
        }

        // Check if interrupt scheduled
        self.ime = self.ime_next;

        // NOP if halted
        if self.halted {
            self.membus.do_cycles(1);
            return;
        }

        // Noraml flow: fetch opcode and execute
        let opcode = self.read_byte();
        let m_cycles = self.exec(opcode);
        self.membus.do_cycles(m_cycles);
    }

    // Execute opcode
    fn exec(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x00 => 1,
            0x01 => {
                let nn = self.read_word();
                self.reg.set_bc(nn);
                3
            }
            0x02 => {
                self.membus.write(self.reg.bc(), self.reg.a);
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
                self.reg.b = self.read_byte();
                2
            }
            0x07 => {
                self.reg.a = self.alu_rlc(self.reg.a);
                self.reg.set_flag(Z, false);
                1
            }
            0x08 => {
                let nn = self.read_word();
                self.membus.write(nn, self.reg.sp as u8);
                self.membus.write(nn + 1, (self.reg.sp >> 8) as u8);
                5
            }
            0x09 => {
                self.alu_add16(self.reg.bc());
                2
            }
            0x0A => {
                self.reg.a = self.membus.read(self.reg.bc());
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
                self.reg.c = self.read_byte();
                2
            }
            0x0F => {
                self.reg.a = self.alu_rrc(self.reg.a);
                self.reg.set_flag(Z, false);
                1
            }
            0x10 => 1, // STOP
            0x11 => {
                let nn = self.read_word();
                self.reg.set_de(nn);
                3
            }
            0x12 => {
                self.membus.write(self.reg.de(), self.reg.a);
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
                self.reg.d = self.read_byte();
                2
            }
            0x17 => {
                self.reg.a = self.alu_rl(self.reg.a);
                self.reg.set_flag(Z, false);
                1
            }
            0x18 => {
                let e = self.read_byte() as i8;
                self.reg.pc = self.reg.pc.wrapping_add(e as i16 as u16);
                3
            }
            0x19 => {
                self.alu_add16(self.reg.de());
                2
            }
            0x1A => {
                self.reg.a = self.membus.read(self.reg.de());
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
                self.reg.e = self.read_byte();
                2
            }
            0x1F => {
                self.reg.a = self.alu_rr(self.reg.a);
                self.reg.set_flag(Z, false);
                1
            }
            0x20 => {
                let e = self.read_byte() as i8;
                if !self.reg.flag(Z) {
                    self.reg.pc = self.reg.pc.wrapping_add(e as u16);
                    3
                } else {
                    2
                }
            }
            0x21 => {
                let nn = self.read_word();
                self.reg.set_hl(nn);
                3
            }
            0x22 => {
                self.membus.write(self.reg.hl(), self.reg.a);
                self.reg.set_hl(self.reg.hl().wrapping_add(1));
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
                self.reg.h = self.read_byte();
                2
            }
            0x27 => {
                // DAA instruction credit: https://forums.nesdev.org/viewtopic.php?t=15944
                if !self.reg.flag(N) {
                    if self.reg.flag(C) || self.reg.a > 0x99 {
                        self.reg.a = self.reg.a.wrapping_add(0x60);
                        self.reg.set_flag(C, true);
                    }
                    if self.reg.flag(H) || (self.reg.a & 0x0f) > 0x09 {
                        self.reg.a = self.reg.a.wrapping_add(0x6);
                    }
                } else {
                    if self.reg.flag(C) {
                        self.reg.a = self.reg.a.wrapping_sub(0x60);
                    }
                    if self.reg.flag(H) {
                        self.reg.a = self.reg.a.wrapping_sub(0x6);
                    }
                }
                self.reg.set_flag(Z, self.reg.a == 0);
                self.reg.set_flag(H, false);
                1
            }
            0x28 => {
                let e = self.read_byte();
                if self.reg.flag(Z) {
                    self.reg.pc = self.reg.pc.wrapping_add(e as i8 as i16 as u16);
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
                self.reg.a = self.membus.read(self.reg.hl());
                self.reg.set_hl(self.reg.hl().wrapping_add(1));
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
                self.reg.l = self.read_byte();
                2
            }
            0x2F => {
                self.reg.a = !self.reg.a;
                self.reg.set_flag(N, true);
                self.reg.set_flag(H, true);
                1
            }
            0x30 => {
                if !self.reg.flag(C) {
                    let e = self.read_byte() as i8;
                    self.reg.pc = self.reg.pc.wrapping_add(e as i16 as u16);
                    3
                } else {
                    self.reg.pc = self.reg.pc.wrapping_add(1);
                    2
                }
            }
            0x31 => {
                let nn = self.read_word();
                self.reg.sp = nn;
                3
            }
            0x32 => {
                self.membus.write(self.reg.hl(), self.reg.a);
                self.reg.set_hl(self.reg.hl().wrapping_sub(1));
                2
            }
            0x33 => {
                self.reg.sp = self.reg.sp.wrapping_add(1);
                2
            }
            0x34 => {
                let res = self.alu_inc(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                3
            }
            0x35 => {
                let res = self.alu_dec(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                3
            }
            0x36 => {
                let byte = self.read_byte();
                self.membus.write(self.reg.hl(), byte);
                3
            }
            0x37 => {
                self.reg.set_flag(N, false);
                self.reg.set_flag(H, false);
                self.reg.set_flag(C, true);
                1
            }
            0x38 => {
                let e = self.read_byte();
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
                self.reg.a = self.membus.read(self.reg.hl());
                self.reg.set_hl(self.reg.hl().wrapping_sub(1));
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
                self.reg.a = self.read_byte();
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
                self.reg.b = self.membus.read(self.reg.hl());
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
                self.reg.c = self.membus.read(self.reg.hl());
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
                self.reg.d = self.membus.read(self.reg.hl());
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
                self.reg.e = self.membus.read(self.reg.hl());
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
                self.reg.h = self.membus.read(self.reg.hl());
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
                self.reg.l = self.membus.read(self.reg.hl());
                2
            }
            0x6F => {
                self.reg.l = self.reg.a;
                1
            }
            0x70 => {
                self.membus.write(self.reg.hl(), self.reg.b);
                2
            }
            0x71 => {
                self.membus.write(self.reg.hl(), self.reg.c);
                2
            }
            0x72 => {
                self.membus.write(self.reg.hl(), self.reg.d);
                2
            }
            0x73 => {
                self.membus.write(self.reg.hl(), self.reg.e);
                2
            }
            0x74 => {
                self.membus.write(self.reg.hl(), self.reg.h);
                2
            }
            0x75 => {
                self.membus.write(self.reg.hl(), self.reg.l);
                2
            }
            0x76 => {
                self.halted = true;
                1
            }
            0x77 => {
                self.membus.write(self.reg.hl(), self.reg.a);
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
                self.reg.a = self.membus.read(self.reg.hl());
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
                self.alu_add(self.membus.read(self.reg.hl()));
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
                self.alu_adc(self.membus.read(self.reg.hl()));
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
                self.alu_sub(self.membus.read(self.reg.hl()));
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
                self.alu_sbc(self.membus.read(self.reg.hl()));
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
                self.alu_and(self.membus.read(self.reg.hl()));
                2
            }
            0xA7 => {
                self.alu_and(self.reg.a);
                1
            }
            0xA8 => {
                self.alu_xor(self.reg.b);
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
                self.alu_xor(self.membus.read(self.reg.hl()));
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
                self.alu_or(self.membus.read(self.reg.hl()));
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
                self.alu_cp(self.membus.read(self.reg.hl()));
                2
            }
            0xBF => {
                self.alu_cp(self.reg.a);
                1
            }
            0xC0 => {
                if !self.reg.flag(Z) {
                    self.reg.pc = self.pop_stack();
                    5
                } else {
                    2
                }
            }
            0xC1 => {
                let res = self.pop_stack();
                self.reg.set_bc(res);
                3
            }
            0xC2 => {
                let nn = self.read_word();
                if !self.reg.flag(Z) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xC3 => {
                let nn = self.read_word();
                self.reg.pc = nn;
                4
            }
            0xC4 => {
                let nn = self.read_word();
                if !self.reg.flag(Z) {
                    self.push_stack(self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xC5 => {
                self.push_stack(self.reg.bc());
                4
            }
            0xC6 => {
                let n = self.read_byte();
                self.alu_add(n);
                2
            }
            0xC7 => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x00;
                4
            }
            0xC8 => {
                if self.reg.flag(Z) {
                    self.reg.pc = self.pop_stack();
                    5
                } else {
                    2
                }
            }
            0xC9 => {
                self.reg.pc = self.pop_stack();
                4
            }
            0xCA => {
                let nn = self.read_word();
                if self.reg.flag(Z) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xCB => {
                let op = self.read_byte();
                self.exec_cb(op)
            }
            0xCC => {
                let nn = self.read_word();
                if self.reg.flag(Z) {
                    self.push_stack(self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xCD => {
                let nn = self.read_word();
                self.push_stack(self.reg.pc);
                self.reg.pc = nn;
                6
            }
            0xCE => {
                let n = self.read_byte();
                self.alu_adc(n);
                2
            }
            0xCF => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x08;
                4
            }
            0xD0 => {
                if !self.reg.flag(C) {
                    self.reg.pc = self.pop_stack();
                    5
                } else {
                    2
                }
            }
            0xD1 => {
                let res = self.pop_stack();
                self.reg.set_de(res);
                3
            }
            0xD2 => {
                let nn = self.read_word();
                if !self.reg.flag(C) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xD4 => {
                let nn = self.read_word();
                if !self.reg.flag(C) {
                    self.push_stack(self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xD5 => {
                self.push_stack(self.reg.de());
                4
            }
            0xD6 => {
                let n = self.read_byte();
                self.alu_sub(n);
                2
            }
            0xD7 => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x10;
                4
            }
            0xD8 => {
                if self.reg.flag(C) {
                    self.reg.pc = self.pop_stack();
                    5
                } else {
                    2
                }
            }
            0xD9 => {
                self.reg.pc = self.pop_stack();
                self.ime = true;
                4
            }
            0xDA => {
                let nn = self.read_word();
                if self.reg.flag(C) {
                    self.reg.pc = nn;
                    4
                } else {
                    3
                }
            }
            0xDC => {
                let nn = self.read_word();
                if self.reg.flag(C) {
                    self.push_stack(self.reg.pc);
                    self.reg.pc = nn;
                    6
                } else {
                    3
                }
            }
            0xDE => {
                let n = self.read_byte();
                self.alu_sbc(n);
                2
            }
            0xDF => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x18;
                4
            }
            0xE0 => {
                let n = self.read_byte();
                let addr = 0xFF00 | n as u16;
                self.membus.write(addr, self.reg.a);
                3
            }
            0xE1 => {
                let res = self.pop_stack();
                self.reg.set_hl(res);
                3
            }
            0xE2 => {
                let addr = (0xFF << 8) | (self.reg.c as u16);
                self.membus.write(addr, self.reg.a);
                2
            }
            0xE5 => {
                self.push_stack(self.reg.hl());
                4
            }
            0xE6 => {
                let n = self.read_byte();
                self.alu_and(n);
                2
            }
            0xE7 => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x20;
                4
            }
            0xE8 => {
                self.reg.sp = self.alu_add16imm(self.reg.sp);
                4
            }
            0xE9 => {
                self.reg.pc = self.reg.hl();
                1
            }
            0xEA => {
                let nn = self.read_word();
                self.membus.write(nn, self.reg.a);
                4
            }
            0xEE => {
                let n = self.read_byte();
                self.alu_xor(n);
                2
            }
            0xEF => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x28;
                4
            }
            0xF0 => {
                let addr = 0xFF00 | (self.read_byte() as u16);
                self.reg.a = self.membus.read(addr);
                3
            }
            0xF1 => {
                let res = self.pop_stack();
                self.reg.set_af(res);
                3
            }
            0xF2 => {
                let addr = 0xFF00 | (self.reg.c as u16);
                self.reg.a = self.membus.read(addr);
                2
            }
            0xF3 => {
                self.ime_next = false;
                1
            }
            0xF5 => {
                self.push_stack(self.reg.af());
                4
            }
            0xF6 => {
                let n = self.read_byte();
                self.alu_or(n);
                2
            }
            0xF7 => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x30;
                4
            }
            0xF8 => {
                let res = self.alu_add16imm(self.reg.sp);
                self.reg.set_hl(res);
                3
            }
            0xF9 => {
                self.reg.sp = self.reg.hl();
                2
            }
            0xFA => {
                let nn = self.read_word();
                self.reg.a = self.membus.read(nn);
                4
            }
            0xFB => {
                self.ime_next = true;
                1
            }
            0xFE => {
                let n = self.read_byte();
                self.alu_cp(n);
                2
            }
            0xFF => {
                self.push_stack(self.reg.pc);
                self.reg.pc = 0x38;
                4
            }
            _ => panic!("Unimplemented opcode: 0x{:02X}", opcode),
        }
    }

    fn exec_cb(&mut self, opcode: u8) -> u32 {
        match opcode {
            0x00 => {
                self.reg.b = self.alu_rlc(self.reg.b);
                2
            }
            0x01 => {
                self.reg.c = self.alu_rlc(self.reg.c);
                2
            }
            0x02 => {
                self.reg.d = self.alu_rlc(self.reg.d);
                2
            }
            0x03 => {
                self.reg.e = self.alu_rlc(self.reg.e);
                2
            }
            0x04 => {
                self.reg.h = self.alu_rlc(self.reg.h);
                2
            }
            0x05 => {
                self.reg.l = self.alu_rlc(self.reg.l);
                2
            }
            0x06 => {
                let res = self.alu_rlc(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x07 => {
                self.reg.a = self.alu_rlc(self.reg.a);
                2
            }
            0x08 => {
                self.reg.b = self.alu_rrc(self.reg.b);
                2
            }
            0x09 => {
                self.reg.c = self.alu_rrc(self.reg.c);
                2
            }
            0x0A => {
                self.reg.d = self.alu_rrc(self.reg.d);
                2
            }
            0x0B => {
                self.reg.e = self.alu_rrc(self.reg.e);
                2
            }
            0x0C => {
                self.reg.h = self.alu_rrc(self.reg.h);
                2
            }
            0x0D => {
                self.reg.l = self.alu_rrc(self.reg.l);
                2
            }
            0x0E => {
                let res = self.alu_rrc(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x0F => {
                self.reg.a = self.alu_rrc(self.reg.a);
                2
            }
            0x10 => {
                self.reg.b = self.alu_rl(self.reg.b);
                2
            }
            0x11 => {
                self.reg.c = self.alu_rl(self.reg.c);
                2
            }
            0x12 => {
                self.reg.d = self.alu_rl(self.reg.d);
                2
            }
            0x13 => {
                self.reg.e = self.alu_rl(self.reg.e);
                2
            }
            0x14 => {
                self.reg.h = self.alu_rl(self.reg.h);
                2
            }
            0x15 => {
                self.reg.l = self.alu_rl(self.reg.l);
                2
            }
            0x16 => {
                let res = self.alu_rl(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x17 => {
                self.reg.a = self.alu_rl(self.reg.a);
                2
            }
            0x18 => {
                self.reg.b = self.alu_rr(self.reg.b);
                2
            }
            0x19 => {
                self.reg.c = self.alu_rr(self.reg.c);
                2
            }
            0x1A => {
                self.reg.d = self.alu_rr(self.reg.d);
                2
            }
            0x1B => {
                self.reg.e = self.alu_rr(self.reg.e);
                2
            }
            0x1C => {
                self.reg.h = self.alu_rr(self.reg.h);
                2
            }
            0x1D => {
                self.reg.l = self.alu_rr(self.reg.l);
                2
            }
            0x1E => {
                let res = self.alu_rr(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x1F => {
                self.reg.a = self.alu_rr(self.reg.a);
                2
            }
            0x20 => {
                self.reg.b = self.alu_sla(self.reg.b);
                2
            }
            0x21 => {
                self.reg.c = self.alu_sla(self.reg.c);
                2
            }
            0x22 => {
                self.reg.d = self.alu_sla(self.reg.d);
                2
            }
            0x23 => {
                self.reg.e = self.alu_sla(self.reg.e);
                2
            }
            0x24 => {
                self.reg.h = self.alu_sla(self.reg.h);
                2
            }
            0x25 => {
                self.reg.l = self.alu_sla(self.reg.l);
                2
            }
            0x26 => {
                let res = self.alu_sla(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x27 => {
                self.reg.a = self.alu_sla(self.reg.a);
                2
            }
            0x28 => {
                self.reg.b = self.alu_sra(self.reg.b);
                2
            }
            0x29 => {
                self.reg.c = self.alu_sra(self.reg.c);
                2
            }
            0x2A => {
                self.reg.d = self.alu_sra(self.reg.d);
                2
            }
            0x2B => {
                self.reg.e = self.alu_sra(self.reg.e);
                2
            }
            0x2C => {
                self.reg.h = self.alu_sra(self.reg.h);
                2
            }
            0x2D => {
                self.reg.l = self.alu_sra(self.reg.l);
                2
            }
            0x2E => {
                let res = self.alu_sra(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x2F => {
                self.reg.a = self.alu_sra(self.reg.a);
                2
            }
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
                let res = self.alu_swap(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x37 => {
                self.reg.a = self.alu_swap(self.reg.a);
                2
            }
            0x38 => {
                self.reg.b = self.alu_srl(self.reg.b);
                2
            }
            0x39 => {
                self.reg.c = self.alu_srl(self.reg.c);
                2
            }
            0x3A => {
                self.reg.d = self.alu_srl(self.reg.d);
                2
            }
            0x3B => {
                self.reg.e = self.alu_srl(self.reg.e);
                2
            }
            0x3C => {
                self.reg.h = self.alu_srl(self.reg.h);
                2
            }
            0x3D => {
                self.reg.l = self.alu_srl(self.reg.l);
                2
            }
            0x3E => {
                let res = self.alu_srl(self.membus.read(self.reg.hl()));
                self.membus.write(self.reg.hl(), res);
                4
            }
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
                self.alu_bit(0, self.membus.read(self.reg.hl()));
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
                self.alu_bit(1, self.membus.read(self.reg.hl()));
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
                self.alu_bit(2, self.membus.read(self.reg.hl()));
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
                self.alu_bit(3, self.membus.read(self.reg.hl()));
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
                self.alu_bit(4, self.membus.read(self.reg.hl()));
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
                self.alu_bit(5, self.membus.read(self.reg.hl()));
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
                self.alu_bit(6, self.membus.read(self.reg.hl()));
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
                self.alu_bit(7, self.membus.read(self.reg.hl()));
                3
            }
            0x7F => {
                self.alu_bit(7, self.reg.a);
                2
            }
            0x80 => {
                self.reg.b = self.reg.b & !(1 << 0);
                2
            }
            0x81 => {
                self.reg.c = self.reg.c & !(1 << 0);
                2
            }
            0x82 => {
                self.reg.d = self.reg.d & !(1 << 0);
                2
            }
            0x83 => {
                self.reg.e = self.reg.e & !(1 << 0);
                2
            }
            0x84 => {
                self.reg.h = self.reg.h & !(1 << 0);
                2
            }
            0x85 => {
                self.reg.l = self.reg.l & !(1 << 0);
                2
            }
            0x86 => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 0);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x87 => {
                self.reg.a = self.reg.a & !(1 << 0);
                2
            }
            0x88 => {
                self.reg.b = self.reg.b & !(1 << 1);
                2
            }
            0x89 => {
                self.reg.c = self.reg.c & !(1 << 1);
                2
            }
            0x8A => {
                self.reg.d = self.reg.d & !(1 << 1);
                2
            }
            0x8B => {
                self.reg.e = self.reg.e & !(1 << 1);
                2
            }
            0x8C => {
                self.reg.h = self.reg.h & !(1 << 1);
                2
            }
            0x8D => {
                self.reg.l = self.reg.l & !(1 << 1);
                2
            }
            0x8E => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 1);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x8F => {
                self.reg.a = self.reg.a & !(1 << 1);
                2
            }
            0x90 => {
                self.reg.b = self.reg.b & !(1 << 2);
                2
            }
            0x91 => {
                self.reg.c = self.reg.c & !(1 << 2);
                2
            }
            0x92 => {
                self.reg.d = self.reg.d & !(1 << 2);
                2
            }
            0x93 => {
                self.reg.e = self.reg.e & !(1 << 2);
                2
            }
            0x94 => {
                self.reg.h = self.reg.h & !(1 << 2);
                2
            }
            0x95 => {
                self.reg.l = self.reg.l & !(1 << 2);
                2
            }
            0x96 => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 2);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x97 => {
                self.reg.a = self.reg.a & !(1 << 2);
                2
            }
            0x98 => {
                self.reg.b = self.reg.b & !(1 << 3);
                2
            }
            0x99 => {
                self.reg.c = self.reg.c & !(1 << 3);
                2
            }
            0x9A => {
                self.reg.d = self.reg.d & !(1 << 3);
                2
            }
            0x9B => {
                self.reg.e = self.reg.e & !(1 << 3);
                2
            }
            0x9C => {
                self.reg.h = self.reg.h & !(1 << 3);
                2
            }
            0x9D => {
                self.reg.l = self.reg.l & !(1 << 3);
                2
            }
            0x9E => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 3);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0x9F => {
                self.reg.a = self.reg.a & !(1 << 3);
                2
            }
            0xA0 => {
                self.reg.b = self.reg.b & !(1 << 4);
                2
            }
            0xA1 => {
                self.reg.c = self.reg.c & !(1 << 4);
                2
            }
            0xA2 => {
                self.reg.d = self.reg.d & !(1 << 4);
                2
            }
            0xA3 => {
                self.reg.e = self.reg.e & !(1 << 4);
                2
            }
            0xA4 => {
                self.reg.h = self.reg.h & !(1 << 4);
                2
            }
            0xA5 => {
                self.reg.l = self.reg.l & !(1 << 4);
                2
            }
            0xA6 => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 4);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xA7 => {
                self.reg.a = self.reg.a & !(1 << 4);
                2
            }
            0xA8 => {
                self.reg.b = self.reg.b & !(1 << 5);
                2
            }
            0xA9 => {
                self.reg.c = self.reg.c & !(1 << 5);
                2
            }
            0xAA => {
                self.reg.d = self.reg.d & !(1 << 5);
                2
            }
            0xAB => {
                self.reg.e = self.reg.e & !(1 << 5);
                2
            }
            0xAC => {
                self.reg.h = self.reg.h & !(1 << 5);
                2
            }
            0xAD => {
                self.reg.l = self.reg.l & !(1 << 5);
                2
            }
            0xAE => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 5);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xAF => {
                self.reg.a = self.reg.a & !(1 << 5);
                2
            }
            0xB0 => {
                self.reg.b = self.reg.b & !(1 << 6);
                2
            }
            0xB1 => {
                self.reg.c = self.reg.c & !(1 << 6);
                2
            }
            0xB2 => {
                self.reg.d = self.reg.d & !(1 << 6);
                2
            }
            0xB3 => {
                self.reg.e = self.reg.e & !(1 << 6);
                2
            }
            0xB4 => {
                self.reg.h = self.reg.h & !(1 << 6);
                2
            }
            0xB5 => {
                self.reg.l = self.reg.l & !(1 << 6);
                2
            }
            0xB6 => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 6);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xB7 => {
                self.reg.a = self.reg.a & !(1 << 6);
                2
            }
            0xB8 => {
                self.reg.b = self.reg.b & !(1 << 7);
                2
            }
            0xB9 => {
                self.reg.c = self.reg.c & !(1 << 7);
                2
            }
            0xBA => {
                self.reg.d = self.reg.d & !(1 << 7);
                2
            }
            0xBB => {
                self.reg.e = self.reg.e & !(1 << 7);
                2
            }
            0xBC => {
                self.reg.h = self.reg.h & !(1 << 7);
                2
            }
            0xBD => {
                self.reg.l = self.reg.l & !(1 << 7);
                2
            }
            0xBE => {
                let res = self.membus.read(self.reg.hl()) & !(1 << 7);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xBF => {
                self.reg.a = self.reg.a & !(1 << 7);
                2
            }
            0xC0 => {
                self.reg.b = self.reg.b | (1 << 0);
                2
            }
            0xC1 => {
                self.reg.c = self.reg.c | (1 << 0);
                2
            }
            0xC2 => {
                self.reg.d = self.reg.d | (1 << 0);
                2
            }
            0xC3 => {
                self.reg.e = self.reg.e | (1 << 0);
                2
            }
            0xC4 => {
                self.reg.h = self.reg.h | (1 << 0);
                2
            }
            0xC5 => {
                self.reg.l = self.reg.l | (1 << 0);
                2
            }
            0xC6 => {
                let res = self.membus.read(self.reg.hl()) | (1 << 0);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xC7 => {
                self.reg.a = self.reg.a | (1 << 0);
                2
            }
            0xC8 => {
                self.reg.b = self.reg.b | (1 << 1);
                2
            }
            0xC9 => {
                self.reg.c = self.reg.c | (1 << 1);
                2
            }
            0xCA => {
                self.reg.d = self.reg.d | (1 << 1);
                2
            }
            0xCB => {
                self.reg.e = self.reg.e | (1 << 1);
                2
            }
            0xCC => {
                self.reg.h = self.reg.h | (1 << 1);
                2
            }
            0xCD => {
                self.reg.l = self.reg.l | (1 << 1);
                2
            }
            0xCE => {
                let res = self.membus.read(self.reg.hl()) | (1 << 1);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xCF => {
                self.reg.a = self.reg.a | (1 << 1);
                2
            }
            0xD0 => {
                self.reg.b = self.reg.b | (1 << 2);
                2
            }
            0xD1 => {
                self.reg.c = self.reg.c | (1 << 2);
                2
            }
            0xD2 => {
                self.reg.d = self.reg.d | (1 << 2);
                2
            }
            0xD3 => {
                self.reg.e = self.reg.e | (1 << 2);
                2
            }
            0xD4 => {
                self.reg.h = self.reg.h | (1 << 2);
                2
            }
            0xD5 => {
                self.reg.l = self.reg.l | (1 << 2);
                2
            }
            0xD6 => {
                let res = self.membus.read(self.reg.hl()) | (1 << 2);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xD7 => {
                self.reg.a = self.reg.a | (1 << 2);
                2
            }
            0xD8 => {
                self.reg.b = self.reg.b | (1 << 3);
                2
            }
            0xD9 => {
                self.reg.c = self.reg.c | (1 << 3);
                2
            }
            0xDA => {
                self.reg.d = self.reg.d | (1 << 3);
                2
            }
            0xDB => {
                self.reg.e = self.reg.e | (1 << 3);
                2
            }
            0xDC => {
                self.reg.h = self.reg.h | (1 << 3);
                2
            }
            0xDD => {
                self.reg.l = self.reg.l | (1 << 3);
                2
            }
            0xDE => {
                let res = self.membus.read(self.reg.hl()) | (1 << 3);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xDF => {
                self.reg.a = self.reg.a | (1 << 3);
                2
            }
            0xE0 => {
                self.reg.b = self.reg.b | (1 << 4);
                2
            }
            0xE1 => {
                self.reg.c = self.reg.c | (1 << 4);
                2
            }
            0xE2 => {
                self.reg.d = self.reg.d | (1 << 4);
                2
            }
            0xE3 => {
                self.reg.e = self.reg.e | (1 << 4);
                2
            }
            0xE4 => {
                self.reg.h = self.reg.h | (1 << 4);
                2
            }
            0xE5 => {
                self.reg.l = self.reg.l | (1 << 4);
                2
            }
            0xE6 => {
                let res = self.membus.read(self.reg.hl()) | (1 << 4);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xE7 => {
                self.reg.a = self.reg.a | (1 << 4);
                2
            }
            0xE8 => {
                self.reg.b = self.reg.b | (1 << 5);
                2
            }
            0xE9 => {
                self.reg.c = self.reg.c | (1 << 5);
                2
            }
            0xEA => {
                self.reg.d = self.reg.d | (1 << 5);
                2
            }
            0xEB => {
                self.reg.e = self.reg.e | (1 << 5);
                2
            }
            0xEC => {
                self.reg.h = self.reg.h | (1 << 5);
                2
            }
            0xED => {
                self.reg.l = self.reg.l | (1 << 5);
                2
            }
            0xEE => {
                let res = self.membus.read(self.reg.hl()) | (1 << 5);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xEF => {
                self.reg.a = self.reg.a | (1 << 5);
                2
            }
            0xF0 => {
                self.reg.b = self.reg.b | (1 << 6);
                2
            }
            0xF1 => {
                self.reg.c = self.reg.c | (1 << 6);
                2
            }
            0xF2 => {
                self.reg.d = self.reg.d | (1 << 6);
                2
            }
            0xF3 => {
                self.reg.e = self.reg.e | (1 << 6);
                2
            }
            0xF4 => {
                self.reg.h = self.reg.h | (1 << 6);
                2
            }
            0xF5 => {
                self.reg.l = self.reg.l | (1 << 6);
                2
            }
            0xF6 => {
                let res = self.membus.read(self.reg.hl()) | (1 << 6);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xF7 => {
                self.reg.a = self.reg.a | (1 << 6);
                2
            }
            0xF8 => {
                self.reg.b = self.reg.b | (1 << 7);
                2
            }
            0xF9 => {
                self.reg.c = self.reg.c | (1 << 7);
                2
            }
            0xFA => {
                self.reg.d = self.reg.d | (1 << 7);
                2
            }
            0xFB => {
                self.reg.e = self.reg.e | (1 << 7);
                2
            }
            0xFC => {
                self.reg.h = self.reg.h | (1 << 7);
                2
            }
            0xFD => {
                self.reg.l = self.reg.l | (1 << 7);
                2
            }
            0xFE => {
                let res = self.membus.read(self.reg.hl()) | (1 << 7);
                self.membus.write(self.reg.hl(), res);
                4
            }
            0xFF => {
                self.reg.a = self.reg.a | (1 << 7);
                2
            }
        }
    }

    // Read/write ops
    /// Read byte at the PC
    fn read_byte(&mut self) -> u8 {
        let byte = self.membus.read(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        byte
    }

    /// Read word (2 bytes) at the PC
    fn read_word(&mut self) -> u16 {
        let lsb = self.read_byte();
        let msb = self.read_byte();
        ((msb as u16) << 8) | (lsb as u16)
    }

    fn push_stack(&mut self, value: u16) {
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.membus.write(self.reg.sp, (value >> 8) as u8);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.membus.write(self.reg.sp, value as u8);
    }

    fn pop_stack(&mut self) -> u16 {
        let lsb = self.membus.read(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);
        let msb = self.membus.read(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);
        ((msb as u16) << 8) | (lsb as u16)
    }

    // 8-bit ALU ops
    fn alu_add(&mut self, b: u8) {
        let a = self.reg.a;
        let res = a.wrapping_add(b);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (a & 0xF) + (b & 0xF) > 0xF);
        self.reg.set_flag(C, (a as u16) + (b as u16) > 0xFF);
        self.reg.a = res;
    }

    fn alu_adc(&mut self, b: u8) {
        let a = self.reg.a;
        let c = if self.reg.flag(C) { 1 } else { 0 };
        let res = a.wrapping_add(c).wrapping_add(b);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (a & 0xF) + (b & 0xF) + c > 0xF);
        self.reg
            .set_flag(C, (a as u16) + (b as u16) + (c as u16) > 0xFF);
        self.reg.a = res;
    }

    fn alu_sub(&mut self, b: u8) {
        let a = self.reg.a;
        let res = a.wrapping_sub(b);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (a & 0x0F) < (b & 0x0F));
        self.reg.set_flag(C, (a as u16) < (b as u16));
        self.reg.a = res;
    }

    fn alu_sbc(&mut self, b: u8) {
        let a = self.reg.a;
        let c = if self.reg.flag(C) { 1 } else { 0 };
        let res = a.wrapping_sub(c).wrapping_sub(b);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (a & 0x0F) < (b & 0x0F) + c);
        self.reg.set_flag(C, (a as u16) < (b as u16) + (c as u16));
        self.reg.a = res;
    }

    fn alu_and(&mut self, b: u8) {
        let res = self.reg.a & b;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    fn alu_xor(&mut self, b: u8) {
        let res = self.reg.a ^ b;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    fn alu_or(&mut self, b: u8) {
        let res = self.reg.a | b;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        self.reg.set_flag(C, false);
        self.reg.a = res;
    }

    fn alu_cp(&mut self, b: u8) {
        let a = self.reg.a;
        self.alu_sub(b);
        self.reg.a = a;
    }

    fn alu_inc(&mut self, b: u8) -> u8 {
        let res = b.wrapping_add(1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (b & 0x0F) + 1 > 0x0F);
        res
    }

    fn alu_dec(&mut self, b: u8) -> u8 {
        let res = b.wrapping_sub(1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, true);
        self.reg.set_flag(H, (b & 0x0F) == 0);
        res
    }

    fn alu_swap(&mut self, b: u8) -> u8 {
        self.reg.set_flag(Z, b == 0);
        self.reg.set_flag(C, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        (b << 4) | (b >> 4)
    }

    fn alu_bit(&mut self, bit: u8, b: u8) {
        self.reg.set_flag(Z, (b >> bit) & 1 == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, true);
    }

    fn alu_rr(&mut self, b: u8) -> u8 {
        let c = b & 1;
        let res = (b >> 1) | (if self.reg.flag(C) { 0x80 } else { 0 });
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(C, c == 1);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    fn alu_rrc(&mut self, b: u8) -> u8 {
        let c = b & 1;
        let res = (b >> 1) | (c << 7);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(C, c == 1);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    fn alu_rl(&mut self, b: u8) -> u8 {
        let c = b & 0x80;
        let res = (b << 1) | (if self.reg.flag(C) { 1 } else { 0 });
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(C, c == 0x80);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    fn alu_rlc(&mut self, b: u8) -> u8 {
        let c = b & 0x80;
        let res = (b << 1) | (c >> 7);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(C, c == 0x80);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    fn alu_srl(&mut self, b: u8) -> u8 {
        self.reg.set_flag(C, (b & 1) == 1);
        let res = b >> 1;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    fn alu_sla(&mut self, b: u8) -> u8 {
        self.reg.set_flag(C, (b & 0x80) == 0x80);
        let res = b << 1;
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    fn alu_sra(&mut self, b: u8) -> u8 {
        self.reg.set_flag(C, (b & 1) == 1);
        let res = (b & 0x80) | (b >> 1);
        self.reg.set_flag(Z, res == 0);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, false);
        res
    }

    // 16-bit ALU ops
    fn alu_add16(&mut self, b: u16) {
        let hl = self.reg.hl();
        let res = hl.wrapping_add(b);
        self.reg.set_flag(N, false);
        self.reg
            .set_flag(H, (((hl & 0xFFF) + (b & 0xFFF)) & 0x1000) == 0x1000);
        self.reg.set_flag(C, hl > 0xFFFF - b);
        self.reg.set_hl(res);
    }

    fn alu_add16imm(&mut self, a: u16) -> u16 {
        let e = self.read_byte() as i8 as i16 as u16;
        let res = a.wrapping_add(e);
        self.reg.set_flag(Z, false);
        self.reg.set_flag(N, false);
        self.reg.set_flag(H, (a & 0xF) + (e & 0xF) > 0xF);
        self.reg.set_flag(C, (a & 0x00FF) + (e & 0x00FF) > 0x00FF);
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halt_bug() {
        let mut mem = Mmu::new();
        mem.load_rom(&"../testroms/blargg/cpu_instrs/individual/02-interrupts.gb".to_string());
        let mut cpu = Cpu::from(mem);

        loop {
            cpu.cycle();
        }
    }
}
