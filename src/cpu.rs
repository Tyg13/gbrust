struct CPU {
    clock: Clock,
    reg8: [u8; 7],
    m: u8,
    t: u8,
    pc: u16,
    sp: u16,
    flags: Flags,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            clock: Clock { m: 0, t: 0 },
            reg8: [0; 7],
            m: 0,
            t: 0,
            pc: 0,
            sp: 0,
            flags: Flags {
                add: false,
                half_carry: false,
                carry: false,
            },
        }
    }
    /// USAGE: self.tick(time) where time is the number of m-cycles
    /// Used to set M and T registers to time taken for previous instruction
    /// t register advances 4 cycles for every 1 m-cycle
    pub fn tick(&mut self, time: u8) {
        self.m = time;
        self.t = time * 4;
    }
    /// USAGE: self.update_clock()
    /// Used to update clock to represent current time
    /// zeroes out m and t registers after adding them to clock.m and clock.t
    pub fn update_clock(&mut self) {
        self.clock.m += self.m;
        self.clock.t += self.t;
        self.m = 0;
        self.t = 0;
    }
    /// USAGE: self.and(R) where R is the register to be compared to A
    /// implements AND r instruction
    /// Returns logical AND of A and R and stores the result in A
    pub fn and(&mut self, register: R8) {
        let res = self.fetch8(register) | self.fetch8(R8::A);
        &mut self.set8(R8::A, res);
    }
    /// USAGE: self.or(R) where R is the register to be compared to A
    /// Implements OR r instruction
    /// Returns logical OR of A and R and stores the result in A
    pub fn or(&mut self, register: R8) {
        let res = self.fetch8(register) | self.fetch8(R8::A);
        self.set8(R8::A, res);
    }
    /// USAGE: self.fetch8(R) where R is the register to fetch
    /// Used internally to access the 8-bit register array
    /// Returns the value of register R, or the value of a 8-bit constant
    pub fn fetch8(&self, register: R8) -> u8 {
        match register {
            R8::A => self.reg8[0],
            R8::B => self.reg8[1], 
            R8::C => self.reg8[2],
            R8::D => self.reg8[3], 
            R8::E => self.reg8[4],
            R8::H => self.reg8[5],
            R8::L => self.reg8[6],
            R8::CONST(n) => n,
        }
    }
    /// USAGE:: self.fetch16(R) where R is the 16 bit register to fetch
    /// Used internally to access 16 bit registers or 2 8-bit registers as one 16-bit register.
    /// Returns the value of register R, or the value of a 16-bit constant
    pub fn fetch16(&self, register: R16) -> u16 {
        match register {
            R16::PC => self.pc,
            R16::SP => self.sp,
            R16::BC => u8s_to_u16(self.fetch8(R8::B), self.fetch8(R8::C)),
            R16::DE => u8s_to_u16(self.fetch8(R8::D), self.fetch8(R8::E)),
            R16::HL => u8s_to_u16(self.fetch8(R8::H), self.fetch8(R8::L)),
            R16::CONST(n) => n,
        }
    }
    /// USAGE: self.set8(R, N) where R is the 8-bit register to set and N is an 8-bit constant
    /// Used internally to set 8-bit registers
    pub fn set8(&mut self, register: R8, value: u8) {
        {
            let reg = match register {
                R8::A => &mut self.reg8[0],
                R8::B => &mut self.reg8[1],
                R8::C => &mut self.reg8[2],
                R8::D => &mut self.reg8[3],
                R8::E => &mut self.reg8[4],
                R8::H => &mut self.reg8[5],
                R8::L => &mut self.reg8[6],
                R8::CONST(_) => panic!("Tried to set 8-bit constant!"),
            };
            *reg = value;
        }
        self.tick(2);
    }
    /// USAGE: self.set16(R, N) where R is the 16-bit register to set and N is a 16-bit constant
    /// Used internally to set 16-bit registers or 2 8-bit registers
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
            R16::CONST(_) => panic!("Tried to set 16-bit constant!"),
        }
        self.tick(3);
    }
    /// USAGE: self.load(TO, FROM) where TO is the destination and FROM is the source
    /// Implements LD n,m instruction
    /// Only valid to load 8-bit registers to 8-bit registers
    pub fn load(&mut self, to: R8, from: R8) {
        if to != from {
            let val = self.fetch8(from);
            self.set8(to, val);
        }
        self.tick(1);
    }
    /// USAGE: self.add8(A, B) where A and B are 8-bit registers
    /// Implements 8-bit version of ADD n, m
    pub fn add8(&mut self, fst: R8, snd: R8) {
        use std::u8::MAX;
        self.flags.add = true;
        let (i, j) = (self.fetch8(fst), self.fetch8(snd));
        let res = (i as u16) + (j as u16);
        if res > (MAX as u16) {
            self.flags.carry = true;
            self.set8(fst, i.wrapping_add(j));
        } else {
            self.set8(fst, i + j);
        }
        self.flags.half_carry = detect_half_carry(i, j);
    }
}

struct Clock {
    m: u8,
    t: u8,
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
    CONST(u8)
}

use std::slice::Iter;
impl R8 {
    pub fn registers() -> Iter<'static, R8> {
        static REGISTERS: [R8; 7] = [R8::A, R8::B, R8::C, R8::D, R8::E, R8::H, R8::L];
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
    CONST(u16)
}

impl R16 {
    pub fn registers() -> Iter<'static, R16> {
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
                // Represent each number as an 8-bit string formatted 0bXXXXXXXX
                let (is, js) = (format!("{:#010b}", i), format!("{:#010b}", j));
                let mut half_carry = false;
                // Indices 2 .. 6 are the high nibble of each byte
                for k in 2..6 {
                    let (n, m) = (is.chars().nth(k).unwrap(), js.chars().nth(k).unwrap());
                    // If any bit is equal, there will be a carry in the high nibble
                    if n == '1' && m == '1' {
                        half_carry = true;
                    }
                }
                // Detect half carry should determine the same as the above, using bit shifts
                assert_eq!(detect_half_carry(i, j), half_carry);
            }
        }
    }
    // Checks for all u16s that splitting and recombining results in the same number
    #[test]
    fn u16_splitting_and_combining_rational() {
        use std::u16::MAX;
        for i in 0..MAX {
            let (high, low) = u16_to_u8s(i);
            assert_eq!(i, u8s_to_u16(high, low));
        }
    }
    // Checks for all pairs of u8s that combining and splitting results in the same numbers back
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
    // Checks that ticking properly updates the clock and zeroes out M and T
    #[test]
    fn cpu_can_tick() {
        let mut cpu = CPU::new();
        let (old_m, old_t) = (cpu.clock.m, cpu.clock.t);
        cpu.tick(1); // Set M register to 1 and T register to 2
        cpu.update_clock(); // add M and T registers to clock.m and clock.t, then zero out M and T
        assert_eq!((cpu.m, cpu.t), (0, 0));
        let (diff_m, diff_t) = (cpu.clock.m - old_m, cpu.clock.t - old_t);
        assert_eq!(diff_t, 4); // t clock increases 4 per tick
        assert_eq!(diff_m, 1); // m clock increases 1 per tick
    }
    // Checks that setting any 8-bit registers with any u8 value will return the same result when fetched
    #[test]
    fn cpu_can_fetch_and_set_8bit_registers() {
        use std::u8::MAX;
        let mut cpu = CPU::new();
        for reg in R8::registers() {
            for i in 0..MAX {
                cpu.set8(*reg, i);
                assert_eq!((2, 8), (cpu.m, cpu.t)); // LD r, n should take 2 m-cycles
                assert_eq!(cpu.fetch8(*reg), i);
            }
        }
    }
    // Checks that setting any 16-bit registers with any u16 will return the same result when
    // fetched
    #[test]
    fn cpu_can_fetch_and_set_16bit_registers() {
        use std::u16::MAX;
        let mut cpu = CPU::new();
        for reg in R16::registers() {
            for i in 0..MAX {
                cpu.set16(*reg, i);
                assert_eq!((3, 12), (cpu.m, cpu.t)); // LD rr, nn should take 3 m-cycles
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
                    R16::CONST(_) => {},
                }
            }
        }
    }
    // Checks that loading any register to any other register with some u8 will properly set it
    #[test]
    fn cpu_can_load_registers_to_registers() {
        use std::u8::MAX;
        let mut cpu = CPU::new();
        for from in R8::registers() {
            for to in R8::registers() {
                for i in 0..MAX {
                    for j in 0..MAX {
                        cpu.set8(*from, i);
                        cpu.set8(*to, j);
                        cpu.load(*to, *from);
                        assert_eq!(cpu.fetch8(*to), cpu.fetch8(*from));
                        assert_eq!((1, 4), (cpu.m, cpu.t)); // LD r, r should take 1 M cycle
                        if from == to {
                            // If from == to, then to == j, and NOT i, since to = from == to = j
                            assert_eq!(cpu.fetch8(*to), j);
                        } else {
                            // Otherwise, since to = from, and from = i, to == i
                            assert_eq!(cpu.fetch8(*to), i);
                        }
                    }
                }
            }
        }
    }
    // Checks that adding any two 8-bit registers results in the correct value stored in the first
    // register
    #[test]
    fn cpu_can_add_8_bit_registers() {
        use std::u8::MAX;
        let max = MAX as u16;
        let mut cpu = CPU::new();
        for i in 0..MAX {
            for j in 0..MAX {
                for reg1 in R8::registers() {
                    for reg2 in R8::registers() {
                        cpu.set8(*reg1, i);
                        cpu.set8(*reg2, j);
                        let i = cpu.fetch8(*reg1);
                        let j = cpu.fetch8(*reg2);
                        cpu.add8(*reg1, *reg2);
                        assert!(cpu.flags.add);
                        let res = (i as u16) + (j as u16);
                        if res >= max {
                            let (_, low) = u16_to_u8s(res);
                            assert!(cpu.flags.carry);
                            assert_eq!(cpu.fetch8(*reg1), low);
                        } else {
                            assert_eq!(cpu.fetch8(*reg1), i + j);
                        }
                        assert_eq!(cpu.flags.half_carry, detect_half_carry(i, j));
                    }
                }
            }
        }
    }
    #[test]
    fn cpu_can_add_constants_to_registers() {
        use std::u8::MAX as MAX8;
        use std::u16::MAX as MAX16;
        let mut cpu = CPU::new();
        for i in 0..MAX8 {
            for reg in R8::registers() {
                cpu.set8(*reg, 0);
                cpu.add8(*reg, R8::CONST(i));
                assert_eq!(i, cpu.fetch8(*reg));
            }
        } 
    }
}
