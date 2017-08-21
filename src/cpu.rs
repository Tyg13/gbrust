struct CPU {
    clock: Clock,
    reg8: [u8; 9],
    pc: u16,
    sp: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            clock: Clock { m: 0, t: 0 },
            reg8: [0; 9],
            pc: 0,
            sp: 0,
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
        let split = u16_to_u8_tuple(value);
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

pub fn u8s_to_u16(high: u8, low: u8) -> u16 {
    let high = (high as u16) << 8;
    high + (low as u16)
}

pub fn u16_to_u8_tuple(wbyte: u16) -> (u8, u8) {
    ((wbyte >> 8) as u8, wbyte as u8)
}

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

pub enum R16 {
    PC,
    SP,
    BC,
    DE,
    HL,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn u16_splitting_and_combining_rational() {
        use std::u16::MAX;
        for i in 0..MAX {
            let (high, low) = u16_to_u8_tuple(i);
            assert_eq!(i, u8s_to_u16(high, low));
        }
    }
    #[test]
    fn u8_combining_and_splitting_rational() {
        use std::u8::MAX;
        for i in 0..MAX {
            for j in 0..MAX {
                let combined = u8s_to_u16(i, j);
                assert_eq!((i, j), u16_to_u8_tuple(combined));
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
        let mut cpu = CPU::new();
        cpu.set8(R8::A, 10);
        assert_eq!(cpu.fetch8(R8::A), 10);
    }
    #[test]
    fn cpu_can_fetch_and_set_16bit_registers() {
        let mut cpu1 = CPU::new();
        cpu1.set8(R8::B, 1);
        cpu1.set8(R8::C, 1);
        let mut cpu2 = CPU::new();
        cpu2.set16(R16::BC, u8s_to_u16(1, 1));
        assert_eq!(cpu1.fetch16(R16::BC), cpu2.fetch16(R16::BC));
        assert_eq!(0b100000001, cpu2.fetch16(R16::BC));
    }
    #[test]
    fn cpu_can_load_registers_to_registers() {
        let mut cpu = CPU::new();
        cpu.set8(R8::A, 1);
        cpu.load(R8::B, R8::A);
        assert_eq!(cpu.fetch8(R8::B), 1);
    }
    #[allow(non_snake_case)]
    #[test]
    fn cpu_can_OR_and_AND() {
        let mut cpu = CPU::new();
        for i in 0..1 {
            for j in 0..1 {
                cpu.set8(R8::A, i);
                cpu.set8(R8::B, j);
                assert_eq!(cpu.or(R8::B), i | j);
                assert_eq!(cpu.and(R8::B), i & j);
            }
        }
    }
}
