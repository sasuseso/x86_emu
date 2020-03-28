use super::{Emulator, modrm::ModRM, add_i2u_32, Eflags, RegIdx, io_func};
use std::process;

impl Emulator {
    pub fn init_instructions(&self) -> [Option<fn(&mut Emulator) -> ()>; 256] {
        let mut instructions: [Option<fn(&mut Emulator) -> ()>; 256] = [None; 256];

        instructions[0x01] = Some(Emulator::add_rm32_r32);

        instructions[0x3b] = Some(Emulator::cmp_r32_rm32);
        instructions[0x3c] = Some(Emulator::cmp_al_imm8);
        instructions[0x3d] = Some(Emulator::cmp_eax_imm8);

        for inst in &mut instructions[0x40..0x48] {
            *inst = Some(Emulator::inc_r32);
        }

        for inst in &mut instructions[0x50..0x58] {
            *inst = Some(Emulator::push_r32);
        }

        for inst in &mut instructions[0x58..0x60] {
            *inst = Some(Emulator::pop_r32);
        }

        instructions[0x68] = Some(Emulator::push_imm32);
        instructions[0x6a] = Some(Emulator::push_imm8);

        instructions[0x70] = Some(Emulator::jo);
        instructions[0x71] = Some(Emulator::jno);
        instructions[0x72] = Some(Emulator::jc);
        instructions[0x73] = Some(Emulator::jnc);
        instructions[0x74] = Some(Emulator::jz);
        instructions[0x75] = Some(Emulator::jnz);
        instructions[0x78] = Some(Emulator::js);
        instructions[0x79] = Some(Emulator::jns);
        instructions[0x7c] = Some(Emulator::jl);
        instructions[0x7e] = Some(Emulator::jle);

        instructions[0x83] = Some(Emulator::code_83);
        instructions[0x88] = Some(Emulator::mov_rm8_r8);
        instructions[0x89] = Some(Emulator::mov_rm32_r32);
        instructions[0x8a] = Some(Emulator::mov_r8_rm8);
        instructions[0x8b] = Some(Emulator::mov_r32_rm32);

        for inst in &mut instructions[0xb0..0xb8] {
            *inst = Some(Emulator::mov_r8_imm8);
        }

        for inst in &mut instructions[0xb8..0xc0] {
            *inst = Some(Emulator::mov_r32_imm32);
        }

        instructions[0xc3] = Some(Emulator::ret);
        instructions[0xc7] = Some(Emulator::mov_rm32_imm32);
        instructions[0xc9] = Some(Emulator::leave);
        instructions[0xcd] = Some(Emulator::int);
        instructions[0xe8] = Some(Emulator::call_rel32);
        instructions[0xe9] = Some(Emulator::near_jump);
        instructions[0xec] = Some(Emulator::in_al_dx);
        instructions[0xee] = Some(Emulator::out_dx_al);
        instructions[0xeb] = Some(Emulator::short_jump);
        instructions[0xff] = Some(Emulator::code_ff);

        instructions
    }

    pub fn mov_r32_imm32(&mut self) {
        let reg: u8 = self.get_code8(0) -  0xb8;
        let val = self.get_code32(1);
        self.registers.regs[reg as usize] = val;
        self.eip += 5;
    }

    pub fn short_jump(&mut self) {
        self.eip = add_i2u_32(self.eip, self.get_signed_code8(1) as i32 + 2);
    }

    pub fn near_jump(&mut self) {
        self.eip = add_i2u_32(self.eip, self.get_signed_code32(1) as i32 + 5);
    }

    pub fn mov_rm32_imm32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let val: u32 = self.get_code32(0);
        self.eip += 4;

        self.set_rm32(&mut modrm, val);
    }

    pub fn mov_rm32_r32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let r32 = self.get_r32(&modrm);
        self.set_rm32(&mut modrm, r32);
    }

    pub fn mov_r32_rm32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let rm32 = self.get_rm32(&mut modrm);
        self.set_r32(&modrm, rm32);
    }

    pub fn add_rm32_r32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let r32 = self.get_r32(&modrm);
        let rm32 = self.get_rm32(&modrm);
        let res = rm32 + r32;
        self.set_rm32(&mut modrm, rm32 + r32);
        self.update_eflags_sub(rm32, r32, res as u64);
    }

    pub fn sub_rm32_imm8(&mut self, modrm: &mut ModRM) {
        let rm32 = self.get_rm32(&modrm);
        let imm8 = self.get_signed_code8(0) as i32 as u32;
        self.eip += 1;
        let res = (rm32 - imm8) as u64;
        self.set_rm32(modrm, rm32 - imm8);
        self.update_eflags_sub(rm32, imm8, res);
    }

    pub fn code_83(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        
        match unsafe { modrm.op_reg.opcode } {
            0 => {
                self.add_rm32_imm8(&mut modrm);
            },
            5 => {
                self.sub_rm32_imm8(&mut modrm);
            },
            7 => {
                self.cmp_rm32_imm8(&mut modrm);
            },
            _ => {
                println!("not implemented : 83 /{}", unsafe { modrm.op_reg.opcode });
                process::exit(1);
            }
        }
    }

    pub fn inc_rm32(&mut self, modrm: &mut ModRM) {
        let val = self.get_rm32(modrm);
        self.set_rm32(modrm, val + 1);
    }

    pub fn code_ff(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);

        match unsafe { modrm.op_reg.opcode } {
            0 => { self.inc_rm32(&mut modrm); },
            _ => {
                println!("not implemented: ff /{}", unsafe { modrm.op_reg.opcode });
                process::exit(1);
            }
        }
    }

    pub fn push_r32(&mut self) {
        let reg = self.get_code8(0) - 0x50;
        self.push32(self.get_register32(reg));
        self.eip += 1;
    }

    pub fn pop_r32(&mut self) {
        let reg = self.get_code8(0) - 0x58;
        let s = self.pop32();
        self.set_register32(reg, s);
        self.eip += 1;
    }

    pub fn call_rel32(&mut self) {
        let diff = self.get_signed_code32(1);
        self.push32(self.eip + 5);
        self.eip = add_i2u_32(self.eip, diff + 5);
    }

    pub fn ret(&mut self) {
        self.eip = self.pop32();
    }

    pub fn leave(&mut self) {
        let ebp = self.get_register32(RegIdx::Ebp as u8);
        self.set_register32(RegIdx::Esp as u8, ebp);
        let r = self.pop32();
        self.set_register32(RegIdx::Ebp as u8, r);
        self.eip += 1;
    }

    pub fn push_imm32(&mut self) {
        let val = self.get_code32(1);
        self.push32(val);
        self.eip += 5;
    }

    pub fn push_imm8(&mut self) {
        let val = self.get_code8(1);
        self.push32(val as u32);
        self.eip += 2;
    }

    pub fn add_rm32_imm8(&mut self, modrm: &mut ModRM) {
        let rm32 = self.get_rm32(modrm);
        let imm8 = self.get_signed_code8(0) as u32;
        self.eip += 1;
        let res = rm32 + imm8;
        self.set_rm32(modrm, res);
        self.update_eflags_sub(rm32, imm8, res as u64);
    }

    pub fn cmp_r32_rm32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let r32 = self.get_r32(&modrm);
        let rm32 = self.get_rm32(&modrm);
        let res = (r32 as i64 - rm32 as i64) as u64;
        self.update_eflags_sub(r32, rm32, res);
    }

    pub fn cmp_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(modrm);
        let imm8 = self.get_signed_code8(0) as u32;
        self.eip += 1;
        let res = (rm32 - imm8) as u64;
        self.update_eflags_sub(rm32, imm8, res);
    }

    pub fn js(&mut self) {
        let diff = if self.check_eflag(Eflags::Sign) {
            self.get_signed_code8(1)
        } else {
            0
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jc(&mut self) {
        let diff = if self.check_eflag(Eflags::Carry) {
            self.get_signed_code8(1)
        } else {
            0
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jz(&mut self) {
        let diff = if self.check_eflag(Eflags::Zero) {
            self.get_signed_code8(1)
        } else {
            0
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jo(&mut self) {
        let diff = if self.check_eflag(Eflags::Overflow) {
            self.get_signed_code8(1)
        } else {
            0
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jns(&mut self) {
        let diff = if self.check_eflag(Eflags::Sign) {
            0
        } else {
            self.get_signed_code8(1)
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jnc(&mut self) {
        let diff = if self.check_eflag(Eflags::Carry) {
            0
        } else {
            self.get_signed_code8(1)
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jnz(&mut self) {
        let diff = if self.check_eflag(Eflags::Zero) {
            0
        } else {
            self.get_signed_code8(1)
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jno(&mut self) {
        let diff = if self.check_eflag(Eflags::Overflow) {
            0
        } else {
            self.get_signed_code8(1)
        };
        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jl(&mut self) {
        let diff = if self.check_eflag(Eflags::Sign)
                       != self.check_eflag(Eflags::Overflow) {
            self.get_signed_code8(1)
        } else {
            0
        };

        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn jle(&mut self) {
        let diff = if self.check_eflag(Eflags::Zero)
            || self.check_eflag(Eflags::Sign)
            != self.check_eflag(Eflags::Overflow) {
            self.get_signed_code8(1)
        } else {
            0
        };

        self.eip = add_i2u_32(self.eip, diff as i32 + 2);
    }

    pub fn in_al_dx(&mut self) {
        let addr = (self.get_register32(2) & 0xffff) as u16;
        let val = io_func::io_in8(addr);
        self.set_register8(0, val);
        self.eip += 1;
    }

    pub fn out_dx_al(&mut self) {
        let addr = (self.get_register32(RegIdx::Edx as u8) & 0xffff) as u16;
        let val = self.get_register8(RegIdx::al()); 
        io_func::io_out8(addr, val);
        self.eip += 1;
    }

    pub fn mov_r8_imm8(&mut self) {
        let reg = self.get_code8(0) - 0xb0;
        self.set_register8(reg as i32, self.get_code8(1));
        self.eip += 2;
    }

    pub fn mov_rm8_r8(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let r8 = self.get_r8(&modrm);
        self.set_rm8(&mut modrm, r8);
    }

    pub fn cmp_al_imm8(&mut self) {
        let val = self.get_code8(1) as u32;
        let al = self.get_register8(RegIdx::al()) as u32;
        let res = (al as i64 - val as i64) as u64;
        self.update_eflags_sub(al, val, res);
        self.eip += 2;
    }

    pub fn cmp_eax_imm8(&mut self) {
        let val = self.get_code32(1);
        let eax = self.get_register32(RegIdx::Eax as u8);
        let res = (eax as i64 - val as i64) as u64;
        self.update_eflags_sub(eax, val, res);
        self.eip += 5;
    }

    pub fn inc_r32(&mut self) {
        let reg = self.get_code8(0) - 0x40;
        self.set_register32(reg, self.get_register32(reg) + 1);
        self.eip += 1;
    }

    pub fn mov_r8_rm8(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let rm8 = self.get_rm8(&modrm);
        self.set_r8(&modrm, rm8);
    }

    pub fn int(&mut self) {
        let int_idx = self.get_code8(1);
        self.eip += 2;

        match int_idx {
            0x10 => { self.bios_video(); },
            _ => { println!("unknown interrupt: {:#02x}", int_idx); }
        }
    }
}
