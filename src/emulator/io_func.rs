use std::io;
use std::io::{Write, Read};

pub fn io_in8(addr: u16) -> u8 {
    match addr {
        0x03f8 => {
            io::stdin().bytes().nth(0).unwrap().unwrap()
        },
        _ => 0
    }
}

pub fn io_out8(addr: u16, val: u8) {
    match addr {
        0x03f8 => {
            print!("{}", val as char);
            io::stdout().flush().unwrap();
        },
        _ => ()
    }
}
