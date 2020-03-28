use std::process;
use super::add_i2u_32;

#[repr(C)]
pub union OpcodeOrRgndx {
    pub opcode: u8,
    pub reg_idx: u8
}

#[repr(C)]
union Disp {
    disp8: i8,
    disp32: u32,
}

pub struct ModRM {
    pub modu: u8,
    pub op_reg: OpcodeOrRgndx,
    pub rm: u8,
    sib: u8,
    disp: Disp,
}

impl ModRM {
    pub fn new() -> Self {
        ModRM {
            modu: 0,
            op_reg: OpcodeOrRgndx { opcode: 0},
            rm: 0,
            sib: 0,
            disp: Disp { disp8: 0 }
        }
    }
}

impl super::Emulator {
    pub fn parse_modrm(&mut self, modrm: &mut ModRM) {

        let code = self.get_code8(0);
        modrm.modu = (code & 0xc0) >> 6;
        modrm.op_reg = OpcodeOrRgndx { opcode: (code & 0x38) >> 3 };
        modrm.rm = code & 0x07;

        self.eip += 1;

        if modrm.modu != 3 && modrm.rm == 4 {
            modrm.sib = self.get_code8(0);
            self.eip += 1;
        }

        if (modrm.modu == 0 && modrm.rm == 5) || modrm.modu == 2 {
            modrm.disp.disp32 = self.get_signed_code32(0) as u32;
            self.eip += 4;
        } else if modrm.modu == 1 {
            modrm.disp.disp8 = self.get_signed_code8(0);
            self.eip += 1;
        }
    }

    pub fn set_rm32(&mut self, modrm: &ModRM, val: u32) {
        if modrm.modu == 3 {
            self.set_register32(modrm.rm, val);
        } else {
            let addr = self.calc_memory_address(modrm);
            self.set_memory32(addr, val);
        }
    }

    pub fn get_rm32(&self, modrm: &ModRM) -> u32 {
        if modrm.modu == 3 {
            self.get_register32(modrm.rm)
        } else {
            let addr = self.calc_memory_address(modrm);
            self.get_memory32(addr)
        }
    }
    
    pub fn get_rm8(&mut self, modrm: &ModRM) -> u8 {
        if modrm.modu == 3 {
            self.get_register8(modrm.rm as usize)
        } else {
            let addr = self.calc_memory_address(modrm);
            self.get_memory8(addr) as u8
        }
    }

    pub fn set_r8(&mut self, modrm: &ModRM, val: u8) {
        self.set_register8(unsafe { modrm.op_reg.reg_idx } as i32, val)
    }

    pub fn set_r32(&mut self, modrm: &ModRM, val: u32) {
        self.set_register32(unsafe { modrm.op_reg.reg_idx }, val);
    }

    pub fn get_r32(&self, modrm: &ModRM) -> u32 {
        self.get_register32(unsafe { modrm.op_reg.reg_idx })
    }

    pub fn calc_memory_address(&self, modrm: &ModRM) -> u32 {
        match modrm.modu {
            0 => {
                match modrm.rm {
                    4 => {
                        println!("not implemented ModRM mod = 0, rm = 4");
                        process::exit(0);
                    },
                    5 => unsafe { modrm.disp.disp32 },
                    _ => self.get_register32(modrm.rm)
                }
            },
            1 => {
                if modrm.rm == 4 {
                        println!("not implemented ModRM mod = 1, rm = 4");
                        process::exit(0);
                } else {
                    unsafe { add_i2u_32(self.get_register32(modrm.rm), 
                                 modrm.disp.disp8 as i32) }
                }
            },
            _ => {
                println!("not implemented ModRM mod = 3");
                process::exit(0);
            }
        }
    }

    pub fn get_r8(&mut self, modrm: &ModRM) -> u8 {
        self.get_register8(unsafe { modrm.op_reg.reg_idx } as usize)
    }

    pub fn set_rm8(&mut self, modrm: &ModRM, val: u8) {
        if modrm.modu == 3 {
            self.set_register8(modrm.rm as i32, val);
        } else {
            let addr = self.calc_memory_address(modrm);
            self.set_memory8(addr, val as u32);
        }
    }
}
