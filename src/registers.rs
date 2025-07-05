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

    pub reg_ix: Register, // index addressing mode register
    pub reg_iy: Register, // index addressing mode register

    pub reg_af_c: Register, // compliment register of af
    pub reg_bc_c: Register, // compliment register of bc
    pub reg_de_c: Register, // compliment register of de
    pub reg_hl_c: Register, // compliment register of hl

    pub reg_wz: Register,   // Undocumented register
    pub reg_wz_c: Register, // compliment register of wz
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
            reg_pc: Register { reg: 0x0100 },
            reg_ix: Register { reg: 0x0000 },
            reg_iy: Register { reg: 0x0000 },
            reg_af_c: Register { reg: 0x0000 },
            reg_bc_c: Register { reg: 0x0000 },
            reg_de_c: Register { reg: 0x0000 },
            reg_hl_c: Register { reg: 0x0000 },
            reg_wz: Register { reg: 0x0000 },
            reg_wz_c: Register { reg: 0x0000 },
        }
    }

    /// Gets an 8 bit register by a basic index
    pub fn get_reg8_by_index(&self, index: Byte) -> Byte {
        match index {
            0 => self.reg_bc.high_value(), // b
            1 => self.reg_bc.low_value(),  // c
            2 => self.reg_de.high_value(), // d
            3 => self.reg_de.low_value(),  // e
            4 => self.reg_hl.high_value(), // h
            5 => self.reg_hl.low_value(),  // l
            6 => self.reg_af.high_value(),  // f
            7 => self.reg_af.low_value(), // a
            _ => 0,
        }
    }

    pub fn set_reg8_by_index(&mut self, index: Byte, val: Byte) {
        match index {
            0 => self.set_b(val),
            1 => self.set_c(val),
            2 => self.set_d(val),
            3 => self.set_e(val),
            4 => self.set_h(val),
            5 => self.set_l(val),
            6 => self.set_a(val),
            7 => self.set_f(val),
            _ => panic!("Invalid index found: u8 set"),
        };
    }

    /// Gets an 8 bit register by a basic index
    pub fn get_reg16_by_index(&self, index: Byte) -> Word {
        match index {
            0 => self.reg_bc.value(), // bc
            2 => self.reg_de.value(), // de
            4 => self.reg_hl.value(), // hl
            6 => self.reg_af.value(),  // af
            8 => self.reg_ix.value(), // ix
            10 => self.reg_iy.value(), // iy
            12 => self.reg_sp.value(), // sp
            14 => self.reg_wz.value(),  // wz
            16 => self.reg_bc_c.value(), // bc_c
            18 => self.reg_de_c.value(), // de_c
            20 => self.reg_hl_c.value(), // hl_c
            22 => self.reg_af_c.value(),  // af_c
            24 => self.reg_wz_c.value(), //wz_c
            _ => 0,
        }
    }

    pub fn set_reg16_by_index(&mut self, index: Byte, val: Word) {
        match index {
            0 => self.reg_bc.set(val), // bc
            2 => self.reg_de.set(val), // de
            4 => self.reg_hl.set(val), // hl
            6 => self.reg_af.set(val),  // af
            8 => self.reg_ix.set(val), // ix
            10 => self.reg_iy.set(val), // iy
            12 => self.reg_sp.set(val), // sp
            14 => self.reg_wz.set(val),  // wz
            16 => self.reg_bc_c.set(val), // bc_c
            18 => self.reg_de_c.set(val), // de_c
            20 => self.reg_hl_c.set(val), // hl_c
            22 => self.reg_af_c.set(val),  // af_c
            24 => self.reg_wz_c.set(val), //wz_c
            _ => panic!("Invalid index found"),
        };
    }

    pub fn get_16b_sp_reg(&self, index: Byte) -> Word {
        match index {
            0 => self.reg_bc.value(),
            1 => self.reg_de.value(),
            2 => self.reg_hl.value(),
            3 => self.reg_sp.value(),
            _ => 0,
        }
    }

    pub fn swap(&mut self, reg_1: Byte, reg_2: Byte) {
        let v = self.get_16b_sp_reg(reg_1);
        let v_ = self.get_16b_sp_reg(reg_2);
        self.set_reg16_by_index(reg_1, v_);
        self.set_reg16_by_index(reg_2, v);
    }

    /// Sets the contents of the f register
    pub fn set_f(&mut self, val: Byte) {
        self.reg_af.bitspace.lo = val;
    }

    /// Gets the contents of the f register
    pub fn val_f(&self) -> Byte {
        unsafe { self.reg_af.bitspace.lo }
    }

    /// Sets the contents of the a register
    pub fn set_a(&mut self, val: Byte) {
        self.reg_af.bitspace.hi = val;
    }

    /// Gets the contents of the a register
    pub fn val_a(&self) -> Byte {
        unsafe { self.reg_af.bitspace.hi }
    }

    /// Sets the contents of the b register
    pub fn set_b(&mut self, val: Byte) {
        self.reg_bc.bitspace.hi = val;
    }

    /// Gets the contents of the b register
    pub fn val_b(&self) -> Byte {
        unsafe { self.reg_bc.bitspace.hi }
    }

    /// Sets the contents of the c register
    pub fn set_c(&mut self, val: Byte) {
        self.reg_bc.bitspace.lo = val;
    }

    /// Gets the contents of the c register
    pub fn val_c(&self) -> Byte {
        unsafe { self.reg_bc.bitspace.lo }
    }

    /// Sets the contents of the d register
    pub fn set_d(&mut self, val: Byte) {
        self.reg_de.bitspace.hi = val;
    }

    /// Gets the contents of the d register
    pub fn val_d(&self) -> Byte {
        unsafe { self.reg_de.bitspace.hi }
    }

    /// Sets the contents of the e register
    pub fn set_e(&mut self, val: Byte) {
        self.reg_de.bitspace.lo = val;
    }

    /// Gets the contents of the e register
    pub fn val_e(&self) -> Byte {
        unsafe { self.reg_de.bitspace.lo }
    }

    /// Sets the contents of the h register
    pub fn set_h(&mut self, val: Byte) {
        self.reg_hl.bitspace.hi = val;
    }

    /// Gets the contents of the h register
    pub fn val_h(&self) -> Byte {
        unsafe { self.reg_hl.bitspace.hi }
    }

    /// Sets the contents of the l register
    pub fn set_l(&mut self, val: Byte) {
        self.reg_de.bitspace.lo = val;
    }

    /// Gets the contents of the l register
    pub fn val_l(&self) -> Byte {
        unsafe { self.reg_de.bitspace.lo }
    }

    /// Gets the contents of the af register
    pub fn val_af(&mut self) -> Word {
        unsafe {self.reg_af.reg}
    }

    /// Sets the contents of the af register
    pub fn set_af(&mut self, val: Word) {
        self.reg_af.reg = val;
    }

    /// Gets the contents of the bc register
    pub fn val_bc(&mut self) -> Word {
        unsafe {self.reg_bc.reg}
    }

    /// Sets the contents of the bc register
    pub fn set_bc(&mut self, val: Word) {
        self.reg_bc.reg = val;
    }

    /// Gets the contents of the de register
    pub fn val_de(&mut self) -> Word {
        unsafe {self.reg_de.reg}
    }

    /// Sets the contents of the de register
    pub fn set_de(&mut self, val: Word) {
        self.reg_de.reg = val;
    }

    /// Gets the contents of the hl register
    pub fn val_hl(&mut self) -> Word {
        unsafe {self.reg_wz.reg}
    }

    /// Sets the contents of the hl register
    pub fn set_hl(&mut self, val: Word) {
        self.reg_hl.reg = val;
    }
    
    /// Gets the contents of the wz register
    pub fn val_wz(&mut self) -> Word {
        unsafe {self.reg_wz.reg}
    }

    /// Sets the contents of the wz register
    pub fn set_wz(&mut self, val: Word) {
        self.reg_wz.reg = val;
    }

    /// Gets the contents of the pc register
    pub fn val_pc(&self) -> Word {
        unsafe { self.reg_pc.reg }
    }

    /// Sets the contents of the pc register
    pub fn set_pc(&mut self, val: Word) {
        self.reg_pc.reg = val;
    }

    /// Gets the contents of the sp register
    pub fn val_sp(&self) -> Word {
        unsafe { self.reg_sp.reg }
    }

    /// Sets the contents of the sp register
    pub fn set_sp(&mut self, val: Word) {
        self.reg_sp.reg = val;
    }


/////////////////////////////////////////////////////////

    /// set 16-bit register by 2-bit index with mapping through SP-table
    #[inline(always)]
    pub fn set_r16sp(&mut self, r: Byte, v: Word) {
        let i = SP_TABLE[r as usize];
        self.set_reg16_by_index(i, v);
    }

    /// set 16-bit register by 2-bit index with mapping through AF-table
    #[inline(always)]
    pub fn set_r16af(&mut self, r: Byte, v: Word) {
        let i = AF_TABLE[r as usize];
        self.set_reg16_by_index(i, v);
    }

    /// get 16-bit register by 2-bit index with mapping through AF-table
    #[inline(always)]
    pub fn r16af(&self, r: Byte) -> Word {
        let i = AF_TABLE[r as usize];
        self.get_reg16_by_index(i)
    }

    /// get 16-bit register by 2-bit index with mapping through SP-table
    #[inline(always)]
    pub fn r16sp(&self, r: Byte) -> Word {
        let i = SP_TABLE[r as usize];
        self.get_reg16_by_index(i)
    }


    #[inline(always)]
    pub fn dec_pc(&mut self, dec: Word) {
        unsafe {self.reg_pc.reg -= dec; }
    }

    #[inline(always)]
    pub fn inc_pc(&mut self, inc: Word) {
        unsafe {self.reg_pc.reg += inc; }
    }
}
