#![allow(dead_code)]

use debug_print::debug_println;

use crate::bus::Bus;
use crate::mem::*;
use crate::registers;
use crate::types::*;
use std::sync::{Arc, Mutex};

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    pub registers: registers::Registers,
    device_memory: SharedMemory, // 64KB address space
    pub timers: Timer,
    halted: bool,
    ime: bool, // Interrupt Master Enable
    cycles: u64,
}

impl CPU {
    pub fn new(mem: SharedMemory) -> Self {
        // Creates new mem object on the heap

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

    /// load d (as in IX+d) from memory and advance PC
    #[inline(always)]
    fn d(&mut self) -> Byte {
        let mem = self.device_memory.lock().unwrap();
        let pc = self.registers.reg_pc.value();
        let d = mem.read_byte(pc);
        self.registers.inc_pc(1);
        d
    }

    /// load effective address HL, IX+d or IY+d with existing d
    /// this is for DD CB and FD DB instructions
    #[inline(always)]
    fn addr_d(&mut self, d: Word, ext: bool) -> Word {
        if ext {
            let addr = (self.registers.r16sp(2) + d) & 0xFFFF;
            self.registers.set_wz(addr);
            addr
        } else {
            self.registers.val_hl()
        }
    }

    #[inline(always)]
    fn check_condition(&self, y: Byte) -> bool {
        let f = self.registers.val_f();
        match y {
            0 => 0 == f & ZF, // JR NZ
            1 => 0 != f & ZF, // JR Z
            2 => 0 == f & CF, // JR NC
            3 => 0 != f & CF, // JC C
            4 => 0 == f & PF, // JR PO
            5 => 0 != f & PF, // JR PE
            6 => 0 == f & SF, // JR P
            7 => 0 != f & SF, // JR M
            _ => false,
        }
    }

    /// load 16-bit immediate operand and bump PC
    #[inline(always)]
    fn imm16(&mut self) -> Word {
        let pc = self.registers.val_pc();
        let mem = self.device_memory.lock().unwrap();

        let imm = mem.read_byte(pc) as Word | ((mem.read_byte(pc + 1) as Word) << 8) as Word;
        self.registers.inc_pc(2);
        imm
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
        self.push_stack(self.registers.reg_pc.value());

        // Set the program counter to the address of the ISRs
        debug_println!("Matching interrupt {interrupt}");
        match interrupt {
            0 => self.registers.reg_pc.set(0x40), // Vblank
            1 => self.registers.reg_pc.set(0x48), // STAT
            2 => self.registers.reg_pc.set(0x50), // Timer Interrupt
            3 => self.registers.reg_pc.set(0x58), // Serial Interrupt
            4 => self.registers.reg_pc.set(0x60), // Joypad
            _ => self.registers.reg_pc.set(0x40), // Default
        }
    }

    /// Pushes the provided register onto the stack
    ///
    /// Careful that this is called after all registers are initialized
    fn push_stack(&mut self, reg: Word) {
        let mut mem = self.device_memory.lock().unwrap();
        // Decriments the stack pointer by one byte
        let reg = registers::Register::new(reg);
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

    pub fn execute_next_opcode(&mut self, extention: bool) -> i64 {
        let (cycle, extention_cycle) = if extention { (4, 8) } else { (0, 0) };

        let operation = self.step_opcode();

        // seems like the best way to do this from online is to do a match case statement
        let x = operation >> 6;
        let y = operation >> 3 & 7;
        let z = operation & 7;

        //let mem = self.device_memory.lock().unwrap();

        cycle
            + match (x, y, z) {
                // --- block 1: 8-bit loads
                // special case LD (HL),(HL): HALT
                (1, 6, 6) => {
                    self.halt();
                    4
                }
                // LD (HL),r; LD (IX+d),r; LD (IY+d),r
                // NOTE: this always loads from H,L, never IXH, ...
                (1, 6, _) => {
                    let a = self.load_addr(extention);
                    let v = self.registers.get_reg8_by_index(z);
                    self.device_memory.lock().unwrap().write_byte(a, v);
                    7 + extention_cycle
                }
                // LD r,(HL); LD r,(IX+d); LD r,(IY+d)
                // NOTE: this always loads to H,L, never IXH,...
                (1, _, 6) => {
                    let a = self.load_addr(extention);
                    let v = self.device_memory.lock().unwrap().read_byte(a);
                    self.registers.set_reg8_by_index(y, v);
                    7 + extention_cycle
                }
                // LD r,s
                (1, _, _) => {
                    let v = self.registers.get_reg8_by_index(z);
                    self.registers.set_reg8_by_index(y, v);
                    4
                }
                // --- block 2: 8-bit ALU instructions
                // ALU (HL); ALU (IX+d); ALU (IY+d)
                (2, _, _) => {
                    if z == 6 {
                        // ALU (HL); ALU (IX+d); ALU (IY+d)
                        let a = self.load_addr(extention);
                        let val = self.device_memory.lock().unwrap().read_byte(a);
                        self.alu8(y, val);
                        7 + extention_cycle
                    } else {
                        // ALU r
                        let val = self.registers.get_reg8_by_index(z);
                        self.alu8(y.into(), val);
                        4
                    }
                }
                // --- block 0: misc ops
                // NOP
                (0, 0, 0) => 4,
                // EX AF,AF'
                (0, 1, 0) => {
                    self.registers.swap(AF, AF_); // TODO: Change out swap
                    4
                }
                // DJNZ
                (0, 2, 0) => self.djnz(),
                // JR d
                (0, 3, 0) => {
                    let pc = self.registers.val_pc();
                    let wz = pc + self.device_memory.lock().unwrap().read_byte(pc) as Word + 1;
                    self.registers.set_pc(wz);
                    self.registers.set_wz(wz);
                    12
                }
                // JR cc
                (0, _, 0) => {
                    let pc = self.registers.val_pc();
                    if self.check_condition(y - 4) {
                        let wz = pc + self.device_memory.lock().unwrap().read_byte(pc) as Word + 1;
                        self.registers.set_pc(wz);
                        self.registers.set_wz(wz);
                        12
                    } else {
                        self.registers.inc_pc(1);
                        7
                    }
                }
                // 16-bit immediate loads and 16-bit ADD
                (0, _, 1) => {
                    let p = y >> 1;
                    let q = y & 1;
                    if q == 0 {
                        // LD rr,nn (inkl IX,IY)
                        let val = self.imm16();
                        self.registers.set_r16sp(p, val);
                        10
                    } else {
                        // ADD HL,rr; ADD IX,rr; ADD IY,rr
                        let acc = self.registers.r16sp(2);
                        let val = self.registers.r16sp(p);
                        let res = self.add16(acc, val);
                        self.registers.set_r16sp(2, res);
                        11
                    }
                }
                (0, _, 2) => {
                    // indirect loads
                    let p = y >> 1;
                    let q = y & 1;
                    match (q, p) {
                        // LD (nn),HL; LD (nn),IX; LD (nn),IY
                        (0, 2) => {
                            let addr = self.imm16();
                            let v = self.registers.r16sp(2);
                            let mut mem = self.device_memory.lock().unwrap();
                            mem.write_word(addr, v);
                            self.registers.set_wz(addr + 1);
                            16
                        }
                        // LD (nn),A
                        (0, 3) => {
                            let addr = self.imm16();
                            let a = self.registers.val_a();
                            self.device_memory.lock().unwrap().write_byte(addr, a);
                            self.registers.set_wz(addr + 1);
                            13
                        }
                        // LD (BC),A; LD (DE),A,; LD (nn),A
                        (0, _) => {
                            let addr = if p == 0 {
                                self.registers.val_bc()
                            } else {
                                self.registers.val_de()
                            };
                            let a = self.registers.val_a();
                            self.device_memory.lock().unwrap().write_byte(addr, a);
                            self.registers
                                .set_wz((a as Word) << 8 | ((addr + 1) & 0xFF));
                            7
                        }
                        // LD HL,(nn); LD IX,(nn); LD IY,(nn)
                        (1, 2) => {
                            let addr = self.imm16();
                            let val = self.device_memory.lock().unwrap().read_word(addr);
                            self.registers.set_r16sp(2, val);
                            self.registers.set_wz(addr + 1);
                            16
                        }
                        // LD A,(nn)
                        (1, 3) => {
                            let addr = self.imm16();
                            let val = self.device_memory.lock().unwrap().read_byte(addr);
                            self.registers.set_a(val);
                            self.registers.set_wz(addr + 1);
                            13
                        }
                        // LD A,(BC); LD A,(DE)
                        (1, _) => {
                            let addr = if p == 0 {
                                self.registers.val_bc()
                            } else {
                                self.registers.val_de()
                            };
                            let val = self.device_memory.lock().unwrap().read_byte(addr);
                            self.registers.set_a(val);
                            self.registers.set_wz(addr + 1);
                            7
                        }
                        (_, _) => unreachable!(),
                    }
                }
                (0, _, 3) => {
                    // 16-bit INC/DEC
                    let p = y >> 1;
                    let q = y & 1;
                    let val = self.registers.r16sp(p) + if q == 0 { 1 } else { Word::MAX };
                    self.registers.set_r16sp(p, val);
                    6
                }
                // INC (HL); INC (IX+d); INC (IY+d)
                (0, 6, 4) => {
                    let addr = self.load_addr(extention);
                    let v = self.device_memory.lock().unwrap().read_byte(addr);
                    let w = self.inc8(v);
                    self.device_memory.lock().unwrap().write_byte(addr, w);
                    11 + extention_cycle
                }
                // INC r
                (0, _, 4) => {
                    let v = self.registers.get_reg8_by_index(y);
                    let w = self.inc8(v);
                    self.registers.set_reg8_by_index(y, w);
                    4
                }
                // DEC (HL); DEC (IX+d); DEC (IY+d)
                (0, 6, 5) => {
                    let addr = self.load_addr(extention);
                    let v = self.device_memory.lock().unwrap().read_byte(addr);
                    let w = self.dec8(v);
                    self.device_memory.lock().unwrap().write_byte(addr, w);
                    11 + extention_cycle
                }
                // DEC r
                (0, _, 5) => {
                    let v = self.registers.get_reg8_by_index(y);
                    let w = self.dec8(v);
                    self.registers.set_reg8_by_index(y, w);
                    4
                }
                // LD r,n; LD (HL),n; LD (IX+d),n; LD (IY+d),n
                (0, _, 6) => {
                    if y == 6 {
                        // LD (HL),n; LD (IX+d),n; LD (IY+d),n
                        let addr = self.load_addr(extention);
                        let v = self.imm8();
                        self.device_memory.lock().unwrap().write_byte(addr, v);
                        if extention { 15 } else { 10 }
                    } else {
                        // LD r,n
                        let v = self.imm8();
                        self.registers.set_reg8_by_index(y, v);
                        7
                    }
                }
                // misc ops on A and F
                (0, _, 7) => {
                    match y {
                        0 => self.rlca8(),
                        1 => self.rrca8(),
                        2 => self.rla8(),
                        3 => self.rra8(),
                        4 => self.daa(),
                        5 => self.cpl(),
                        6 => self.scf(),
                        7 => self.ccf(),
                        _ => unreachable!(),
                    }
                    4
                }
                // --- block 3: misc and prefixed ops
                (3, _, 0) => {
                    // RET cc
                    self.retcc(y)
                }
                (3, _, 1) => {
                    let p = y >> 1;
                    let q = y & 1;
                    match (q, p) {
                        (0, _) => {
                            // POP BC,DE,HL,IX,IY
                            let val = self.pop_stack();
                            self.registers.set_r16af(p, val);
                            10
                        }
                        (1, 0) => {
                            // RET
                            self.ret()
                        }
                        (1, 1) => {
                            // EXX
                            self.registers.swap(BC, BC_);
                            self.registers.swap(DE, DE_);
                            self.registers.swap(HL, HL_);
                            self.registers.swap(WZ, WZ_);
                            4
                        }
                        (1, 2) => {
                            // JP HL; JP IX; JP IY
                            let v = self.registers.r16sp(2);
                            self.registers.set_pc(v);
                            4
                        }
                        (1, 3) => {
                            // LD SP,HL, LD SP,IX; LD SP,IY
                            let v = self.registers.r16sp(2);
                            self.registers.set_sp(v);
                            6
                        }
                        (_, _) => unreachable!(),
                    }
                }
                (3, _, 2) => {
                    // JP cc,nn
                    let nn = self.imm16();
                    self.registers.set_wz(nn);
                    if self.check_condition(y) {
                        self.registers.set_pc(nn);
                    }
                    10
                }
                (3, _, 3) => {
                    // misc ops
                    match y {
                        0 => {
                            // JP nn
                            let nn = self.imm16();
                            self.registers.set_wz(nn);
                            self.registers.set_pc(nn);
                            10
                        }
                        1 => self.do_cb_op(extention),
                        2 => {
                            // OUT (n),A
                            let a = self.registers.val_a();
                            let port = (((a as Word) << 8 | self.imm8() as Word) as Word & 0xFFFF) as Word;
                            // self.outp(bus, port, a); TODO: Implement port output
                            11
                        }
                        3 => {
                            // IN A,(n)
                            let port = ((self.registers.val_a() as Word) << 8
                                | self.imm8() as Word)
                                & 0xFFFF;
                            //let v = self.inp(bus, port); TODO: Implement port input
                            // self.registers.set_a(v as Byte);
                            11
                        }
                        4 => {
                            // EX (SP),HL; EX (SP),IX; EX (SP),IY
                            let sp = self.registers.val_sp();
                            let v_reg = self.registers.r16sp(2);
                            let v_mem = self.device_memory.lock().unwrap().read_word(sp);
                            self.device_memory.lock().unwrap().write_word(sp, v_reg);
                            self.registers.set_wz(v_mem);
                            self.registers.set_r16sp(2, v_mem);
                            19
                        }
                        5 => {
                            // EX DE,HL
                            self.registers.swap(DE, HL);
                            4
                        }
                        6 => {
                            // DI
                            // self.iff1 = false;
                            // self.iff2 = false;
                            4
                        }
                        7 => {
                            // EI
                            self.ime = true;
                            4
                        }
                        _ => unreachable!(),
                    }
                }
                (3, _, 4) => {
                    // CALL cc
                    self.callcc(y)
                }
                (3, _, 5) => {
                    let p = y >> 1;
                    let q = y & 1;
                    match (q, p) {
                        (0, _) => {
                            // PUSH BC,DE,HL,IX,IY,AF
                            let v = self.registers.r16af(p);
                            self.push_stack(v);
                            11
                        }
                        (1, 0) => {
                            // CALL nn
                            self.call()
                        }
                        (1, 1) => {
                            // DD prefix instructions
                            // self.registers.patch_ix(); TODO: Implement IX patching
                            // let cycles = self.do_op(bus, true);
                            // self.registers.unpatch(); TODO: Implement IX patching
                            8 as i64
                        }
                        (1, 2) => {
                            // ED prefix instructions
                            self.do_ed_op()
                        }
                        (1, 3) => {
                            // FD prefix instructions
                            //self.registers.patch_iy();
                            //let cycles = self.do_op(bus, true);
                            //self.registers.unpatch(); TODO: Implement IX patching
                            8 as i64
                        }
                        (_, _) => unreachable!(),
                    }
                }
                // ALU n
                (3, _, 6) => {
                    let val = self.imm8();
                    self.alu8(y, val);
                    7
                }
                // RST
                (3, _, 7) => {
                    self.rst((y * 8) as Word);
                    11
                }
                // not implemented
                _ => panic!("Invalid instruction!"),
            }
    }

    pub fn djnz(&mut self) -> i64 {
        let b = (self.registers.val_b() - 1) & 0xFF;
        self.registers.set_b(b);
        if b > 0 {
            let addr = self.registers.val_pc();
            let d = self.device_memory.lock().unwrap().read_byte(addr) as Word;
            let wz = addr + d + 1;
            self.registers.set_wz(wz);
            self.registers.set_pc(wz);
            13 // return num cycles if branch taken
        } else {
            let pc = self.registers.val_pc() + 1;
            self.registers.set_pc(pc);
            8 // return num cycles if loop finished
        }
    }

    #[inline(always)]
    pub fn rst(&mut self, val: Word) {
        let pc = self.registers.val_pc();
        self.push_stack(pc);
        self.registers.set_pc(val);
        self.registers.set_wz(val);
    }

    /// fetch and execute ED prefix instruction
    fn do_ed_op(&mut self) -> i64 {
        let op = self.step_opcode();

        // split instruction byte into bit groups
        let x = op >> 6;
        let y = op >> 3 & 7;
        let z = op & 7;
        match (x, y, z) {
            // block instructions
            (2, 4, 0) => {
                self.ldi();
                16
            }
            (2, 5, 0) => {
                self.ldd();
                16
            }
            (2, 6, 0) => self.ldir(),
            (2, 7, 0) => self.lddr(),
            (2, 4, 1) => {
                self.cpi();
                16
            }
            (2, 5, 1) => {
                self.cpd();
                16
            }
            (2, 6, 1) => self.cpir(),
            (2, 7, 1) => self.cpdr(),
            (2, 4, 2) => {
                self.ini();
                16
            }
            (2, 5, 2) => {
                //self.ind(bus); TODO: Enable port input
                16
            }
            //(2, 6, 2) => self.inir(bus), TODO: Enable port input
            //(2, 7, 2) => self.indr(bus),
            (2, 4, 3) => {
                // self.outi(bus); TODO: Enable port output
                16
            }
            (2, 5, 3) => {
                //self.outd(bus); TODO: Enable port output
                16
            }
            //(2, 6, 3) => self.otir(bus), TODO: Enable port output
            //(2, 7, 3) => self.otdr(bus),
            (1, 6, 0) => {
                // IN F,(C) (undocumented special case, only alter flags,
                // don't store result)
                let bc = self.registers.val_bc();
                //let v = self.inp(bus, bc); TODO: Enable port input
                //let f = flags_szp(v) | (self.registers.val_f() & CF);
                //self.registers.set_f(f);
                12
            }
            (1, _, 0) => {
                // IN r,(C)
                let bc = self.registers.val_bc();
                let v = self.inp(bc);
                self.registers.set_reg8_by_index(y as Byte, v);
                let f = flags_szp(v) | (self.registers.val_f() & CF);
                self.registers.set_f(f);
                12
            }
            (1, 6, 1) => {
                // OUT (C),F (undocumented special case, always output 0)
                let bc = self.registers.val_bc();
                self.outp(bc, 0);
                12
            }
            (1, _, 1) => {
                // OUT (C),r
                let bc = self.registers.val_bc();
                let v = self.registers.get_reg8_by_index(y as Byte);
                self.outp(bc, v);
                12
            }
            (1, _, 2) => {
                // SBC/ADC HL,rr
                let p = y >> 1;
                let q = y & 1;
                let acc = self.registers.val_hl();
                let val = self.registers.r16sp(p);
                let res = if q == 0 {
                    self.sbc16(acc, val)
                } else {
                    self.adc16(acc, val)
                };
                self.registers.set_hl(res);
                15
            }
            (1, _, 3) => {
                // 16-bit immediate address load/store
                let p = y >> 1;
                let q = y & 1;
                let nn = self.imm16();
                if q == 0 {
                    // LD (nn),rr
                    let val = self.registers.r16sp(p as Byte);
                    self.device_memory.lock().unwrap().write_word(nn, val);
                } else {
                    // LD rr,(nn)
                    let val = self.device_memory.lock().unwrap().read_word(nn);
                    self.registers.set_r16sp(p, val);
                }
                self.registers.set_wz(nn + 1);
                20
            }
            (1, _, 4) => {
                self.neg8();
                8
            }
            (1, 1, 5) => {
                // RETI (RETN is not implemented)
                // self.reti(bus) TODO: Implement reti
                unimplemented!();
                10
            }
            (1, _, 6) => {
                // TODO: Implement
                let mut mem = self.device_memory.lock().unwrap();
                match y {
                    0 | 1 | 4 | 5 => {
                        mem.request_interrupt(0);
                    }
                    2 | 6 => {
                        mem.request_interrupt(1);
                    }
                    3 | 7 => {
                        mem.request_interrupt(2);
                    }
                    _ => unreachable!(),
                }
                drop(mem);
                8
            }
            (1, 0, 7) => {
                // self.registers.i = self.registers.val_a(); TODO: Implement I register
                unimplemented!();
                9
            } // LD I,A
            (1, 1, 7) => {
                // TODO: Implement refresh register
                //self.registers.r = self.registers.a();
                unimplemented!();
                9
            } // LD R,A
            (1, 2, 7) => {
                // LD A,I
                // let i = self.registers.i;
                // self.registers.set_a(i);
                // let f = flags_sziff2(i, self.iff2) | (self.registers.f() & CF);
                // self.registers.set_f(f);
                //TODO: Implement i register
                unimplemented!();
                9
            }
            (1, 3, 7) => {
                // LD A,R
                // let r = self.registers.r;
                // self.registers.set_a(r);
                // let f = flags_sziff2(r, self.iff2) | (self.registers.f() & CF);
                // self.registers.set_f(f);
                // TODO: Implement A register
                unimplemented!();
                9
            }
            (1, 4, 7) => {
                self.rrd();
                18
            } // RRD
            (1, 5, 7) => {
                self.rld();
                18
            } // RLD
            (1, _, 7) => 9, // NOP (ED)
            _ => panic!("CB: Invalid instruction!"),
        }
    }

    /// fetch and execute CB prefix instruction
    fn do_cb_op(&mut self, ext: bool) -> i64 {
        let d = if ext { self.d() } else { 0 };
        let op = self.step_opcode();
        let cyc = if ext { 4 } else { 0 };

        // split instruction byte into bit groups
        let x = op >> 6;
        let y = op >> 3 & 7;
        let z = op & 7;
        cyc + match x {
            0 => {
                // rotates and shifts
                if z == 6 {
                    // ROT (HL); ROT (IX+d); ROT (IY+d)
                    let a = self.addr_d(d as Word, ext);
                    let v = self.device_memory.lock().unwrap().read_byte(a);
                    let w = self.rot(y, v);
                    self.device_memory.lock().unwrap().write_byte(a, w);
                    15
                } else if ext {
                    // undocumented: ROT (IX+d), (IY+d),r
                    // (also stores result in a register)
                    let a = self.addr_d(d as Word, ext);
                    let v = self.device_memory.lock().unwrap().read_byte(a);
                    let w = self.rot(y, v);
                    self.registers.set_reg8_by_index(z, w);
                    self.device_memory.lock().unwrap().write_byte(a, w);
                    15
                } else {
                    // ROT r
                    let v = self.registers.get_reg8_by_index(z);
                    let w = self.rot(y, v);
                    self.registers.set_reg8_by_index(z, w);
                    8
                }
            }
            1 => {
                // BIT n
                if z == 6 {
                    // BIT n,(HL); BIT n,(IX+d); BIT n,(IY+d)
                    let a = self.addr_d(d as Word, ext);
                    let v = self.device_memory.lock().unwrap().read_byte(a);
                    self.ibit(v, 1 << y);
                    12
                } else {
                    // BIT n,r
                    let v = self.registers.get_reg8_by_index(z);
                    self.bit(v, 1 << y);
                    8
                }
            }
            2 => {
                // RES n
                if z == 6 {
                    // RES n,(HL); RES n,(IX+d); RES n,(IY+d)
                    let a = self.addr_d(d.into(), ext);
                    let v = self.device_memory.lock().unwrap().read_byte(a) & !(1 << y);
                    self.device_memory.lock().unwrap().write_byte(a, v);
                    15
                } else if ext {
                    // RES n,(IX+d),r; RES n,(IY+d),r
                    // (also stores result in a register)
                    let a = self.addr_d(d.into(), ext);
                    let v = self.device_memory.lock().unwrap().read_byte(a) & !(1 << y);
                    self.registers.set_reg8_by_index(z, v);
                    self.device_memory.lock().unwrap().write_byte(a, v);
                    15
                } else {
                    // RES n,r
                    let v = self.registers.get_reg8_by_index(z) & !(1 << y);
                    self.registers.set_reg8_by_index(z, v);
                    8
                }
            }
            3 => {
                // SET n
                if z == 6 {
                    // SET n,(HL); SET n,(IX+d); SET n,(IY+d)
                    let a = self.addr_d(d.into(), ext);
                    let v = self.device_memory.lock().unwrap().read_byte(a) | 1 << y;
                    self.device_memory.lock().unwrap().write_byte(a, v);
                    15
                } else if ext {
                    // SET n,(IX+d),r; SET n,(IY+d),r
                    // (also stores result in a register)
                    let a = self.addr_d(d.into(), ext);
                    let v = self.device_memory.lock().unwrap().read_byte(a) | 1 << y;
                    self.registers.set_reg8_by_index(z, v);
                    self.device_memory.lock().unwrap().write_byte(a, v);
                    15
                } else {
                    // SET n,r
                    let v = self.registers.get_reg8_by_index(z) | 1 << y;
                    self.registers.set_reg8_by_index(z, v);
                    8
                }
            }
            _ => unreachable!(),
        }
    }

    /// Gets the opcode at the PC and incriments it by one
    fn step_opcode(&mut self) -> Byte {
        let mem = self.device_memory.lock().unwrap();

        let opcode = mem.read_byte_forced(self.registers.reg_pc.value());
        self.registers.reg_pc.incriment();

        // Return opcode
        opcode
    }

    fn halt(&mut self) {
        self.halted = true;
        self.registers.dec_pc(1);
    }

    fn alu8(&mut self, alu: Byte, val: Byte) {
        match alu {
            0 => self.add8(val),
            1 => self.adc8(val),
            2 => self.sub8(val),
            3 => self.sbc8(val),
            4 => self.and8(val),
            5 => self.xor8(val),
            6 => self.or8(val),
            7 => self.cp8(val),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn add8(&mut self, add: Byte) {
        let acc = self.registers.val_a();
        let res = acc + add;
        self.registers.set_f(flags_add(acc, add, res));
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn adc8(&mut self, add: Byte) {
        let acc = self.registers.val_a();
        let res = acc + add + (self.registers.val_f() & CF);
        self.registers.set_f(flags_add(acc, add, res));
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn sub8(&mut self, sub: Byte) {
        let acc = self.registers.val_a();
        let res = acc - sub;
        self.registers.set_f(flags_sub(acc, sub, res));
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn sbc8(&mut self, sub: Byte) {
        let acc = self.registers.val_a();
        let res = acc - sub - (self.registers.val_f() & CF);
        self.registers.set_f(flags_sub(acc, sub, res));
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn cp8(&mut self, sub: Byte) {
        let acc = self.registers.val_a();
        let res = acc - sub;
        self.registers.set_f(flags_cp(acc, sub, res));
    }

    #[inline(always)]
    pub fn neg8(&mut self) {
        let sub = self.registers.val_a();
        self.registers.set_a(0);
        self.sub8(sub);
    }

    #[inline(always)]
    pub fn and8(&mut self, val: Byte) {
        let res = self.registers.val_a() & val;
        self.registers.set_a(res);
        self.registers.set_f(flags_szp(res) | HF);
    }

    #[inline(always)]
    pub fn or8(&mut self, val: Byte) {
        let res = self.registers.val_a() | val;
        self.registers.set_a(res);
        self.registers.set_f(flags_szp(res));
    }

    #[inline(always)]
    pub fn xor8(&mut self, val: Byte) {
        let res = self.registers.val_a() ^ val;
        self.registers.set_a(res);
        self.registers.set_f(flags_szp(res));
    }

    fn load_addr(&mut self, extention: bool) -> Word {
        if extention {
            let addr = (self.registers.get_16b_sp_reg(2) + self.d() as Word) & 0xFFFF;
            self.registers.set_wz(addr);
            addr
        } else {
            self.registers.val_hl()
        }
    }

    #[inline(always)]
    pub fn add16(&mut self, acc: Word, add: Word) -> Word {
        self.registers.set_wz(acc + 1);
        let res = acc + add;
        let f = (self.registers.val_f() & (SF | ZF | VF))
            | (((acc ^ res ^ add) >> 8) as Byte & HF)
            | (((res as u32) >> 16) as Byte & CF)
            | ((res >> 8) as Byte & (YF | XF));
        self.registers.set_f(f);
        res & 0xFFFF
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn inc8(&mut self, val: Byte) -> Byte {
        let res = (val + 1) & 0xFF;
        let f = (if res == 0 {ZF} else {res & SF}) |
            (res & (XF | YF)) | ((res ^ val) & HF) |
            (if res == 0x80 {VF} else {0}) |
            (self.registers.val_f() & CF);
        self.registers.set_f(f);
        res
    }

    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn dec8(&mut self, val: Byte) -> Byte {
        let res = (val - 1) & 0xFF;
        let f = NF | (if res == 0 {ZF} else {res & SF}) |
            (res & (XF | YF)) | ((res ^ val) & HF) |
            (if res == 0x7F {VF} else {0}) |
            (self.registers.val_f() & CF);
        self.registers.set_f(f);
        res
    }

    /// load 8-bit unsigned immediate operand and increment PC
    #[inline(always)]
    fn imm8(&mut self) -> Byte {
        let pc = self.registers.val_pc();
        let imm = self.device_memory.lock().unwrap().read_byte(pc);
        self.registers.inc_pc(1);
        imm
    }

    #[inline(always)]
    pub fn rlca8(&mut self) {
        let acc = self.registers.val_a();
        let res = (acc << 1 | acc >> 7) & 0xFF;
        let f = ((acc >> 7) & CF) | (res & (XF | YF)) | (self.registers.val_f() & (SF | ZF | PF));
        self.registers.set_f(f);
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn rrca8(&mut self) {
        let acc = self.registers.val_a();
        let res = (acc >> 1 | acc << 7) & 0xFF;
        let f = (acc & CF) | (res & (XF | YF)) | (self.registers.val_f() & (SF | ZF | PF));
        self.registers.set_f(f);
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn rl8(&mut self, val: Byte) -> Byte {
        let res = (val << 1 | (self.registers.val_f() & CF)) & 0xFF;
        self.registers.set_f(flags_szp(res) | ((val >> 7) & CF));
        res
    }

    #[inline(always)]
    pub fn rla8(&mut self) {
        let acc = self.registers.val_a();
        let f = self.registers.val_f();
        let res = (acc << 1 | (f & CF)) & 0xFF;
        self.registers
            .set_f(((acc >> 7) & CF) | (res & (XF | YF)) | (f & (SF | ZF | PF)));
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn rr8(&mut self, val: Byte) -> Byte {
        let res = (val >> 1 | (self.registers.val_f() & CF) << 7) & 0xFF;
        self.registers.set_f(flags_szp(res) | (val & CF));
        res
    }

    #[inline(always)]
    pub fn rra8(&mut self) {
        let acc = self.registers.val_a();
        let f = self.registers.val_f();
        let res = (acc >> 1 | (f & CF) << 7) & 0xFF;
        self.registers
            .set_f((acc & CF) | (res & (XF | YF)) | (f & (SF | ZF | PF)));
        self.registers.set_a(res);
    }

    #[inline(always)]
    pub fn sla8(&mut self, val: Byte) -> Byte {
        let res = (val << 1) & 0xFF;
        self.registers.set_f(flags_szp(res) | (val >> 7 & CF));
        res
    }

    #[inline(always)]
    pub fn sll8(&mut self, val: Byte) -> Byte {
        // undocumented, sll8 is identical with sla8, but shifts a 1 into LSB
        let res = (val << 1 | 1) & 0xFF;
        self.registers.set_f(flags_szp(res) | (val >> 7 & CF));
        res
    }

    #[inline(always)]
    pub fn sra8(&mut self, val: Byte) -> Byte {
        let res = (val >> 1 | (val & 0x80)) & 0xFF;
        self.registers.set_f(flags_szp(res) | (val & CF));
        res
    }

    #[inline(always)]
    pub fn srl8(&mut self, val: Byte) -> Byte {
        let res = val >> 1 & 0xFF;
        self.registers.set_f(flags_szp(res) | (val & CF));
        res
    }

    #[inline(always)]
    pub fn rld(&mut self) {
        let addr = self.registers.val_hl();
        let v = self.device_memory.lock().unwrap().read_byte(addr);
        let ah = self.registers.val_a() & 0xF0;
        let al = self.registers.val_a() & 0x0F;
        let a = ah | (v >> 4 & 0x0F);
        self.registers.set_a(a);
        self.device_memory
            .lock()
            .unwrap()
            .write_byte(addr, (v << 4 | al) & 0xFF);
        self.registers.set_wz(addr + 1);
        let f = flags_szp(a) | (self.registers.val_f() & CF);
        self.registers.set_f(f);
    }

    #[inline(always)]
    pub fn rrd(&mut self) {
        let addr = self.registers.val_hl();
        let v = self.device_memory.lock().unwrap().read_byte(addr);
        let ah = self.registers.val_a() & 0xF0;
        let al = self.registers.val_a() & 0x0F;
        let a = ah | (v & 0x0F);
        self.registers.set_a(a);
        self.device_memory
            .lock()
            .unwrap()
            .write_byte(addr, (v >> 4 | al << 4) & 0xFF);
        self.registers.set_wz(addr + 1);
        let f = flags_szp(a) | (self.registers.val_f() & CF);
        self.registers.set_f(f);
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn bit(&mut self, val: Byte, mask: Byte) {
        let res = val & mask;
        let f = HF | (self.registers.val_f() & CF) | (if res == 0 {ZF | PF} else {res & SF}) |
            (val & (XF | YF));
        self.registers.set_f(f)
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn daa(&mut self) {
        let a = self.registers.val_a();
        let mut val = a;
        let f = self.registers.val_f();
        if 0 != (f & NF) {
            if ((a & 0xF) > 0x9) || (0 != (f & HF)) {
                val = (val - 0x06) & 0xFF;
            }
            if (a > 0x99) || (0 != (f & CF)) {
                val = (val - 0x60) & 0xFF;
            }
        } else {
            if ((a & 0xF) > 0x9) || (0 != (f & HF)) {
                val = (val + 0x06) & 0xFF;
            }
            if (a > 0x99) || (0 != (f & CF)) {
                val = (val + 0x60) & 0xFF;
            }
        }
        self.registers.set_f((f & (CF | NF)) |
                       (if a > 0x99 {CF} else {0}) |
                       ((a ^ val) & HF) | flags_szp(val));
        self.registers.set_a(val);
    }

    #[inline(always)]
    pub fn cpl(&mut self) {
        let f = self.registers.val_f();
        let a = self.registers.val_a() ^ 0xFF;
        self.registers
            .set_f((f & (SF | ZF | PF | CF)) | (HF | NF) | (a & (YF | XF)));
        self.registers.set_a(a);
    }

    #[inline(always)]
    pub fn scf(&mut self) {
        let f = self.registers.val_f();
        let a = self.registers.val_a();
        self.registers
            .set_f((f & (SF | ZF | YF | XF | PF)) | CF | (a & (YF | XF)));
    }

    #[inline(always)]
    pub fn ccf(&mut self) {
        let f = self.registers.val_f();
        let a = self.registers.val_a();
        self.registers
            .set_f(((f & (SF | ZF | YF | XF | PF | CF)) | ((f & CF) << 4) | (a & (YF | XF))) ^ CF);
    }

    #[inline(always)]
    pub fn ret(&mut self) -> i64 {
        let sp = self.registers.val_sp();
        let wz = self.device_memory.lock().unwrap().read_word(sp);
        self.registers.set_wz(wz);
        self.registers.set_pc(wz);
        self.registers.set_sp(sp + 2);
        10
    }

    #[inline(always)]
    pub fn call(&mut self) -> i64 {
        let wz = self.imm16();
        let sp = (self.registers.val_sp() - 2) & 0xFFFF;
        self.device_memory
            .lock()
            .unwrap()
            .write_word(sp, self.registers.val_pc());
        self.registers.set_sp(sp);
        self.registers.set_wz(wz);
        self.registers.set_pc(wz);
        17
    }

    #[inline(always)]
    pub fn retcc(&mut self, y: Byte) -> i64 {
        if self.check_condition(y) {
            self.ret() + 1
        } else {
            5
        }
    }

    #[inline(always)]
    pub fn callcc(&mut self, y: Byte) -> i64 {
        if self.check_condition(y) {
            self.call()
        } else {
            let wz = self.imm16();
            self.registers.set_wz(wz);
            10
        }
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn ldi(&mut self) {
        let hl = self.registers.val_hl();
        let de = self.registers.val_de();
        let val = self.device_memory.lock().unwrap().read_byte(hl);
        self.device_memory.lock().unwrap().write_byte(de, val);
        self.registers.set_hl(hl + 1);
        self.registers.set_de(de + 1);
        let bc = (self.registers.val_bc() - 1) & 0xFFFF;
        self.registers.set_bc(bc);
        let n = (val + self.registers.val_a()) & 0xFF;
        let f = (self.registers.val_f() & (SF | ZF | CF)) |
                (if (n & 0x02) != 0 {YF} else {0}) |
                (if (n & 0x08) != 0 {XF} else {0}) |
                (if bc > 0 {VF} else {0});
        self.registers.set_f(f);
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn ldd(&mut self) {
        let hl = self.registers.val_hl();
        let de = self.registers.val_de();
        let val = self.device_memory.lock().unwrap().read_byte(hl);
        self.device_memory.lock().unwrap().write_byte(de, val);
        self.registers.set_hl(hl - 1);
        self.registers.set_de(de - 1);
        let bc = (self.registers.val_bc() - 1) & 0xFFFF;
        self.registers.set_bc(bc);
        let n = (val + self.registers.val_a()) & 0xFF;
        let f = (self.registers.val_f() & (SF | ZF | CF)) |
                (if (n & 0x02) != 0 {YF} else {0}) |
                (if (n & 0x08) != 0 {XF} else {0}) |
                (if bc > 0 {VF} else {0});
        self.registers.set_f(f);
    }

    #[inline(always)]
    pub fn ldir(&mut self) -> i64 {
        self.ldi();
        if (self.registers.val_f() & VF) != 0 {
            let pc = self.registers.val_pc();
            self.registers.dec_pc(2);
            self.registers.set_wz(pc + 1);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    pub fn lddr(&mut self) -> i64 {
        self.ldd();
        if (self.registers.val_f() & VF) != 0 {
            let pc = self.registers.val_pc();
            self.registers.dec_pc(2);
            self.registers.set_wz(pc + 1);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn cpi(&mut self) {
        let wz = self.registers.val_wz();
        self.registers.set_wz(wz + 1);
        let hl = self.registers.val_hl();
        self.registers.set_hl(hl + 1);
        let bc = (self.registers.val_bc() - 1) & 0xFFFF;
        self.registers.set_bc(bc);
        let a = self.registers.val_a();
        let mut v = a - self.device_memory.lock().unwrap().read_byte(hl);
        let mut f = NF | (self.registers.val_f() & CF) |
                    (if v == 0 {ZF} else {v & SF}) |
                    (if (v & 0xF) > (a & 0xF) {HF} else {0}) |
                    (if bc != 0 {VF} else {0});
        if (f & HF) != 0 {
            v -= 1;
        }
        if (v & 0x02) != 0 {
            f |= YF
        };
        if (v & 0x08) != 0 {
            f |= XF
        };
        self.registers.set_f(f);
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn cpd(&mut self) {
        let wz = self.registers.val_wz();
        self.registers.set_wz(wz - 1);
        let hl = self.registers.val_hl();
        self.registers.set_hl(hl - 1);
        let bc = (self.registers.val_bc() - 1) & 0xFFFF;
        self.registers.set_bc(bc);
        let a = self.registers.val_a();
        let mut v = a - self.device_memory.lock().unwrap().read_byte(hl);
        let mut f = NF | (self.registers.val_f() & CF) |
                    (if v == 0 {ZF} else {v & SF}) |
                    (if (v & 0xF) > (a & 0xF) {HF} else {0}) |
                    (if bc != 0 {VF} else {0});
        if (f & HF) != 0 {
            v -= 1;
        }
        if (v & 0x02) != 0 {
            f |= YF
        };
        if (v & 0x08) != 0 {
            f |= XF
        };
        self.registers.set_f(f);
    }

    #[inline(always)]
    pub fn cpir(&mut self) -> i64 {
        self.cpi();
        if (self.registers.val_f() & (VF | ZF)) == VF {
            let pc = self.registers.val_pc();
            self.registers.dec_pc(2);
            self.registers.set_wz(pc + 1);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    pub fn cpdr(&mut self) -> i64 {
        self.cpd();
        if (self.registers.val_f() & (VF | ZF)) == VF {
            let pc = self.registers.val_pc();
            self.registers.dec_pc(2);
            self.registers.set_wz(pc + 1);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    pub fn outp(&mut self, _port: Word, _val: Byte) {
        // bus.cpu_outp(port, val); TODO: Enable port output
    }
    #[inline(always)]
    pub fn inp(&mut self, _port: Word) -> Byte {
        //bus.cpu_inp(port) & 0xFF // TODO: Enable port input
        1
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn ini_ind_flags(&self, val: Byte, add: Byte) -> Byte {
        let b = self.registers.val_b();
        let c = self.registers.val_c();
        let t: u16 = (((c + add) & 0xFF)+ val) as u16;
        (if b != 0 {b & SF} else {ZF}) |
            (if (val & SF) != 0 {NF} else {0}) |
            (if (t & 0x100) != 0 {HF | CF} else {0}) |
            (flags_szp((t & 0x07) as u8 ^ b) & PF)
    }

    #[inline(always)]
    pub fn ini(&mut self) {
        let bc = self.registers.val_bc();
        // let io_val = self.inp(bus, bc); TODO: Enable Port Input
        self.registers.set_wz(bc + 1);
        let b = self.registers.val_b();
        self.registers.set_b(b - 1);
        let hl = self.registers.val_hl();
        //self.device_memory.lock().unwrap().write_byte(hl, io_val);
        self.registers.set_hl(hl + 1);
        // let f = self.ini_ind_flags(io_val, 1);
        //self.registers.set_f(f);
    }

    #[inline(always)]
    pub fn ind(&mut self) {
        let bc = self.registers.val_bc();
        // let io_val = self.inp(bus, bc); TODO: Enable port input
        self.registers.set_wz(bc - 1);
        let b = self.registers.val_b();
        self.registers.set_b(b - 1);
        let hl = self.registers.val_hl();
        // self.device_memory.lock().unwrap().write_byte(hl, io_val);
        self.registers.set_hl(hl - 1);
        //let f = self.ini_ind_flags(io_val, Byte::MAX);
        //self.registers.set_f(f);
    }

    #[inline(always)]
    pub fn inir(&mut self) -> i64 {
        self.ini();
        if self.registers.val_b() != 0 {
            self.registers.dec_pc(2);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    pub fn indr(&mut self) -> i64 {
        self.ind();
        if self.registers.val_b() != 0 {
            self.registers.dec_pc(2);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    pub fn outi(&mut self) {
        let hl = self.registers.val_hl();
        let io_val = self.device_memory.lock().unwrap().read_byte(hl);
        self.registers.set_hl(hl + 1);
        let b = self.registers.val_b();
        self.registers.set_b(b - 1);
        let bc = self.registers.val_bc();
        //self.outp(bus, bc, io_val); TODO: Implement port output
        self.registers.set_wz(bc + 1);
        let f = self.outi_outd_flags(io_val);
        self.registers.set_f(f);
    }

    #[inline(always)]
    pub fn outd(&mut self) {
        let hl = self.registers.val_hl();
        let io_val = self.device_memory.lock().unwrap().read_byte(hl);
        self.registers.set_hl(hl - 1);
        let b = self.registers.val_b();
        self.registers.set_b(b - 1);
        let bc = self.registers.val_bc();
        // self.outp(bus, bc, io_val); TODO: Enable port output
        self.registers.set_wz(bc - 1);
        let f = self.outi_outd_flags(io_val);
        self.registers.set_f(f);
    }

    #[inline(always)]
    pub fn otir(&mut self) -> i64 {
        self.outi();
        if self.registers.val_b() != 0 {
            self.registers.dec_pc(2);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    pub fn otdr(&mut self) -> i64 {
        self.outd();
        if self.registers.val_b() != 0 {
            self.registers.dec_pc(2);
            21
        } else {
            16
        }
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn outi_outd_flags(&self, val: Byte) -> Byte {
        let b = self.registers.val_b();
        let l = self.registers.val_l();
        let t:u16 = (l + val) as u16;
        (if b != 0 {b & SF} else {ZF}) |
            (if (val & SF) != 0 {NF} else {0}) |
            (if (t & 0x100) != 0 {HF | CF} else {0}) |
            (flags_szp((t & 0x07) as u8 ^ b) & PF)
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn adc16(&mut self, acc: Word, add: Word) -> Word {
        self.registers.set_wz(acc + 1);
        let res = acc + add + (self.registers.val_f() & CF) as Word;
        self.registers.set_f(((((acc ^ res ^ add) >> 8) & HF as Word) | ((((res as u32) >> 16) as Word) & CF as Word) |
                       ((res >> 8) & (SF | XF | YF) as Word) |
                       (if (res & 0xFFFF) == 0 {ZF as Word} else {0}) |
                       (((add ^ acc ^ 0x8000) & (add ^ res) & 0x8000)) >> 13) as Byte);
        res & 0xFFFF
    }

    #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn sbc16(&mut self, acc: Word, sub: Word) -> Word {
        self.registers.set_wz(acc + 1);
        let res = acc - sub - (self.registers.val_f() & CF) as Word;
        self.registers.set_f(NF | ((((acc ^ res ^ sub) >> 8) & HF as Word) | ((((res as u32) >> 16) as Word) & CF as Word) |
                       ((res >> 8) & (SF | XF | YF) as Word) |
                       (if (res & 0xFFFF) == 0 {ZF as Word} else {0}) |
                       (((sub ^ acc) & (acc ^ res) & 0x8000) >> 13))as Byte);
        res & 0xFFFF
    }

    #[inline(always)]
    pub fn rot(&mut self, op: Byte, val: Byte) -> Byte {
        match op {
            0 => self.rlc8(val),
            1 => self.rrc8(val),
            2 => self.rl8(val),
            3 => self.rr8(val),
            4 => self.sla8(val),
            5 => self.sra8(val),
            6 => self.sll8(val),
            7 => self.srl8(val),
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn rlc8(&mut self, val: Byte) -> Byte {
        let res = (val << 1 | val >> 7) & 0xFF;
        self.registers.set_f(flags_szp(res) | ((val >> 7) & CF));
        res
    }

    #[inline(always)]
    pub fn rrc8(&mut self, val: Byte) -> Byte {
        let res = (val >> 1 | val << 7) & 0xFF;
        self.registers.set_f(flags_szp(res) | (val & CF));
        res
    }

     #[inline(always)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    pub fn ibit(&mut self, val: Byte, mask: Byte) {
    // special version of the BIT instruction for
    // (HL), (IX+d), (IY+d) to set the undocumented XF|YF flags
    // from high byte of HL+1 or IX/IY+d (expected in WZ)
        let res = val & mask;
        let f = HF | (self.registers.val_f() & CF) | (if res == 0 {ZF | PF} else {res & SF}) |
            (self.registers.val_w() & (XF | YF));
        self.registers.set_f(f)
    }
}

/// Memory wrapper class that implements functions to update and run timers
pub struct Timer {
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
        self.do_divider_registers(cycles);

        //the clock must be enabled to update itself
        if self.is_clock_enabled() {
            let mut mem = self.mem.lock().unwrap();
            mem.timer_counter -= cycles;

            // enough cpu cycled have happened to update the timer
            if mem.timer_counter <= 0 {
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

    fn do_divider_registers(&mut self, cycles: i32) {
        self.divider_counter += cycles as u32;
        if self.divider_counter >= 255 {
            self.divider_counter = 0;

            let mut mem = self.mem.lock().unwrap();
            // 0xFF04 is the location of the divider register
            debug_println!("Force reading from divider reg");
            let divider_register = mem.read_byte_forced(DIVIDER_REGISTER).wrapping_add(1);
            debug_println!("Divider Register Value: {:X}", divider_register);
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

#[cfg_attr(rustfmt, rustfmt_skip)]
fn flags_add(acc: Byte, add: Byte, res: Byte) -> Byte {
    (if (res & 0xFF) == 0 {ZF} else {res & SF}) |
    (res & (YF | XF)) | ((((res as Word) >> 8) as Byte) & CF) |
    ((acc ^ add ^ res) & HF) | ((((acc ^ add ^ 0x80) & (add ^ res)) >> 5) & VF)
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn flags_sub(acc: Byte, sub: Byte, res: Byte) -> Byte {
    NF | (if (res & 0xFF) == 0 {ZF} else {res & SF}) |
    (res & (YF | XF)) | ((((res as Word) >> 8) as Byte) & CF) |
    ((acc ^ sub ^ res) & HF) | ((((acc ^ sub) & (res ^ acc)) >> 5) & VF)
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn flags_cp(acc: Byte, sub: Byte, res: Byte) -> Byte {
    // the only difference to flags_sub() is that the
    // 2 undocumented flag bits X and Y are taken from the
    // sub-value, not the result
    NF | (if (res & 0xFF) == 0 {ZF} else {res & SF}) |
    (sub & (YF | XF)) | ((((res as Word) >> 8) as Byte) & CF) |
    ((acc ^ sub ^ res) & HF) | ((((acc ^ sub) & (res ^ acc)) >> 5) & VF)
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn flags_szp(val: Byte) -> Byte {
    let v = val & 0xFF;
    (if (v.count_ones() & 1) == 0 {PF} else {0}) |
    (if v == 0 {ZF} else {v & SF}) | (v & (YF | XF))
}

#[inline(always)]
fn flags_sziff2(val: Byte, iff2: bool) -> Byte {
    (if (val & 0xFF) == 0 { ZF } else { val & SF }) | (val & (YF | XF)) | if iff2 { PF } else { 0 }
}

#[cfg(test)]
mod test {

    use super::*;
    use ntest::timeout;

    #[test]
    #[timeout(10)]
    fn test_cpu_init() {
        let mem = Arc::new(Mutex::new(Memory::new()));

        let cpu = CPU::new(mem);

        assert_eq!(cpu.cycles, 0);
    }

    #[test]
    #[timeout(10)]
    fn test_cpu_interrupts() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        let mut cpu = CPU::new(mem);

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
    #[timeout(10)]
    fn test_push_pop_stack() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        let mut cpu = CPU::new(mem);

        cpu.push_stack(cpu.registers.reg_af.value());
        assert_eq!(cpu.registers.reg_sp.value(), 0xFFFC);

        let ret = cpu.pop_stack();
        assert_eq!(ret, 0x01B0);
        assert_eq!(cpu.registers.reg_sp.value(), 0xFFFE);
    }

    #[test]
    #[timeout(10)]
    fn test_timer_increment() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        {
            let mut m = mem.lock().unwrap();
            m.write_byte(TMC, 0x5); // enable timer, freq select 1
            m.set_clock_frequency();
            m.timer_counter = 0; // force immediate update
        }
        let mut timer = Timer::new(Arc::clone(&mem));
        timer.update_timers(16);
        let m = mem.lock().unwrap();
        assert_eq!(m.read_byte(TIMA), 1);
        assert_eq!(m.read_byte(IF), 0);
    }

    #[test]
    #[timeout(10)]
    fn test_timer_overflow() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        {
            let mut m = mem.lock().unwrap();
            m.write_byte(TMC, 0x5); // enable timer, freq select 1
            m.write_byte(TIMA, 255);
            m.write_byte(TMA, 7);
            m.set_clock_frequency();
            m.timer_counter = 0; // force immediate update
        }
        let mut timer = Timer::new(Arc::clone(&mem));
        timer.update_timers(16);
        let m = mem.lock().unwrap();
        assert_eq!(m.read_byte(TIMA), 7);
        assert_eq!(m.read_byte(IF) & 0x4, 0x4);
    }

    #[test]
    #[timeout(10)]
    fn test_divider_register_increment() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        let mut timer = Timer::new(Arc::clone(&mem));
        timer.update_timers(255);
        let m = mem.lock().unwrap();
        assert_eq!(m.read_byte_forced(DIVIDER_REGISTER), 1);
    }

    #[test]
    #[timeout(10)]
    fn test_register_operations() {
        let mut reg = registers::Register::new(0x1234);
        assert_eq!(reg.value(), 0x1234);
        assert_eq!(reg.high_value(), 0x12);
        assert_eq!(reg.low_value(), 0x34);

        reg.incriment();
        assert_eq!(reg.value(), 0x1235);

        reg.decriment();
        reg.decriment();
        assert_eq!(reg.value(), 0x1233);

        reg.set(0xABCD);
        assert_eq!(reg.value(), 0xABCD);
    }

    #[test]
    #[timeout(10)]
    fn test_cpu_reset() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        let mut cpu = CPU::new(Arc::clone(&mem));

        cpu.cycles = 42;
        cpu.ime = false;
        cpu.registers.reg_af.set(0xFFFF);

        let old_mem = Arc::clone(&cpu.device_memory);
        cpu.reset();

        assert_eq!(cpu.cycles, 0);
        assert!(!cpu.ime);
        assert_eq!(cpu.registers.reg_af.value(), 0x01B0);
        assert!(!Arc::ptr_eq(&old_mem, &cpu.device_memory));
        let new_mem = cpu.device_memory.lock().unwrap();
        assert_eq!(new_mem.read_byte(TIMA), 0);
    }

    #[test]
    #[timeout(10)]
    fn test_timer_disabled_no_increment() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        {
            let mut m = mem.lock().unwrap();
            m.write_byte(TMC, 0x0); // timer disabled
            m.set_clock_frequency();
            m.timer_counter = 0;
        }
        let mut timer = Timer::new(Arc::clone(&mem));
        timer.update_timers(16);
        let m = mem.lock().unwrap();
        assert_eq!(m.read_byte(TIMA), 0);
    }
    #[test]
    #[timeout(10)]
    fn test_serial_interrupt_vector() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        let mut cpu = CPU::new(Arc::clone(&mem));

        {
            let mut m = mem.lock().unwrap();
            m.request_interrupt(3);
            m.enable_interrupt(3);
            assert_eq!(m.read_byte(IF), 1 << 3);
        }

        cpu.handle_interrupts();

        {
            let m = cpu.device_memory.lock().unwrap();
            assert_eq!(m.read_byte(IF), 0);
        }

        assert_eq!(cpu.registers.reg_pc.value(), 0x58);
        let ret = cpu.pop_stack();
        assert_eq!(ret, 0x100);
    }

    #[test]
    #[timeout(10)]
    fn test_timer_update() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        let mut timer = Timer::new(Arc::clone(&mem));

        {
            let mut m = mem.lock().unwrap();
            m.ram_startup();
            m.write_byte(TMC, 0x04); // enable timer, freq = 4096
            m.write_byte(TIMA, 0x00);
            m.set_clock_frequency();
        }

        timer.update_timers(1024);

        let m = mem.lock().unwrap();
        assert_eq!(m.read_byte(TIMA), 1);
        assert!(m.timer_counter > 0);
    }
}
