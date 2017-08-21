struct CPU {
    clock: Clock,
    reg8: [u8; 9],
    pc: u16,
    sp: u16,
    flags: Flags,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            clock: Clock { m: 0, t: 0 },
            reg8: [0; 9],
            pc: 0,
            sp: 0,
            flags: Flags {
                add: false,
                half_carry: false,
                carry: false,
            },
        }
    }
    pub fn tick(&mut self, time: u8) {
        self.set8(R8::M, time);
        self.set8(R8::T, time * 4);
    }
    pub fn update_clock(&mut self) {
        let (m, t) = (self.fetch8(R8::M), self.fetch8(R8::T));
        self.clock.update(m, t);
        self.set8(R8::M, 0);
        self.set8(R8::T, 0);
    }
    pub fn and(&self, register: R8) -> u8 {
        self.fetch8(register) & self.fetch8(R8::A)
    }
    pub fn or(&self, register: R8) -> u8 {
        self.fetch8(register) | self.fetch8(R8::A)
    }
    pub fn fetch8(&self, register: R8) -> u8 {
        match register {
            R8::A => self.reg8[0],
            R8::B => self.reg8[1], 
            R8::C => self.reg8[2],
            R8::D => self.reg8[3], 
            R8::E => self.reg8[4],
            R8::H => self.reg8[5],
            R8::L => self.reg8[6],
            R8::M => self.reg8[7],
            R8::T => self.reg8[8],
        }
    }
    pub fn fetch16(&self, register: R16) -> u16 {
        match register {
            R16::PC => self.pc,
            R16::SP => self.sp,
            R16::BC => u8s_to_u16(self.fetch8(R8::B), self.fetch8(R8::C)),
            R16::DE => u8s_to_u16(self.fetch8(R8::D), self.fetch8(R8::E)),
            R16::HL => u8s_to_u16(self.fetch8(R8::H), self.fetch8(R8::L)),
        }
    }
    pub fn set8(&mut self, register: R8, value: u8) {
        let reg = match register {
            R8::A => &mut self.reg8[0],
            R8::B => &mut self.reg8[1],
            R8::C => &mut self.reg8[2],
            R8::D => &mut self.reg8[3],
            R8::E => &mut self.reg8[4],
            R8::H => &mut self.reg8[5],
            R8::L => &mut self.reg8[6],
            R8::M => &mut self.reg8[7],
            R8::T => &mut self.reg8[8],
        };
        *reg = value;
    }
    pub fn set16(&mut self, register: R16, value: u16) {
        let split = u16_to_u8s(value);
        match register {
            R16::PC => self.pc = value,
            R16::SP => self.sp = value,
            R16::BC => {
                self.set8(R8::B, split.0);
                self.set8(R8::C, split.1);
            }
            R16::DE => {
                self.set8(R8::D, split.0);
                self.set8(R8::E, split.1);
            }
            R16::HL => {
                self.set8(R8::H, split.0);
                self.set8(R8::L, split.1);
            }
        }
    }
    pub fn load(&mut self, to: R8, from: R8) {
        let val = self.fetch8(from);
        self.set8(to, val);
    }
    pub fn add(&mut self, fst: R8, snd: R8) {
        use std::u8::MAX;
        self.flags.add = true;
        let (i, j) = (self.fetch8(fst), self.fetch8(snd));
        let res = (i as u16) + (j as u16);
        if res > (MAX as u16) {
            self.flags.carry = true;
            self.set8(R8::A, i.wrapping_add(j));
        } else {
            self.set8(R8::A, i + j);
        }
        if detect_half_carry(i, j) {
            self.flags.half_carry = true;
        }
    }
}

#[derive(Clone)]
struct Clock {
    m: u8,
    t: u8,
}

impl Clock {
    pub fn update(&mut self, m: u8, t: u8) {
        self.m += m;
        self.t += t;
    }
}

struct Flags {
    add: bool,
    carry: bool,
    half_carry: bool,
}

pub fn u8s_to_u16(high: u8, low: u8) -> u16 {
    let high = (high as u16) << 8;
    high + (low as u16)
}

pub fn u16_to_u8s(wbyte: u16) -> (u8, u8) {
    ((wbyte >> 8) as u8, wbyte as u8)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    M,
    T,
}

use std::slice::Iter;
impl R8 {
    pub fn values() -> Iter<'static, R8> {
        static REGISTERS: [R8; 9] = [
            R8::A,
            R8::B,
            R8::C,
            R8::D,
            R8::E,
            R8::H,
            R8::L,
            R8::M,
            R8::T,
        ];
        REGISTERS.into_iter()
    }
}

#[derive(Clone, Copy)]
pub enum R16 {
    PC,
    SP,
    BC,
    DE,
    HL,
}

impl R16 {
    pub fn values() -> Iter<'static, R16> {
        static REGISTERS: [R16; 5] = [R16::PC, R16::SP, R16::BC, R16::DE, R16::HL];
        REGISTERS.into_iter()
    }
}

fn detect_half_carry(fst: u8, snd: u8) -> bool {
    let (fst, snd) = ((fst >> 4), (snd >> 4));
    (fst & snd) != 0
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn can_detect_half_carry() {
        use std::u8::MAX;
        for i in 0..MAX {
            for j in 0..MAX {
                let (is, js) = (format!("{:#010b}", i), format!("{:#010b}", j));
                let mut half_carry = false;
                for k in 2..6 {
                    let (n, m) = (is.chars().nth(k).unwrap(), js.chars().nth(k).unwrap());
                    if n == '1' && m == '1' {
                        half_carry = true;
                    }
                }
                if half_carry {
                    assert!(detect_half_carry(i, j));
                } else {
                    assert!(!detect_half_carry(i, j));
                }
            }
        }
    }
    fn u16_splitting_and_combining_rational() {
        use std::u16::MAX;
        for i in 0..MAX {
            let (high, low) = u16_to_u8s(i);
            assert_eq!(i, u8s_to_u16(high, low));
        }
    }
    #[test]
    fn u8_combining_and_splitting_rational() {
        use std::u8::MAX;
        for i in 0..MAX {
            for j in 0..MAX {
                let combined = u8s_to_u16(i, j);
                assert_eq!((i, j), u16_to_u8s(combined));
            }
        }
    }
    #[test]
    fn cpu_can_tick() {
        let mut cpu = CPU::new();
        cpu.tick(1);
        let old_clock = cpu.clock.clone();
        cpu.update_clock();
        let new_clock = cpu.clock;
        let diff_m = new_clock.m - old_clock.m;
        let diff_t = new_clock.t - old_clock.t;
        assert_eq!(diff_t, 4); // t clock increases 4 per tick
        assert_eq!(diff_m, 1); // m clock increases 1 per tick
    }
    #[test]
    fn cpu_can_fetch_and_set_8bit_registers() {
        use std::u8::MAX;
        let mut cpu = CPU::new();
        for reg in R8::values() {
            for i in 0..MAX {
                cpu.set8(*reg, i);
                assert_eq!(cpu.fetch8(*reg), i);
            }
        }
    }
    #[test]
    fn cpu_can_fetch_and_set_16bit_registers() {
        use std::u16::MAX;
        let mut cpu = CPU::new();
        for reg in R16::values() {
            for i in 0..MAX {
                cpu.set16(*reg, i);
                assert_eq!(cpu.fetch16(*reg), i);
                match *reg {
                    R16::PC | R16::SP => {
                        continue;
                    }
                    R16::BC => {
                        let (high, low) = u16_to_u8s(i);
                        assert_eq!(cpu.fetch8(R8::B), high);
                        assert_eq!(cpu.fetch8(R8::C), low);
                    }
                    R16::DE => {
                        let (high, low) = u16_to_u8s(i);
                        assert_eq!(cpu.fetch8(R8::D), high);
                        assert_eq!(cpu.fetch8(R8::E), low);
                    }
                    R16::HL => {
                        let (high, low) = u16_to_u8s(i);
                        assert_eq!(cpu.fetch8(R8::H), high);
                        assert_eq!(cpu.fetch8(R8::L), low);
                    }
                }
            }
        }
    }
    #[test]
    fn cpu_can_load_registers_to_registers() {
        use std::u8::MAX;
        let mut cpu = CPU::new();
        for from in R8::values() {
            for to in R8::values() {
                if from == to {
                    continue;
                }
                for i in 0..MAX {
                    for j in 0..MAX {
                        cpu.set8(*from, i);
                        cpu.set8(*to, j);
                        cpu.load(*to, *from);
                        assert_eq!(cpu.fetch8(*to), i);
                        assert_eq!(cpu.fetch8(*to), cpu.fetch8(*from));
                    }
                }
            }
        }
    }
    #[test]
    fn cpu_can_add_registers() {
        use std::u8::MAX;
        let max = MAX as u16;
        let mut cpu = CPU::new();
        for i in 0..MAX {
            for j in 0..MAX {
                for reg1 in R8::values() {
                    for reg2 in R8::values() {
                        cpu.set8(*reg1, i);
                        cpu.set8(*reg2, j);
                        let i = cpu.fetch8(*reg1);
                        let j = cpu.fetch8(*reg2);
                        cpu.add(*reg1, *reg2);
                        assert!(cpu.flags.add);
                        let res = (i as u16) + (j as u16);
                        if res >= max {
                            let (high, low) = u16_to_u8s(res);
                            assert!(cpu.flags.carry);
                            assert_eq!(cpu.fetch8(R8::A), low); 
                        } else {
                            assert_eq!(cpu.fetch8(R8::A), i + j);
                        }
                        if detect_half_carry(i, j) {
                            assert!(cpu.flags.half_carry);
                        }
                    }
                }
            }
        }
    }
}
