#![allow(dead_code)]

use crate::mem::*;

pub mod registers {
    use crate::types::*;

    pub const FLAG_Z: Byte = 7;
    pub const FLAG_N: Byte = 6;
    pub const FLAG_H: Byte = 5;
    pub const FLAG_C: Byte = 4;

    // Double check the memory behavior of rust union creation
    #[derive(Copy, Clone)]
    struct BitSpace {
        lo: Byte,
        hi: Byte,
    }

    pub struct Registers {
        reg_af: Register,
        reg_bc: Register,
        reg_de: Register,
        reg_hl: Register,
        reg_sp: Register, // Stack pointer
        reg_pc: Register, // Program counter
    }

    // reg is the comination of the base registers
    // bitspace lets you select upper 8 bits or lower for indiv regs
    union Register {
        reg: Word,
        bitspace: BitSpace,
    }

    // Would it be best to not make a reg struct and implement new for it
    // I'm kinda inconsistant in what I do:
    // I impl functions for registers but for memory I create global statics
    //
    impl Registers {
        /// Values set are just what the gameboy does on bootup.
        ///
        /// For details on what values are set see [gameboy_regs].
        ///
        /// [gameboy_regs] = http://www.codeslinger.co.uk/pages/projects/gameboy/hardware.html
        pub fn new() -> Self {
            Registers {
                reg_af: Register { reg: 0x01B0 },
                reg_bc: Register { reg: 0x0013 },
                reg_de: Register { reg: 0x00D8 },
                reg_hl: Register { reg: 0x014D },
                reg_sp: Register { reg: 0xFFFE },
                reg_pc: Register { reg: 0x100 },
            }
        }
    }
}

pub mod instructions {}

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    pub registers: registers::Registers,
    device_memory: Memory, // 64KB address space
    pub halted: bool,
    pub ime: bool, // Interrupt Master Enable
    pub cycles: u64,
}

impl CPU {
    pub fn new() -> Self {
        let mut cpu = CPU {
            registers: registers::Registers::new(),
            device_memory: Memory::new(),
            halted: false,
            ime: false,
            cycles: 0,
        };

        cpu.device_memory.ram_startup();

        cpu
    }

    pub fn reset(&mut self) {
        self.registers = registers::Registers::new();
        self.device_memory = Memory::new();
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
        unimplemented!();
    }
}

#[cfg(test)]
mod test {

    //use super::*;
}
