use super::{Emulator, RegIdx, io_func::io_out8};

const BIOS_TO_TERMINAL: [u8; 8] = [30, 43, 32, 36, 31, 35, 33, 37];

fn put_string(s: String) {
    for c in s.chars() {
        io_out8(0x03f8, c as u8);
    }
}

impl Emulator {
    pub fn bios_video_teletype(&mut self) {
        let color: u8 = self.get_register8(RegIdx::bl()) & 0xf;
        let ch = self.get_register8(RegIdx::al());

        let terminal_color = BIOS_TO_TERMINAL[(color & 0x7) as usize];
        let bright = if (color & 0x8) != 0 { 1 } else { 0 };
        let buf = format!("\x1b[{};{}m{}\x1b[0m",
                          bright,
                          terminal_color,
                          (ch as char).to_string());
        put_string(buf);
    }

    pub fn bios_video(&mut self) {
        let f = self.get_register8(RegIdx::ah());
        match f {
            0x0e => { self.bios_video_teletype(); },
            _ => { println!("not implemented BIOS video function: {:#02x}", f); }
        }
    }
}
