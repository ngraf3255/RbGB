use crate::types::{Byte, Word};

#[derive(Copy, Clone)]
pub enum CpuFlag {
    Z = 0x80,
    N = 0x40,
    H = 0x20,
    C = 0x10,
}

pub struct Registers {
    pub a: Byte,
    pub b: Byte,
    pub c: Byte,
    pub d: Byte,
    pub e: Byte,
    pub h: Byte,
    pub l: Byte,
    pub f: Byte,
    pub sp: Word,
    pub pc: Word,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    pub fn reset(&mut self) {
        *self = Registers::new();
    }

    pub fn af(&self) -> Word {
        ((self.a as Word) << 8) | self.f as Word
    }

    pub fn setaf(&mut self, value: Word) {
        self.a = (value >> 8) as Byte;
        self.f = (value as Byte) & 0xF0;
    }

    pub fn bc(&self) -> Word {
        ((self.b as Word) << 8) | self.c as Word
    }

    pub fn setbc(&mut self, value: Word) {
        self.b = (value >> 8) as Byte;
        self.c = value as Byte;
    }

    pub fn de(&self) -> Word {
        ((self.d as Word) << 8) | self.e as Word
    }

    pub fn setde(&mut self, value: Word) {
        self.d = (value >> 8) as Byte;
        self.e = value as Byte;
    }

    pub fn hl(&self) -> Word {
        ((self.h as Word) << 8) | self.l as Word
    }

    pub fn sethl(&mut self, value: Word) {
        self.h = (value >> 8) as Byte;
        self.l = value as Byte;
    }

    pub fn hli(&mut self) -> Word {
        let addr = self.hl();
        self.sethl(addr.wrapping_add(1));
        addr
    }

    pub fn hld(&mut self) -> Word {
        let addr = self.hl();
        self.sethl(addr.wrapping_sub(1));
        addr
    }

    pub fn flag(&mut self, flag: CpuFlag, set: bool) {
        let bit = flag as Byte;
        if set {
            self.f |= bit;
        } else {
            self.f &= !bit;
        }
    }

    pub fn getflag(&self, flag: CpuFlag) -> bool {
        (self.f & flag as Byte) != 0
    }
}

