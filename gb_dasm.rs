use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Bytes;

macro_rules! advance {
    ($stream:expr, $instruction:expr) => {{
        let stream: &mut Bytes<File> = $stream;
        let instruction: &str = $instruction;
        let bytes = instruction.matches("$").count() + 1;
        let result = match bytes {
            1 => format!("{}", instruction),
            2 => {
                let arg = format!("{:02X}", unwrap(stream.next()));
                instruction.replace("$0", &arg)
            }
            3 => {
                let (arg1, arg2) = get_next_two(stream);
                let (arg1, arg2) = (format!("{:02X}", arg1), format!("{:02X}", arg2));
                instruction.replace("$0", &arg1).replace("$1",&arg2)
            }
            _ => panic!("BYTES: {}", bytes),
        };
        (bytes, result)
    }}
}

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
fn dissasm(stream: &mut Bytes<File>) -> String {
    let mut output = String::from("");
    let mut total_bytes = 0;
    while let Some(val) = stream.next() {
        let byte = val.ok().unwrap();
        // NOTE When you see a jump instruction, e.g
        // JR NZ,$BLAH
        // The address to jump to will be CURRENT - (0xFE - $BLAH)
        // For example, JR NZ,$FE will jump constantly to itself, locking up
        let (bytes, result) = match byte {
            0x00 => advance!(stream, "NOP"),
            0x07 => advance!(stream, "RLCA"),
            0x17 => advance!(stream, "RLA"),
            0x27 => advance!(stream, "DAA"),
            0x37 => advance!(stream, "SCF"),
            0x10 => advance!(stream, "STOP"),
            0x76 => advance!(stream, "HALT"),
            0xCB => advance!(stream, "CB {}"), 
            0xF3 => advance!(stream, "DI"),
            0xF0 => advance!(stream, "LDH A,($0)"),
            0xE0 => advance!(stream, "LDH ($0),A"),
            0xEA => advance!(stream, "LD ($0),A"),
            0x08 => advance!(stream, "LD ($1$0),SP"),
            0xE2 => advance!(stream, "LD (C),A"),
            0xF2 => advance!(stream, "LD A,(C)"),
            0x2A => advance!(stream, "LD A,(HL+)"),
            0x3A => advance!(stream, "LD A,(HL-)"),
            0x22 => advance!(stream, "LD (HL+),A"),
            0x32 => advance!(stream, "LD (HL-),A"),
            0xC9 => advance!(stream, "RET"),
            0xCC => advance!(stream, "CALL Z,$1$0"),
            0xCD => advance!(stream, "CALL $1$0"),
            0xDC => advance!(stream, "CALL C,$1$0"),
            n if in_range(n, 0x1, 0x1, 0xC, 0xF) => {
                let (upper, _) = get_nibbles(n);
                let register = match upper {
                    0xC => "BC",
                    0xD => "DE",
                    0xE => "HL",
                    0xF => "AF",
                    _ => unreachable!(),
                };
                advance!(stream, &format!("POP {}", register))
            }
            n if in_range(n, 0x5, 0x5, 0xC, 0xF) => {
                let (upper, _) = get_nibbles(n);
                let register = match upper {
                    0xC => "BC",
                    0xD => "DE",
                    0xE => "HL",
                    0xF => "AF",
                    _ => unreachable!(),
                };
                advance!(stream, &format!("PUSH {}", register))
            }
            n if in_range(n, 0xE, 0xE, 0xC, 0xF) => {
                let (upper, _) = get_nibbles(n);
                let instruction = match upper {
                    0xC => "ADC A,",
                    0xD => "SBC A,",
                    0xE => "XOR ",
                    0xF => "CP ",
                    _ => unreachable!(),
                };
                advance!(stream, &format!("{}$0", instruction))
            }
            n if in_range(n, 0xA, 0xB, 0x0, 0x3) => {
                let (upper, lower) = get_nibbles(n);
                let register = match upper {
                    0x0 => "BC",
                    0x1 => "DE",
                    0x2 => "HL",
                    0x3 => "SP",
                    _ => unreachable!(),
                };
                match lower {
                    0xA => advance!(stream, &format!("LD A,({})", register)),
                    0xB => advance!(stream, &format!("DEC {}", register)),
                    _ => unreachable!(),
                }
            }
            n if in_range(n, 0xC, 0xE, 0x0, 0x3) => {
                let (upper, lower) = get_nibbles(n);
                let register = match upper {
                    0x0 => "C",
                    0x1 => "E",
                    0x2 => "L",
                    0x3 => "A",
                    _ => unreachable!(),
                };
                match lower {
                    0xC => advance!(stream, &format!("INC {}", register)),
                    0xD => advance!(stream, &format!("DEC {}", register)),
                    0xE => advance!(stream, &format!("LD {},$0", register)),
                    _ => unreachable!(),
                }
            }
            n if in_range(n, 0x0, 0x7, 0x8, 0xB) => {
                let (upper, lower) = get_nibbles(n);
                let instruction = match upper {
                    0x8 => "ADD A,",
                    0x9 => "SUB ",
                    0xA => "AND ",
                    0xB => "OR ",
                    _ => unreachable!(),
                };
                let register = match lower {
                    0x0 => "B",
                    0x1 => "C",
                    0x2 => "D",
                    0x3 => "E",
                    0x4 => "H",
                    0x5 => "L",
                    0x6 => "(HL)",
                    0x7 => "A",
                    _ => unreachable!(),
                };
                advance!(stream, &format!("{}{}", instruction, register))
            }
            n if in_range(n, 0x8, 0xF, 0x8, 0xB) => {
                let (upper, lower) = get_nibbles(n);
                let instruction = match upper {
                    0x8 => "ADC A,",
                    0x9 => "SBC A,",
                    0xA => "XOR ",
                    0xB => "CP ",
                    _ => unreachable!(),
                };
                let register = match lower {
                    0x8 => "B",
                    0x9 => "C",
                    0xA => "D",
                    0xB => "E",
                    0xC => "H",
                    0xD => "L",
                    0xE => "(HL)",
                    0xF => "A",
                    _ => unreachable!(),
                };
                advance!(stream, &format!("{}{}", instruction, register))
            }
            n if in_range(n, 0x2, 0x3, 0x0, 0x3) => {
                let (upper, lower) = get_nibbles(n);
                let to = match upper {
                    0x0 => "BC",
                    0x1 => "DE",
                    0x2 => "HL",
                    0x3 => "SP",
                    _ => unreachable!(),
                };
                match lower {
                    0x2 => advance!(stream, &format!("LD ({}),A", to)),
                    0x3 => advance!(stream, &format!("INC {}", to)),
                    _ => unreachable!(),
                }
            }
            n if in_range(n, 0x1, 0x1, 0x0, 0x3) => {
                let (upper, _) = get_nibbles(n);
                let to = match upper {
                    0x0 => "BC",
                    0x1 => "DE",
                    0x2 => "HL",
                    0x3 => "SP",
                    _ => unreachable!(),
                };
                advance!(stream, &format!("LD {},$1$0", to))
            }
            n if in_range(n, 0x8, 0xF, 0x4, 0x7) => {
                let (upper, lower) = get_nibbles(n);
                let to = match upper {
                    0x4 => "C",
                    0x5 => "E",
                    0x6 => "L",
                    0x7 => "A",
                    _ => unreachable!(),
                };
                let from = match lower {
                    0x8 => "B",
                    0x9 => "C",
                    0xA => "D",
                    0xB => "E",
                    0xC => "H",
                    0xD => "L",
                    0xE => "(HL)",
                    0xF => "A",
                    _ => unreachable!(),
                };
                advance!(stream, &format!("LD {},{}", to, from))
            } 
            n if in_range(n, 0x0, 0x7, 0x4, 0x7) => {
                let (upper, lower) = get_nibbles(n);
                let to = match upper {
                    4 => "B",
                    5 => "D",
                    6 => "H",
                    7 => "(HL)",
                    _ => unreachable!(),
                };
                let from = match lower {
                    0 => "B",
                    1 => "C",
                    2 => "D",
                    3 => "E",
                    4 => "H",
                    5 => "L",
                    6 => "(HL)",
                    7 => "A",
                    _ => unreachable!(), 
                };
                advance!(stream, &format!("LD {},{}", to, from))
            }
            n if in_range(n, 0x4, 0x6, 0x0, 0x3) => {
                let (upper, lower) = get_nibbles(n);
                let register = match upper {
                    0 => "B",
                    1 => "D",
                    2 => "H",
                    3 => "(HL)",
                    _ => unreachable!(),
                };
                match lower {
                    4 => advance!(stream, &format!("INC {}", register)),
                    5 => advance!(stream, &format!("DEC {}", register)),
                    6 => advance!(stream, &format!("LD {},$0", register)),
                    _ => unreachable!(),
                }
            }
            n => (1, format!("{:#04X}", n)),
        };
        output = format!("{}{:04X}: {}\t{:#04X}\n", output, total_bytes, result, byte);
        total_bytes += bytes;
    }
    output
}
fn in_range(n: u8, left: u8, right: u8, top: u8, bottom: u8) -> bool {
    let (upper, lower) = get_nibbles(n);
    lower >= left && lower <= right && upper >= top && upper <= bottom
}
fn get_nibbles(n: u8) -> (u8, u8) {
    let upper = n >> 4;
    let lower = (n << 4) >> 4;
    (upper, lower)
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
