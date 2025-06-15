#![allow(dead_code)]

use crate::types::*;



pub mod registers {
    use crate::types::*;

    pub const FLAG_Z: Byte = 7;
    pub const FLAG_N: Byte = 6;
    pub const FLAG_H: Byte = 5;
    pub const FLAG_C: Byte = 4;

    // Double check the memory behavior of rust union creation
    #[derive(Copy, Clone)]
    struct BitSpace{
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
    device_memory: Ram, // 64KB address space
    pub halted: bool,
    pub ime: bool, // Interrupt Master Enable
    pub cycles: u64,
}

impl CPU {
    pub fn new() -> Self {
        
        let mut cpu = CPU {
            registers: registers::Registers::new(),
            device_memory: [0; MEM_SIZE],
            halted: false,
            ime: false,
            cycles: 0,
        };

        cpu.ram_startup();

        cpu
    }

    pub fn reset(&mut self) {
        self.registers = registers::Registers::new();
        self.device_memory = [0; MEM_SIZE];
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

    // Wrapper for memory read functionality
    pub fn read_byte(&self, addr: Word) -> u8 {
        
        self.device_memory[addr as usize]
        
    }

    // Wrapper for memory write functionality
    pub fn write_byte(&mut self, addr: u16, value: u8) {// this is read only memory and should not be written to
        if addr < 0x8000 {
            //TODO: implement error handling here
        }
        // echo ram writes to two locations
        else if ( addr >= 0xE000 ) && (addr < 0xFE00) {
            self.device_memory[addr as usize] = value;
            self.write_byte(addr - 0x2000, value);

        }
        // restricted memory area
        else if ( addr >= 0xFEA0 ) && ( addr < 0xFEFF ) {
            //TODO: implement error handling here (likely throw some kind of interrupt)
        }
        else {
            self.device_memory[addr as usize] = value;
        }
    }

    /// Function for setting ram to requred startup values
    /// 
    /// Its pretty messy
    fn ram_startup(&mut self) {
        self.device_memory[0xFF05] = 0x00;
        self.device_memory[0xFF06] = 0x00;
        self.device_memory[0xFF07] = 0x00;
        self.device_memory[0xFF10] = 0x80;
        self.device_memory[0xFF11] = 0xBF;
        self.device_memory[0xFF12] = 0xF3;
        self.device_memory[0xFF14] = 0xBF;
        self.device_memory[0xFF16] = 0x3F;
        self.device_memory[0xFF17] = 0x00;
        self.device_memory[0xFF19] = 0xBF;
        self.device_memory[0xFF1A] = 0x7F;
        self.device_memory[0xFF1B] = 0xFF;
        self.device_memory[0xFF1C] = 0x9F;
        self.device_memory[0xFF1E] = 0xBF;
        self.device_memory[0xFF20] = 0xFF;
        self.device_memory[0xFF21] = 0x00;
        self.device_memory[0xFF22] = 0x00;
        self.device_memory[0xFF23] = 0xBF;
        self.device_memory[0xFF24] = 0x77;
        self.device_memory[0xFF25] = 0xF3;
        self.device_memory[0xFF26] = 0xF1;
        self.device_memory[0xFF40] = 0x91;
        self.device_memory[0xFF42] = 0x00;
        self.device_memory[0xFF43] = 0x00;
        self.device_memory[0xFF45] = 0x00;
        self.device_memory[0xFF47] = 0xFC;
        self.device_memory[0xFF48] = 0xFF;
        self.device_memory[0xFF49] = 0xFF;
        self.device_memory[0xFF4A] = 0x00;
        self.device_memory[0xFF4B] = 0x00;
        self.device_memory[0xFFFF] = 0x00;
    }
}



#[cfg(test)]
mod test{

    use super::*;

    #[test]
    fn test_cpu_startup() {
        let cpu: CPU = CPU::new();

        // Selects a couple of memory addresses to check
        assert_eq!(cpu.read_byte(0xFF11), 0xBF);
        assert_eq!(cpu.read_byte(0xFF19), 0xBF);
        assert_eq!(cpu.read_byte(0xFF24), 0x77);
    }

    #[test]
    fn test_read_write_ram() {
        let mut cpu: CPU = CPU::new();
        
        //Writing to valid memory space
        cpu.write_byte(0xD000, 0x9);
        assert_eq!(0x9, cpu.read_byte(0xD000));

        //Writing to valid memory space
        cpu.write_byte(0xD010, 0x9);
        assert_eq!(0x9, cpu.read_byte(0xD010));

        //Writing to echo memory space
        cpu.write_byte(0xE000, 0x9);
        assert_eq!(0x9, cpu.read_byte(0xE000));
        assert_eq!(0x9, cpu.read_byte(0xE000 - 0x2000));

    }

    #[test]
    fn test_invalid_write() {
        let mut cpu: CPU = CPU::new();

        //Writing to invalid memory space
        cpu.write_byte(0x0, 0x9);
        assert_ne!(0x9, cpu.read_byte(0x0));

        //Writing to invalid memory space
        cpu.write_byte(0x10, 0x9);
        assert_ne!(0x9, cpu.read_byte(0x10));

        //Writing to invalid memory space
        cpu.write_byte(0xFEA0, 0x9);
        assert_ne!(0x9, cpu.read_byte(0xFEA0));
    }
}
