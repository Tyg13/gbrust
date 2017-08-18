struct CPU {
    clock: Clock,
    registers: Registers
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            clock: Clock {
                m: 0, t: 0
            },
            registers: Registers {
                a: 0, b: 0, c: 0, d: 0, e: 0,
                h: 0, l: 0, flags: 0,
                pc: 0, sp: 0,
                m: 0, t: 0
            }
        }
    }
    pub fn get_clock(&self) -> &Clock {
        &self.clock
    }
    pub fn tick(&mut self) {
        self.clock.m += 1;
        self.clock.t += 4;
    }
}

#[derive(Clone)]
struct Clock {
    m: u8,
    t: u8
}

struct Registers {
    a: u8, b: u8, c: u8, d: u8, e: u8,
    h: u8, l: u8,
    flags: u8,
    pc: u16, sp: u16,
    m: u8, t: u8
}

impl Registers {
    pub fn fetch8(&self, register: Register8) -> u8 {
        match register {
            a => self.a, b => self.b, c => self.c,
            d => self.d, e => self.e,
            h => self.h, l => self.l,
            m => self.m, t => self.t
        }
    }
    pub fn fetch16(&self, register: Register16) -> u16 {
        match register {
            Register16::pc => self.pc, sp => self.sp,
            Register16::bc => u8s_to_u16(self.b, self.c),
            Register16::de => u8s_to_u16(self.d, self.e),
            Register16::hl => u8s_to_u16(self.h, self.l)
        }
    }
}

pub fn u8s_to_u16(high: u8, low: u8) -> u16 {
    let top = (high as u16) << 8;
    top + (low as u16)
}

pub fn u16_to_u8_tuple(wbyte: u16) -> (u8, u8) {
    ((wbyte << 8) as u8, wbyte as u8)
}

pub enum Register8 {
    a, b, c, d, e,
    h, l,
}

pub enum Register16 {
    pc, sp, bc, de, hl
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn cpu_can_tick() {
        let mut cpu = CPU::new();
        let old_clock = cpu.get_clock().clone();
        cpu.tick();
        let new_clock = cpu.get_clock();
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
        assert_eq!(cpu.registers.a, cpu.registers.fetch8(Register8::a));
        assert_eq!(1, cpu.registers.fetch8(Register8::a));
        assert_eq!(u8s_to_u16(cpu.registers.b, cpu.registers.c), cpu.registers.fetch16(Register16::bc));
        assert_eq!(257, cpu.registers.fetch16(Register16::bc));
    }
}