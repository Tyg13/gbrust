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
                flags: 0,
                pc: 0,
                sp: 0,
                m: 0,
                t: 0,
            },
        }
    }
    pub fn tick(&mut self) {
        self.clock.tick()
    }
}

#[derive(Clone)]
struct Clock {
    m: u8,
    t: u8,
}

impl Clock {
    pub fn tick(&mut self) { 
        self.m += 1;
        self.t += 4;
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
    flags: u8,
    pc: u16,
    sp: u16,
    m: u8,
    t: u8,
}

impl Registers {
    pub fn fetch8(&self, register: R8) -> u8 {
        match register {
            R8::A => self.a,
            R8::B => self.b, 
            R8::C => self.c,
            R8::D => self.d, 
            R8::E => self.e,
            R8::H => self.h,
            R8::L => self.l,
            R8::M => self.m,
            R8::T => self.t,
        }
    }
    pub fn fetch16(&self, register: R16) -> u16 {
        match register {
            R16::PC => self.pc,
            R16::SP => self.sp,
            R16::BC => u8s_to_u16(self.b, self.c),
            R16::DE => u8s_to_u16(self.d, self.e),
            R16::HL => u8s_to_u16(self.h, self.l),
        }
    }
    pub fn set8(&mut self, register: R8, value: u8) {
        let reg = match register {
            R8::A => &mut self.a,
            R8::B => &mut self.b,
            R8::C => &mut self.c,
            R8::D => &mut self.d,
            R8::E => &mut self.e,
            R8::H => &mut self.h,
            R8::L => &mut self.l,
            R8::M => &mut self.m,
            R8::T => &mut self.t,
        };
        *reg = value;
    }
    pub fn set16(&mut self, register: R16, value: u16) {
        match register {
            R16::PC => self.pc = value,
            R16::SP => self.sp = value,
            R16::BC => {
                let val = u16_to_u8_tuple(value);
                self.b = val.0;
                self.c = val.1;
            }
            R16::DE => {
                let val = u16_to_u8_tuple(value);
                self.d = val.0;
                self.e = val.1;
            }
            R16::HL => {
                let val = u16_to_u8_tuple(value);
                self.h = val.0;
                self.l = val.1;
            }
        }
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
        let old_clock = cpu.clock.clone();
        cpu.tick();
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
        assert_eq!(cpu.registers.a, cpu.registers.fetch8(R8::A));
        assert_eq!(
            u8s_to_u16(cpu.registers.b, cpu.registers.c),
            cpu.registers.fetch16(R16::BC)
        );
        assert_eq!(0b100000001, cpu.registers.fetch16(R16::BC));
    }
    #[test]
    fn cpu_can_set_8bit_registers() {
        let mut cpu1 = CPU::new();
        cpu1.registers.a = 1;
        let mut cpu2 = CPU::new();
        cpu2.registers.set8(R8::A, 1);
        assert_eq!(cpu1.registers.a, cpu2.registers.a);
    }
    #[test]
    fn cpu_can_set_16bit_registers() {
        let mut cpu1 = CPU::new();
        cpu1.registers.b = 1;
        cpu1.registers.c = 1;
        let mut cpu2 = CPU::new();
        cpu2.registers.set16(R16::BC, u8s_to_u16(1, 1));
        assert_eq!(
            cpu1.registers.fetch16(R16::BC),
            cpu2.registers.fetch16(R16::BC)
        );
        assert_eq!(0b100000001, cpu2.registers.fetch16(R16::BC));
    }
}
