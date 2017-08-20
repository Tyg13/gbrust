struct CPU {
    clock: Clock,
    registers: Registers,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            clock: Clock { m: 0, t: 0 },
            registers: Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                h: 0,
                l: 0,
                pc: 0,
                sp: 0,
                m: 0,
                t: 0,
            },
        }
    }
    pub fn tick(&mut self, time: u8) {
        self.registers.m = time;
        self.registers.t = time * 4;
    }
    pub fn update_clock(&mut self) {
        self.clock.update(self.registers.m, self.registers.t);
        self.registers.m = 0;
        self.registers.t = 0;
    }
    pub fn and(&self, register: R8) -> u8 {
        self.fetch8(register) & self.registers.a
    }
    pub fn or(&self, register: R8) -> u8 {
        self.fetch8(register) | self.registers.a
    }
    pub fn fetch8(&self, register: R8) -> u8 {
        match register {
            R8::A => self.registers.a,
            R8::B => self.registers.b, 
            R8::C => self.registers.c,
            R8::D => self.registers.d, 
            R8::E => self.registers.e,
            R8::H => self.registers.h,
            R8::L => self.registers.l,
            R8::M => self.registers.m,
            R8::T => self.registers.t,
        }
    }
    pub fn fetch16(&self, register: R16) -> u16 {
        match register {
            R16::PC => self.registers.pc,
            R16::SP => self.registers.sp,
            R16::BC => u8s_to_u16(self.registers.b, self.registers.c),
            R16::DE => u8s_to_u16(self.registers.d, self.registers.e),
            R16::HL => u8s_to_u16(self.registers.h, self.registers.l),
        }
    }
    pub fn set8(&mut self, register: R8, value: u8) {
        let reg = match register {
            R8::A => &mut self.registers.a,
            R8::B => &mut self.registers.b,
            R8::C => &mut self.registers.c,
            R8::D => &mut self.registers.d,
            R8::E => &mut self.registers.e,
            R8::H => &mut self.registers.h,
            R8::L => &mut self.registers.l,
            R8::M => &mut self.registers.m,
            R8::T => &mut self.registers.t,
        };
        *reg = value;
    }
    pub fn set16(&mut self, register: R16, value: u16) {
        let split = u16_to_u8_tuple(value);
        match register {
            R16::PC => self.registers.pc = value,
            R16::SP => self.registers.sp = value,
            R16::BC => {
                self.registers.b = split.0;
                self.registers.c = split.1;
            }
            R16::DE => {
                self.registers.d = split.0;
                self.registers.e = split.1;
            }
            R16::HL => {
                self.registers.h = split.0;
                self.registers.l = split.1;
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

struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    m: u8,
    t: u8,
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
    fn cpu_can_fetch_registers() {
        let mut cpu = CPU::new();
        cpu.registers.a = 1;
        cpu.registers.b = 1;
        cpu.registers.c = 1;
        assert_eq!(cpu.registers.a, cpu.fetch8(R8::A));
        assert_eq!(
            u8s_to_u16(cpu.registers.b, cpu.registers.c),
            cpu.fetch16(R16::BC)
        );
        assert_eq!(0b100000001, cpu.fetch16(R16::BC));
    }
    #[test]
    fn cpu_can_set_8bit_registers() {
        let mut cpu1 = CPU::new();
        cpu1.registers.a = 1;
        let mut cpu2 = CPU::new();
        cpu2.set8(R8::A, 1);
        assert_eq!(cpu1.registers.a, cpu2.registers.a);
    }
    #[test]
    fn cpu_can_set_16bit_registers() {
        let mut cpu1 = CPU::new();
        cpu1.registers.b = 1;
        cpu1.registers.c = 1;
        let mut cpu2 = CPU::new();
        cpu2.set16(R16::BC, u8s_to_u16(1, 1));
        assert_eq!(
            cpu1.fetch16(R16::BC),
            cpu2.fetch16(R16::BC)
        );
        assert_eq!(0b100000001, cpu2.fetch16(R16::BC));
    }
    #[test]
    fn cpu_can_load_registers_to_registers() {
        let mut cpu = CPU::new();
        cpu.registers.a = 1;
        cpu.load(R8::B, R8::A);
        assert_eq!(cpu.registers.b, 1);
    }
    #[test]
    fn cpu_can_OR_and_AND() {
        let mut cpu = CPU::new();
        for i in 0..1 {
            for j in 0..1 {

                cpu.registers.a = i;
                cpu.registers.b = j;
                assert_eq!(cpu.or(R8::B), i | j);
                assert_eq!(cpu.and(R8::B), i & j);
            }
        }
    }
}
