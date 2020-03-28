use std::env;
use std::process;
use std::io::prelude::*;
use std::fs::File;
use x86_emu::emulator;

const MEM_SIZE: usize = 1024 * 1024;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    
    let mut quiet_flag = false;

    if args[1] == "quiet" {
        quiet_flag = true;
        args.remove(1);
    }

    if args.len() != 2 {
        println!("usage: px86 filename");
        process::exit(1);
    }

    let mut emu = emulator::Emulator::new(MEM_SIZE, 0x7c00, 0x7c00);

    let f = match File::open(args[1].to_string()) {
        Ok(f) => f,
        Err(_) => {
            println!("cannot open file.");
            process::exit(1);
        }
    };

    let data: Result<Vec<_>, _> = f.bytes().collect();

    emu.memory.splice(0x7c00.., data.map_err(|_e| {
            println!("memory loading error.");
            process::exit(1);
    }).unwrap()); 

    let instructions = emu.init_instructions();

    println!();
    while emu.eip < MEM_SIZE as u32 {
        let code = emu.get_code8(0);

        if let Some(inst) = instructions[code as usize] {
            if !quiet_flag {
                println!("EIP: {:#06x}, ESP: {:#06x}, Code: {:#02x}",
                         emu.eip, emu.registers.regs[4], code);
            }

            inst(&mut emu);

            if emu.eip == 0x00 {
                println!("\n\n--------End of Program--------\n");
                break;
            }
        } else {
            println!("\n\nNot Implemented Instruction: 0x{:x}\n", code);
            break;
        }
    }

    println!("{}", emu.registers);
    println!("EIP: {:#010x}", emu.eip);

    for  (a, m) in emu.memory[0x7c00..].iter().enumerate() {
        print!("{:02x} ", m);
        if (a+1) % 8 == 0 {
            println!();
        }
    }
    println!();

    process::exit(0);
}
