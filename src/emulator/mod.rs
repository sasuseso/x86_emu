use std::fmt;
extern crate byteorder;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

mod modrm;
mod instructions;

#[derive(Copy, Debug, Default, Clone)]
pub struct Regs32 {
    regs: [u32; 8]
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
        "EAX: 0x{:x}\n\
        ECX: 0x{:x}\n\
        EDX: 0x{:x}\n\
        EBX: 0x{:x}\n\
        ESP: 0x{:x}\n\
        EBP: 0x{:x}\n\
        ESI: 0x{:x}\n\
        EDI: 0x{:x}",
        self.regs[0], self.regs[1], self.regs[2], self.regs[3],
        self.regs[4], self.regs[5], self.regs[6], self.regs[7])
    }
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
}

fn add(a: u32, b: i32) -> u32 {
    if b.is_negative() {
        a - b.wrapping_abs() as u32
    } else {
        a + b as u32
    }
}
