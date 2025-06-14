#![allow(dead_code)]

pub mod registers {
    use crate::types::*;

    pub struct Registers {
        eax: Register,
    }
    union Register {
        int: Byte,
    }

    impl Registers {
        pub fn new() -> Self {
            Registers {
                eax: Register { int: 0 },
            }
        }
    }
}

pub mod instructions {}

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    pub registers: registers::Registers,
    pub memory: [u8; 0x10000], // 64KB address space
    pub halted: bool,
    pub ime: bool, // Interrupt Master Enable
    pub cycles: u64,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: registers::Registers::new(),
            memory: [0; 0x10000],
            halted: false,
            ime: false,
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.registers = registers::Registers::new();
        self.memory = [0; 0x10000];
        self.halted = false;
        self.ime = false;
        self.cycles = 0;
    }

    pub fn step(&mut self) {
        // Skeleton: Fetch, Decode, Execute cycle
        // let pc = self.registers.pc;
        // let opcode = self.memory[pc as usize];
        // instructions::execute(self, opcode);
    }

    // Placeholder for interrupt handling
    pub fn handle_interrupts(&mut self) {
        // TODO: Implement interrupt handling
    }

    // Placeholder for memory read
    pub fn read_byte(&self, _addr: u16) -> u8 {
        // TODO: Implement memory read logic
        0
    }

    // Placeholder for memory write
    pub fn write_byte(&mut self, _addr: u16, _value: u8) {
        // TODO: Implement memory write logic
    }
}
