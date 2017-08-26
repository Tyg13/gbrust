use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Bytes;

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        fail(Failure::NotEnoughArgs);
    }
    if !args[1].contains(".bin") {
        fail(Failure::ArgumentNotBinary);
    }
    let filename = &args[1];
    let file = File::open(filename);
    let handle = match file {
        Ok(result) => result,
        Err(_) => fail(Failure::FileReadError(filename)),
    };
    println!("{}", dissasm(&mut handle.bytes()));
}
fn dissasm(bytes: &mut Bytes<File>) -> String {
    let mut output = String::from("");
    let mut line = 0;
    while let Some(val) = bytes.next() {
        let result = match val.ok().unwrap() {
            0x20 => {
                let arg = unwrap(bytes.next());
                format!("JR NZ, {:02X}", arg)
            }
            0x21 => {
                let (arg1, arg2) = get_next_two(bytes);
                format!("LD HL, ${:02X}{:02X}", arg2, arg1)
            }
            0x31 => {
                let (arg1, arg2) = get_next_two(bytes);
                format!("LD SP, ${:02X}{:02X}", arg2, arg1)
            }
            0x32 => format!("LD (HL-), A"),
            0xAF => format!("XOR A"),
            0xCB => format!("CB {:#2X}", bytes.next().unwrap().ok().unwrap()), 
            n => format!("{:#2X}", n),
        };
        output = format!("{}\n{:04X}: {}", output, line, result);
        line += 1;
    }
    output
}
fn get_next_two(bytes: &mut Bytes<File>) -> (u8, u8) {
    (unwrap(bytes.next()), unwrap(bytes.next()))
}
fn unwrap(bytes: std::option::Option<std::result::Result<u8, std::io::Error>>) -> u8 {
    bytes.unwrap().ok().unwrap()
}

fn fail(error: Failure) -> ! {
    use Failure::*;
    let err = match error {
        NotEnoughArgs => String::from("Not enough arguments"),
        ArgumentNotBinary => String::from("Input must be a binary file .bin"),
        FileReadError(file) => format!("Not able to read input file {}", file),
    };
    println!("ERR: {}\n", err);
    println!("Usage: gd_dasm FILE");
    println!("Where FILE is any binary GB file");
    std::process::exit(1);
}

enum Failure<'a> {
    NotEnoughArgs,
    ArgumentNotBinary,
    FileReadError(&'a str),
}
