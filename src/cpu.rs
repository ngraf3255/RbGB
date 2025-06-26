#![allow(dead_code)]

use debug_print::debug_println;

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
            Register { reg: val }
        }

        pub fn set(&mut self, val: Word) {
            self.reg = val;
        }

        /// Returns the value in the register
        pub fn value(&self) -> Word {
            unsafe { self.reg }
        }

        /// Gets the upper 4 bits
        pub fn high_value(&self) -> Byte {
            unsafe { self.bitspace.hi }
        }

        /// Gets the lower 4 bits
        pub fn low_value(&self) -> Byte {
            unsafe { self.bitspace.lo }
        }

        /// Decriments the register
        pub fn decriment(&mut self) {
            unsafe {
                self.reg -= 1;
            }
        }

        ///Incriments the register
        pub fn incriment(&mut self) {
            unsafe {
                self.reg += 1;
            }
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
            ime: true,
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
            drop(mem); // Memory lock is dropped after all reads are done

            if request != 0 {
                debug_println!("Interrupt Requested");
                for i in 0..5 {
                    let req_bit = (request >> i) & 1 != 0;
                    let ena_bit = (enabled >> i) & 1 != 0;
                    if req_bit && ena_bit {
                        self.service_interrupt(i);
                    }
                }
            }
        }
    }

    fn service_interrupt(&mut self, interrupt: Byte) {
        // New lock aquired on memory
        let mut mem = self.device_memory.lock().unwrap();

        debug_println!("Servicing interrupt {}", interrupt);
        self.ime = false; // Disables new interrupts
        let mut request = mem.read_byte(IF);
        // Clear the requested interrupt flag. The previous implementation used
        // `2 ^ interrupt` which performs a bitwise XOR and resulted in the
        // wrong bit being cleared.
        request &= !(1 << interrupt); // Clears interrupt
        mem.write_byte(IF, request);

        drop(mem); // Drops memory since we are done writing

        // Save current execution location on stack
        debug_println!("Pushing PC on stack: {:#X}", self.registers.reg_pc.value());
        self.push_stack(self.registers.reg_pc);

        // Set the program counter to the address of the ISRs
        match interrupt {
            0 => self.registers.reg_pc.set(0x40),
            1 => self.registers.reg_pc.set(0x48),
            2 => self.registers.reg_pc.set(0x50),
            4 => self.registers.reg_pc.set(0x60),
            _ => self.registers.reg_pc.set(0x40),
        }
    }

    /// Pushes the provided register onto the stack
    ///
    /// Careful that this is called after all registers are initialized
    fn push_stack(&mut self, reg: registers::Register) {
        let mut mem = self.device_memory.lock().unwrap();
        // Decriments the stack pointer by one byte
        self.registers.reg_sp.decriment();
        mem.write_byte(self.registers.reg_sp.value(), reg.high_value());

        self.registers.reg_sp.decriment();
        mem.write_byte(self.registers.reg_sp.value(), reg.low_value());
        drop(mem);
    }

    /// Pops from the top of the stack
    fn pop_stack(&mut self) -> Word {
        let mem = self.device_memory.lock().unwrap();

        let mut return_word: Word = (mem.read_byte(self.registers.reg_sp.value() + 1) as Word) << 8;
        return_word |= mem.read_byte(self.registers.reg_sp.value()) as Word;

        self.registers.reg_sp.incriment();
        self.registers.reg_sp.incriment();

        return_word
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

    #[test]
    #[timeout(1)]
    fn test_cpu_interrupts() {
        let mut cpu = CPU::new();

        assert_eq!(cpu.cycles, 0);
        assert!(cpu.ime);

        let mut mem = cpu.device_memory.lock().unwrap();
        mem.request_interrupt(1); // Request vblank interrupt
        mem.enable_interrupt(1); // Enabled interrupt
        assert_eq!(mem.read_byte(IF), 0x2);
        drop(mem);
        cpu.handle_interrupts();

        assert_eq!(cpu.registers.reg_pc.value(), 0x48);
        assert!(!cpu.ime);

        let ret = cpu.pop_stack();
        assert_eq!(ret, 0x100);
    }

    #[test]
    #[timeout(1)]
    fn test_push_pop_stack() {
        let mut cpu = CPU::new();

        cpu.push_stack(cpu.registers.reg_af);
        assert_eq!(cpu.registers.reg_sp.value(), 0xFFFC);

        let ret = cpu.pop_stack();
        assert_eq!(ret, 0x01B0);
        assert_eq!(cpu.registers.reg_sp.value(), 0xFFFE);
    }
}
