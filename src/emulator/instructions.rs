use super::{Emulator, modrm::ModRM};
use std::process;

impl Emulator {
    pub fn init_instructions(&self) -> [Option<fn(&mut Emulator) -> ()>; 256] {
        let mut instructions: [Option<fn(&mut Emulator) -> ()>; 256] = [None; 256];

        instructions[0x01] = Some(Emulator::add_rm32_r32);
        instructions[0x83] = Some(Emulator::code_83);
        instructions[0x89] = Some(Emulator::mov_rm32_r32);
        instructions[0x8b] = Some(Emulator::mov_r32_rm32);

        for inst in &mut instructions[0xb8..0xc0] {
            *inst = Some(Emulator::mov_r32_imm32);
        }

        instructions[0xc7] = Some(Emulator::mov_rm32_imm32);
        instructions[0xe9] = Some(Emulator::near_jump);
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
        self.eip -= (self.get_signed_code8(1) + 2).wrapping_abs() as u32;
    }

    pub fn near_jump(&mut self) {
        self.eip -= (self.get_signed_code32(1) + 5).wrapping_abs() as u32;
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
        self.set_r32(&mut modrm, rm32);
    }

    pub fn add_rm32_r32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        let r32 = self.get_r32(&modrm);
        let rm32 = self.get_rm32(&modrm);
        self.set_rm32(&mut modrm, rm32 + r32);
    }

    pub fn sub_rm32_imm8(&mut self, modrm: &mut ModRM) {
        let rm32 = self.get_rm32(&modrm);
        let imm8 = self.get_signed_code8(0) as i32 as u32;
        self.eip += 1;
        self.set_rm32(modrm, rm32 - imm8);
    }

    pub fn code_83(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::new();
        self.parse_modrm(&mut modrm);
        
        match unsafe { modrm.op_reg.opcode } {
            5 => {
                self.sub_rm32_imm8(&mut modrm);
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
}
