use std::env;
use std::fs::File;
use std::io::prelude::*;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        fail(Failure::NotEnoughArgs);
    }
    if !args[1].contains(".bin") {
        fail(Failure::ArgumentNotBinary);
    }
    let file = File::open(&args[1]).unwrap();
    let mut handle = file.bytes();
    loop {
        if let Some(val) = handle.next() {
            let n = val.unwrap();
            decode(n);
        } else {
            break;
        }
    }
}

fn decode(opcode: u8) {
    match opcode {
        0x0C => print!("\nINC C\n"),
        0x0E => print!("\nLD C, "),
        0xE2 => print!("\nLD (C), A\n"),
        0x3E => print!("\nLD A, "),
        0x32 => print!("\nLD (HL-), A\n"),
        0x31 => print!("\nLD SP, "),
        0x21 => print!("\nLD HL, "),
        0x20 => print!("\nJR NZ, "),
        0xAF => print!("\nXOR A "),
        n => print!("{:02X} ", n),
    }
}

fn fail(error: Failure) {
    use Failure::*;
    let err = match error {
        NotEnoughArgs => "Not enough arguments",
        ArgumentNotBinary => "Input must be a binary file .bin",
    };
    println!("ERR: {}\n", err);
    println!("Usage: gd_dasm FILE");
    println!("Where FILE is any binary GB file");
    std::process::exit(1);
}

enum Failure {
    NotEnoughArgs,
    ArgumentNotBinary,
}
