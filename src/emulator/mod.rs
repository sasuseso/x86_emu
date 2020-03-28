use std::fmt;
extern crate byteorder;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

mod modrm;
mod instructions;
mod io_func;
mod bios;

#[derive(Copy, Debug, Default, Clone)]
pub struct Regs32 {
    pub regs: [u32; 8]
}

impl Regs32 {
    pub fn new(regs: [u32; 8]) -> Regs32 {
        Regs32 {
            regs: regs
        }
    }
}

impl fmt::Display for Regs32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
        "EAX: {:#010x}\n\
        ECX: {:#010x}\n\
        EDX: {:#010x}\n\
        EBX: {:#010x}\n\
        ESP: {:#010x}\n\
        EBP: {:#010x}\n\
        ESI: {:#010x}\n\
        EDI: {:#010x}",
        self.regs[0], self.regs[1], self.regs[2], self.regs[3],
        self.regs[4], self.regs[5], self.regs[6], self.regs[7])
    }
}

enum RegIdx {
    Eax = 0,
    Ecx = 1,
    Edx = 2,
    Ebx = 3,
    Esp = 4,
    Ebp = 5,
    Esi = 6,
    Edi = 7,
}

impl RegIdx {
    pub fn al() -> usize { Self::Eax as usize }
    pub fn cl() -> usize { Self::Ecx as usize }
    pub fn bl() -> usize { Self::Ebx as usize }
    pub fn dl() -> usize { Self::Edx as usize }
    pub fn ah() -> usize { Self::al() + 4 }
    pub fn ch() -> usize { Self::cl() + 4 }
    pub fn dh() -> usize { Self::dl() + 4 }
    pub fn bh() -> usize { Self::bl() + 4 }
}

enum Eflags {
    Carry,
    Zero,
    Sign,
    Overflow
}

#[derive(Debug, Default, Clone)]
pub struct Emulator {
    pub registers: Regs32,
    pub eflags: u32,
    pub memory: Vec<u8>,
    pub eip: u32
}

impl Emulator {
    pub fn new(size: usize, eip: u32, esp: u32) -> Emulator {
        Emulator {
            registers: Regs32::new([0, 0, 0, 0, esp, 0, 0, 0]),
            eflags: 0,
            memory: vec![0; size],
            eip: eip,
        }
    }

    pub fn get_signed_code8(&self, idx: usize) -> i8 {
        self.memory[self.eip as usize + idx] as i8
    }

    pub fn get_code8(&self, idx: usize) -> u8 {
        self.memory[self.eip as usize + idx]
    }

    pub fn get_code32(&self, idx: usize) -> u32 {
        let idx = idx + self.eip as usize;
        let mut m = &self.memory[idx..idx+4];
        m.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_signed_code32(&self, idx: usize) -> i32 {
        let idx = idx + self.eip as usize;
        let mut m = &self.memory[idx..idx+4];
        m.read_u32::<LittleEndian>().unwrap() as i32
    }

    pub fn set_memory8(&mut self, addr: u32, val: u32) {
        self.memory[addr as usize] = (val & 0xff) as u8;
    }

    pub fn set_memory32(&mut self, addr: u32, val: u32) {
        let mut valm = vec![];
        valm.write_u32::<LittleEndian>(val).unwrap();
        let idx = addr as usize;
        self.memory.splice(idx..idx+4, valm);
    }
    
    pub fn get_register32(&self, idx: u8) -> u32 {
        self.registers.regs[idx as usize]
    }

    pub fn set_register32(&mut self, idx: u8, val: u32) {
        self.registers.regs[idx as usize] = val;
    }

    pub fn get_memory8(&self, addr: u32) -> u32 {
        self.memory[addr as usize] as u32
    }

    pub fn get_memory32(&self, addr: u32) -> u32 {
        (&(self.memory[addr as usize..]))
            .read_u32::<LittleEndian>().unwrap()
    }

    pub fn push32(&mut self, val: u32) {
        let addr = self.get_register32(4) - 4; // registers.regs[4] = ESP
        self.set_register32(4, addr);
        self.set_memory32(addr, val);
    }

    pub fn pop32(&mut self) -> u32 {
        let addr = self.get_register32(4);
        let ret = self.get_memory32(addr);
        self.set_register32(4, addr + 4);
        ret
    }

    fn update_eflags_sub(&mut self, v1: u32, v2: u32, res: u64) {
        let sign1 = v1 >> 31;
        let sign2 = v2 >> 31;
        let signr = (res >> 31) & 1;

        self.set_eflags(Eflags::Carry, (res >> 32) != 0);
        self.set_eflags(Eflags::Zero, res == 0);
        self.set_eflags(Eflags::Sign, signr != 0);
        self.set_eflags(Eflags::Overflow, sign1 != sign2 && sign1 != signr as u32);
    }

    fn set_eflags(&mut self, which_bit: Eflags, new_flag: bool) {
        let flag = match which_bit {
            Eflags::Carry => 1,
            Eflags::Zero => 1 << 6,
            Eflags::Sign => 1 << 7,
            Eflags::Overflow => 1 << 11,
        };
        if new_flag {
            self.eflags |= flag;
        } else {
            self.eflags &= !flag;
        }
    }

    fn check_eflag(&self, eflag: Eflags) -> bool {
        match eflag {
            Eflags::Carry => (self.eflags & 1) != 0,
            Eflags::Zero => (self.eflags & (1 << 6)) != 0,
            Eflags::Sign => (self.eflags & (1 << 7)) != 0,
            Eflags::Overflow => (self.eflags & (1 << 11)) != 0,
        }
    }

    fn get_register8(&mut self, idx: usize) -> u8 {
        if idx < 4 {
            (self.registers.regs[idx] & 0xff) as u8
        } else {
            ((self.registers.regs[idx - 4] >> 8) & 0xff) as u8
        }
    }

    fn set_register8(&mut self, idx: i32, val: u8) {
        if idx < 4 {
            let r = self.registers.regs[idx as usize] & 0xffffff00;
            self.registers.regs[idx as usize] = r | (val as u32);
        } else {
            let r = self.registers.regs[idx as usize - 4] & 0xffff00ff;
            self.registers.regs[idx as usize - 4] = r | ((val as u32) << 8);
        }
    }
}

fn add_i2u_32(a: u32, b: i32) -> u32 {
    if b.is_negative() {
        a - b.wrapping_abs() as u32
    } else {
        a + b as u32
    }
}
