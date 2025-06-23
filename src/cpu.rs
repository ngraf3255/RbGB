#![allow(dead_code)]

use registers::Register;

use crate::mem::*;
use crate::types::*;
use std::sync::{Arc, Mutex};

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
        pub reg_af: Register,
        pub reg_bc: Register,
        pub reg_de: Register,
        pub reg_hl: Register,
        pub reg_sp: Register, // Stack pointer
        pub reg_pc: Register, // Program counter
    }

    // reg is the comination of the base registers
    // bitspace lets you select upper 8 bits or lower for indiv regs
    #[derive(Copy, Clone)]
    pub union Register {
        reg: Word,
        bitspace: BitSpace,
    }

    impl Register {
        pub fn new(val: Word) -> Self {
            Register {
                reg: val,
            }
        }

        pub fn set(&mut self, val: Word) {
            self.reg = val;
        }
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
    device_memory: SharedMemory, // 64KB address space
    timers: Timer,
    halted: bool,
    ime: bool, // Interrupt Master Enable
    cycles: u64,
}

impl CPU {
    pub fn new() -> Self {
        // Creates new mem object on the heap
        let mem = Arc::new(Mutex::new(Memory::new()));

        let cpu = CPU {
            registers: registers::Registers::new(),
            device_memory: Arc::clone(&mem),
            timers: Timer::new(Arc::clone(&mem)),
            halted: false,
            ime: false,
            cycles: 0,
        };

        cpu.device_memory.lock().unwrap().ram_startup();

        cpu
    }

    pub fn reset(&mut self) {
        let mem = Arc::new(Mutex::new(Memory::new()));
        self.registers = registers::Registers::new();
        self.device_memory = Arc::clone(&mem);
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
        // Checks interrupt master enable
        if self.ime {
            // Aquire lock on memory
            let mem = self.device_memory.lock().unwrap();

            let request = mem.read_byte(IF);
            let enabled = mem.read_byte(IE);
            drop(mem); // Memory is dropped after all reads are done

            if request != 0 {
                for i in 0..5 {
                    let req_bit = (request >> i) & 1 != 0;
                    let ena_bit = (enabled >> i) & 1 != 0;
                    if req_bit && ena_bit{
                        self.service_interrupt(i);
                    }
                }
            }
        }
    }

    fn service_interrupt(&mut self, interrupt: Byte) {
        // New lock aquired on memory
        let mut mem = self.device_memory.lock().unwrap(); 

        self.ime = false; // Disables new interrupts
        let mut request = mem.read_byte(IF);
        request = request & (!(2^interrupt)); // Clears interrupt
        mem.write_byte(IF, request);

        drop(mem); // Drops memory since we are done writing

        // Save current execution location on stack
        self.push_stack(self.registers.reg_pc);

        // Set the program counter to the address of the ISRs
        match interrupt {
            0 => self.registers.reg_pc.set(0x40),
            1 => self.registers.reg_pc.set(0x48),
            2 => self.registers.reg_pc.set(0x50),
            4 => self.registers.reg_pc.set(0x60),
            _ => self.registers.reg_pc.set(0x40),
        }

        unimplemented!();
    }

    /// Pushes the provided register onto the stack
    fn push_stack(&self, reg: registers::Register) {
        unimplemented!()

    }
}

/// Memory wrapper class that implements functions to update and run timers
struct Timer {
    mem: SharedMemory,
    divider_counter: u32,
}

impl Timer {
    pub fn new(mem: SharedMemory) -> Self {
        Timer {
            mem,

            divider_counter: 0,
        }
    }

    pub fn update_timers(&mut self, cycles: i32) {
        self.do_divier_register(cycles);

        //the clock must be enabled to update itself
        if self.is_clock_enabled() {
            let mut mem = self.mem.lock().unwrap();
            let timer_counter = mem.timer_counter;

            mem.timer_counter -= cycles;

            // enough cpu cycled have happened to update the timer
            if timer_counter <= 0 {
                // reset timer counter to correct value;
                mem.set_clock_frequency();

                // timer is close to overflowing
                // aquire a lock on memory for this operation
                if mem.read_byte(TIMA) == 255 {
                    //Reset the timer back to the overflow state
                    let tma_val = mem.read_byte(TMA);
                    mem.write_byte(TIMA, tma_val);
                    // We are done accessing memory so we drop the lock
                    mem.request_interrupt(2);
                } else {
                    // Incriments timer
                    let tima_val = mem.read_byte(TIMA) + 1;
                    mem.write_byte(TIMA, tima_val);
                }
            }
        }
    }

    fn do_divier_register(&mut self, cycles: i32) {
        self.divider_counter += cycles as u32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;

            let mut mem = self.mem.lock().unwrap();
            // 0xFF04 is the location of the divider register
            let divider_register = mem.read_byte_forced(DIVIDER_REGISTER) + 1;
            mem.write_byte_forced(DIVIDER_REGISTER, divider_register);
        }
    }

    /// Checks bit 2 in TMC to see if timer is currently enabled
    fn is_clock_enabled(&self) -> bool {
        // Get a lock on the memory
        let mem = self.mem.lock().unwrap();
        let tmc_reg = mem.read_byte(TMC);
        // Test bit 2
        tmc_reg & 0x4 != 0
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use ntest::timeout;

    #[test]
    #[timeout(1)]
    fn test_cpu_init() {
        let cpu = CPU::new();

        assert_eq!(cpu.cycles, 0);
    }
}
