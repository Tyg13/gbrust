use std::env;
use std::cmp;
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
    dissasm(&mut handle.bytes());
}

struct Disassembly<'a> {
    stream: &'a mut Bytes<File>,
    bytes: usize,
    data_ends: usize,
    calls: Vec<usize>,
    jumps_forward: Vec<usize>,
}

impl<'a> Disassembly<'a> {
    pub fn new(stream: &'a mut Bytes<File>) -> Self {
        Disassembly {
            stream: stream,
            bytes: 0,
            data_ends: 0,
            calls: Vec::new(),
            jumps_forward: Vec::new(),
        }
    }
    pub fn next(&mut self) -> std::option::Option<std::result::Result<u8, std::io::Error>> {
        self.stream.next()
    }
    fn check_if_data_segment(&mut self) {
        let n = self.jumps_forward
            .clone()
            .into_iter()
            .filter(|n| n > &self.bytes)
            .min()
            .unwrap_or(0);
        let m = self.calls
            .clone()
            .into_iter()
            .filter(|n| n > &self.bytes)
            .min()
            .unwrap_or(0);
        // The data will end at the minimal jumped address
        // If there is no minimal jumped address, data_end will be
        // 0 so that the data check always fails.
        self.data_ends = if n == 0 {
            m
        } else if m == 0 {
            n
        } else {
            cmp::min(n, m)
        };
    }
    pub fn parse_line(&mut self, instruction: String, byte: u8) {
        let is_data = self.bytes < self.data_ends;
        if !is_data {
            self.data_ends = 0;
        }
        let is_call = instruction.contains("CALL");
        let is_absolute_jump = instruction.contains("JP");
        let is_relative_jump = instruction.contains("JR");
        let number_of_args = instruction.matches("#").count();
        if is_data {
            let mut line = String::new();
            let mut data_bytes = 0;
            // Break up data in 16 byte chunks
            for n in 0..16 {
                if n == 0 {
                    line.push_str(&format!(".db ${:02X}", byte));
                } else {
                    line.push_str(&format!("${:02X}", self.unwrap_next()));
                }
                data_bytes += 1;
                if (self.bytes + data_bytes) >= self.data_ends {
                    break;
                } else if n != 15 {
                    line.push_str(", ");
                }
            }
            println!("{:04X}: {}", self.bytes, line);
            self.bytes += data_bytes;
        } else {
            let line = match number_of_args {
                0 => {
                    // Returns clear the calls that have already been passed
                    if instruction == "RET" {
                        let mut new_calls = Vec::new();
                        for n in self.calls.clone().into_iter() {
                            if n > self.bytes {
                                new_calls.push(n);
                            }
                        }
                        self.calls = new_calls;
                        self.check_if_data_segment();
                    }
                    format!("{}", instruction)
                }
                1 => {
                    let arg = self.unwrap_next();
                    if is_relative_jump {
                        // Interpret byte as signed, then add 2 since 0xFE = -2,
                        // and 0xFE corresponds to a jump offset of 0
                        let offset = (arg as i8) + 2;
                        let address = (self.bytes as i16) + (offset as i16);
                        if offset > 0 {
                            // Add forward conditional jumps
                            if conditional(&instruction) {
                                self.jumps_forward.push(address as usize);
                            }
                        } else if offset < 0 {
                            // If we see a non-conditional forward jump, we may be starting
                            // a data segment
                            if !conditional(&instruction) {
                                self.check_if_data_segment();
                            }
                        }
                        instruction.replace("#0", &format!("${:04X}", address))
                    } else {
                        let _arg = format!("${:02X}", arg);
                        instruction.replace("#0", &_arg)
                    }
                }
                2 => {
                    let (arg1, arg2) = self.unwrap_next_two();
                    if is_call {
                        let address = combine(arg2, arg1) as usize;
                        if address > self.bytes {
                            self.calls.push(address);
                        };
                    }
                    if is_absolute_jump && !conditional(&instruction) {
                        self.data_ends = combine(arg2, arg1) as usize;
                    }
                    let (arg1, arg2) = (format!("{:02X}", arg1), format!("${:02X}", arg2));
                    instruction.replace("#0", &arg1).replace("#1", &arg2)
                }
                _ => panic!("# OF ARGS: {}", number_of_args),
            };
            let bytes = number_of_args + 1;
            println!("{:04X}: {: <15}x{:02X}", self.bytes, line, byte);
            self.bytes += bytes;
        }
    }
    pub fn unwrap_next(&mut self) -> u8 {
        unwrap(self.stream.next())
    }
    pub fn unwrap_next_two(&mut self) -> (u8, u8) {
        get_next_two(&mut self.stream)
    }
}

fn conditional(instruction: &str) -> bool {
    for c in instruction.chars() {
        match c {
            'N' | 'Z' | 'C' => {
                return true;
            }
            _ => (),
        }
    }
    false
}

fn combine(a: u8, b: u8) -> u16 {
    ((a as u16) << 8) + (b as u16)
}

fn dissasm(bytes: &mut Bytes<File>) {
    let disassembly = &mut Disassembly::new(bytes);
    while let Some(val) = disassembly.next() {
        let byte = val.ok().unwrap();
        let instruction: String = match byte {
            0x00 => String::from("NOP"),
            0x07 => String::from("RLCA"),
            0x08 => String::from("LD (#1#0),SP"),
            0x0E => String::from("LDH A,(#0)"),
            0x0F => String::from("LDH (#0),A"),
            0x10 => String::from("STOP"),
            0x17 => String::from("RLA"),
            0x20 => String::from("JR NZ,#0"),
            0x22 => String::from("LD (HL+),A"),
            0x27 => String::from("DAA"),
            0x2A => String::from("LD A,(HL+)"),
            0x30 => String::from("JR NC,#0"),
            0x32 => String::from("LD (HL-),A"),
            0x37 => String::from("SCF"),
            0x3A => String::from("LD A,(HL-)"),
            0x76 => String::from("HALT"),
            0xC3 => String::from("JP #1#0"),
            0xC9 => String::from("RET"),
            0xCA => String::from("JP Z,#1#0"),
            0xCC => String::from("CALL Z,#1#0"),
            0xCB => String::from("CB #0"), 
            0xCD => String::from("CALL #1#0"),
            0xDA => String::from("JP C,#1#0"),
            0xDC => String::from("CALL C,#1#0"),
            0xE0 => String::from("LDH (#0),A"),
            0xE2 => String::from("LD (C),A"),
            0xEA => String::from("LD (#1#0),A"),
            0xF0 => String::from("LDH A,(#0)"),
            0xF2 => String::from("LD A,(C)"),
            0xF3 => String::from("DI"),
            0xFB => String::from("EI"),
            n => {
                let (upper, lower) = get_nibbles(n);
                if in_range(n, 0x1, 0x1, 0xC, 0xF) {
                    let register = match upper {
                        0xC => "BC",
                        0xD => "DE",
                        0xE => "HL",
                        0xF => "AF",
                        _ => unreachable!(),
                    };
                    format!("POP {}", register)
                } else if in_range(n, 0xF, 0xF, 0xC, 0xF) {
                    let op = match upper {
                        0xC => "$00",
                        0xD => "$10",
                        0xE => "$20",
                        0xF => "$30",
                        _ => unreachable!(),
                    };
                    format!("RST {}", op)
                } else if in_range(n, 0x5, 0x5, 0xC, 0xF) {
                    let register = match upper {
                        0xC => "BC",
                        0xD => "DE",
                        0xE => "HL",
                        0xF => "AF",
                        _ => unreachable!(),
                    };
                    format!("PUSH {}", register)
                } else if in_range(n, 0xE, 0xE, 0xC, 0xF) {
                    let instruction = match upper {
                        0xC => "ADC A,",
                        0xD => "SBC A,",
                        0xE => "XOR ",
                        0xF => "CP ",
                        _ => unreachable!(),
                    };
                    format!("{}$0", instruction)
                } else if in_range(n, 0xA, 0xB, 0x0, 0x3) {
                    let register = match upper {
                        0x0 => "BC",
                        0x1 => "DE",
                        0x2 => "HL",
                        0x3 => "SP",
                        _ => unreachable!(),
                    };
                    match lower {
                        0xA => format!("LD A,({})", register),
                        0xB => format!("DEC {}", register),
                        _ => unreachable!(),
                    }
                } else if in_range(n, 0xC, 0xE, 0x0, 0x3) {
                    let register = match upper {
                        0x0 => "C",
                        0x1 => "E",
                        0x2 => "L",
                        0x3 => "A",
                        _ => unreachable!(),
                    };
                    match lower {
                        0xC => format!("INC {}", register),
                        0xD => format!("DEC {}", register),
                        0xE => format!("LD {},#0", register),
                        _ => unreachable!(),
                    }
                } else if in_range(n, 0x0, 0x7, 0x8, 0xB) {
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
                    format!("{}{}", instruction, register)
                } else if in_range(n, 0x8, 0xF, 0x8, 0xB) {
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
                    format!("{}{}", instruction, register)
                } else if in_range(n, 0x8, 0x8, 0x1, 0x3) {
                    let cond = match upper {
                        0x1 => "",
                        0x2 => "Z,",
                        0x3 => "C,",
                        _ => unreachable!(),
                    };
                    format!("JR {}#0", cond)
                } else if in_range(n, 0x2, 0x3, 0x0, 0x3) {
                    let to = match upper {
                        0x0 => "BC",
                        0x1 => "DE",
                        0x2 => "HL",
                        0x3 => "SP",
                        _ => unreachable!(),
                    };
                    match lower {
                        0x2 => format!("LD ({}),A", to),
                        0x3 => format!("INC {}", to),
                        _ => unreachable!(),
                    }
                } else if in_range(n, 0x1, 0x1, 0x0, 0x3) {
                    let to = match upper {
                        0x0 => "BC",
                        0x1 => "DE",
                        0x2 => "HL",
                        0x3 => "SP",
                        _ => unreachable!(),
                    };
                    format!("LD {},#1#0", to)
                } else if in_range(n, 0x8, 0xF, 0x4, 0x7) {
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
                    format!("LD {},{}", to, from)
                } else if in_range(n, 0x6, 0x6, 0xC, 0xF) {
                    let instruction = match upper {
                        0xC => "ADD a,",
                        0xD => "SUB ",
                        0xE => "AND ",
                        0xF => "OR ",
                        _ => unreachable!(),
                    };
                    format!("{}#0", instruction)
                } else if in_range(n, 0x0, 0x7, 0x4, 0x7) {
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
                    format!("LD {},{}", to, from)
                } else if in_range(n, 0x4, 0x6, 0x0, 0x3) {
                    let register = match upper {
                        0 => "B",
                        1 => "D",
                        2 => "H",
                        3 => "(HL)",
                        _ => unreachable!(),
                    };
                    match lower {
                        4 => format!("INC {}", register),
                        5 => format!("DEC {}", register),
                        6 => format!("LD {},#0", register),
                        _ => unreachable!(),
                    }
                } else {
                    format!("{:#04X}", n)
                }
            }
        };
        disassembly.parse_line(instruction, byte);
    }
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
