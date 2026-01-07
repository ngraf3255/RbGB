use std::{
    ops::BitAnd,
    sync::{Arc, Mutex},
};

/// Functions and storage for operating on device memory
use crate::types::*;
#[allow(unused_imports)]
use debug_print::debug_println;

pub type SharedMemory = Arc<Mutex<Memory>>;

pub struct Memory {
    mem: Ram,
    rom: Vec<Byte>,
    external_ram: [[Byte; 0x2000]; 4],
    rom_banking_type: RomBankingType,
    rom_banks: CurrentRomBank,
    ram_banks: CurrentRamBank,
    ram_write_enable: bool,
    rom_bank_enable: bool,
    joypad_buttons: Byte,
    joypad_directions: Byte,

    pub timer_counter: i32,
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            mem: [0; MEM_SIZE],
            rom: Vec::new(),
            external_ram: [[0; 0x2000]; 4],
            rom_banking_type: RomBankingType::None,
            rom_banks: CurrentRomBank::Bank(1),
            ram_banks: CurrentRamBank::Bank0,
            ram_write_enable: false,
            rom_bank_enable: true,
            joypad_buttons: 0x0F,
            joypad_directions: 0x0F,

            timer_counter: 1024,
        }
    }

    // Wrapper for memory read functionality
    pub fn read_byte(&self, addr: Word) -> Byte {
        self.read_byte_internal(addr)
    }

    // Wrapper for memory write functionality
    pub fn write_byte(&mut self, addr: Word, value: Byte) {
        // this is read only memory and should not be written to
        if addr < 0x8000 {
            self.handle_banking(addr, value);
        }
        //inserts value into the ram banks if enabled
        else if (0xA000..0xC000).contains(&addr) {
            if self.ram_write_enable {
                let offset = (addr - 0xA000) as usize;
                let bank = self.ram_banks as usize;
                self.external_ram[bank][offset] = value;
            }
        }
        // echo ram writes to two locations
        else if (0xE000..0xFE00).contains(&addr) {
            self.mem[addr as usize] = value;
            self.write_byte(addr - 0x2000, value);
        }
        // restricted memory area
        else if (0xFEA0..0xFEFF).contains(&addr) {
            //TODO: implement error handling here (likely throw some kind of interrupt)
        } else if TMC == addr {
            let current_frequency: Byte = self.get_clock_freq();
            self.mem[TMC as usize] = value;
            let new_frequency = self.get_clock_freq();

            if current_frequency != new_frequency {
                self.set_clock_frequency();
            }
        } else if DIVIDER_REGISTER == addr {
            // If ever writing to the divider register always set it to 0
            self.mem[DIVIDER_REGISTER as usize] = 0;
        } else if addr == CURRENT_SCANLINE {
            // If ever writing to the current scanline always set it to 0
            self.mem[CURRENT_SCANLINE as usize] = 0;
        } else if addr == INPUT_REGISTER {
            let current = self.mem[INPUT_REGISTER as usize];
            self.mem[INPUT_REGISTER as usize] = (value & 0x30) | 0xC0 | (current & 0x0F);
            self.recompute_joypad();
        } else if addr == DMA_REG {
            // Game is activating a direct memory access
            self.dma_transfer(value);
        } else {
            self.mem[addr as usize] = value;
        }
    }

    ///Wrapper for forced memory writing
    ///
    /// Be careful as there are no bounds on providing the wrong mem index
    pub fn write_byte_forced(&mut self, addr: Word, value: Byte) {
        if addr < 0x8000 {
            let index = addr as usize;
            if self.rom.len() <= index {
                self.rom.resize(index + 1, 0);
            }
            self.rom[index] = value;
        } else if (0xA000..=0xBFFF).contains(&addr) {
            let offset = (addr - 0xA000) as usize;
            let bank = self.ram_banks as usize;
            self.external_ram[bank][offset] = value;
        }
        //Sets byte
        self.mem[addr as usize] = value;
    }

    ///Function to write a word
    pub fn write_word(&mut self, addr: Word, value: Word) {
        let l = value & 0xff;
        let h = (value >> 8) & 0xff;
        self.write_byte(addr, l as Byte);
        self.write_byte(addr.wrapping_add(1), h as Byte);
    }

    ///Function to read a word
    pub fn read_word(&self, addr: Word) -> Word {
        let l = self.read_byte(addr) as Word;
        let h = self.read_byte(addr.wrapping_add(1)) as Word;
        h << 8 | l
    }

    ///Wrapper for forced memory reading
    ///
    /// Be careful as there are no bounds on providing the wrong mem index
    pub fn read_byte_forced(&self, addr: Word) -> Byte {
        self.read_byte_internal(addr)
    }

    /// Reads TMC memory location to get current clock frequency
    ///
    /// Selects bits 0 and 1 to map to a frequency value
    pub fn get_clock_freq(&self) -> Byte {
        self.read_byte(TMC) & 0x3
    }

    /// Sets the timer_counter to the current clock frequency
    pub fn set_clock_frequency(&mut self) {
        let frequency = self.get_clock_freq();

        // Magic numbers again frome from val of CLOCKSPEED / frequency
        match frequency {
            0 => self.timer_counter = 1024, // freq = 4096
            1 => self.timer_counter = 16,   // freq = 262144
            2 => self.timer_counter = 64,   // freq = 65536
            3 => self.timer_counter = 256,  // freq = 16382
            _ => self.timer_counter = 1024, // default
        }
    }

    fn handle_banking(&mut self, addr: Word, value: Byte) {
        // Performs a ram bank change
        if addr < 0x2000 {
            if self.rom_banking_type != RomBankingType::None {
                self.enable_ram_banking(addr, value);
            }
        }
        // Performas a ROM bank change
        else if (0x2000..0x4000).contains(&addr) {
            if self.rom_banking_type != RomBankingType::None {
                self.change_low_rom_banking(value);
            }
        }
        // Performs a rom or ram bank change
        else if (0x4000..0x6000).contains(&addr) {
            if self.rom_banking_type == RomBankingType::MBC1 {
                // if in rom banking mode, set the rom bank value
                // otherwise modify ram banking
                if self.rom_bank_enable {
                    self.change_high_rom_banking(value);
                } else {
                    self.change_ram_banking(value);
                }
            }
        }
        // Now handle whether we are rom banking or ram banking
        else if (0x6000..0x8000).contains(&addr) && self.rom_banking_type == RomBankingType::MBC1
        {
            self.change_banking_mode(value);
        }
    }

    fn enable_ram_banking(&mut self, addr: Word, value: Byte) {
        // If the mdoe is MBC2 we don't need to change anything
        if self.rom_banking_type == RomBankingType::MBC2 && addr.bitand(0x10) == 0x10 {
            return;
        }

        // Checks and sets ram enablement
        if value.bitand(0xF) == 0xA {
            self.ram_write_enable = true;
        } else if value.bitand(0xF) == 0x0 {
            self.ram_write_enable = false;
        }
    }

    fn change_ram_banking(&mut self, value: Byte) {
        self.ram_banks = match value {
            0 => CurrentRamBank::Bank0,
            1 => CurrentRamBank::Bank1,
            2 => CurrentRamBank::Bank2,
            3 => CurrentRamBank::Bank3,
            _ => CurrentRamBank::Bank0,
        }
    }

    fn change_low_rom_banking(&mut self, value: Byte) {
        if self.rom_banking_type == RomBankingType::MBC2 {
            self.rom_banks = CurrentRomBank::from(value & 0xF);
            return;
        }

        //turns off the lower 5 bits of the banking mode
        let lower5: Byte = value & 31;
        let current = self.rom_banks.value();
        // debug_println!("Current Banking Type: {:#?}", current);
        // debug_println!("Maked Lower 5: {:#?}", lower5);
        let masked = (current & 224) | lower5;
        // debug_println!("Post Banking Type: {:#?}", masked);
        self.rom_banks = CurrentRomBank::from(masked);
        if self.rom_banks == CurrentRomBank::Bank(0) {
            self.rom_banks = CurrentRomBank::Bank(1);
        }
    }
    fn change_high_rom_banking(&mut self, value: Byte) {
        //turns off the upper 3 bits of the banks and the lower 5 of the data
        let current = self.rom_banks.value();
        let masked = (current & 31) | (value & 224);
        self.rom_banks = CurrentRomBank::from(masked);
        if self.rom_banks == CurrentRomBank::Bank(0) {
            self.rom_banks = CurrentRomBank::Bank(1);
        }
    }

    fn change_banking_mode(&mut self, value: Byte) {
        self.rom_bank_enable = match value & 0x1 {
            0 => true,
            1 => false,
            _ => true,
        };

        if self.rom_bank_enable {
            self.ram_banks = CurrentRamBank::Bank0;
        }
    }

    /// Function for setting ram to requred startup values
    ///
    /// Its pretty messy
    pub fn ram_startup(&mut self) {
        self.mem[0xFF05] = 0x00;
        self.mem[0xFF06] = 0x00;
        self.mem[0xFF07] = 0x00;
        self.mem[0xFF10] = 0x80;
        self.mem[0xFF11] = 0xBF;
        self.mem[0xFF12] = 0xF3;
        self.mem[0xFF14] = 0xBF;
        self.mem[0xFF16] = 0x3F;
        self.mem[0xFF17] = 0x00;
        self.mem[0xFF19] = 0xBF;
        self.mem[0xFF1A] = 0x7F;
        self.mem[0xFF1B] = 0xFF;
        self.mem[0xFF1C] = 0x9F;
        self.mem[0xFF1E] = 0xBF;
        self.mem[0xFF20] = 0xFF;
        self.mem[0xFF21] = 0x00;
        self.mem[0xFF22] = 0x00;
        self.mem[0xFF23] = 0xBF;
        self.mem[0xFF24] = 0x77;
        self.mem[0xFF25] = 0xF3;
        self.mem[0xFF26] = 0xF1;
        self.mem[0xFF40] = 0x91;
        self.mem[0xFF42] = 0x00;
        self.mem[0xFF43] = 0x00;
        self.mem[0xFF45] = 0x00;
        self.mem[0xFF47] = 0xFC;
        self.mem[0xFF48] = 0xFF;
        self.mem[0xFF49] = 0xFF;
        self.mem[0xFF4A] = 0x00;
        self.mem[0xFF4B] = 0x00;
        self.mem[0xFFFF] = 0x00;
        self.refresh_rom_banking_type();
    }

    /// Checks to get the current rom banking type
    pub fn refresh_rom_banking_type(&mut self) {
        match self.read_byte_forced(0x147) {
            1..=3 => self.rom_banking_type = RomBankingType::MBC1,
            5..=6 => self.rom_banking_type = RomBankingType::MBC2,
            _ => self.rom_banking_type = RomBankingType::None,
        }
    }

    /// Requests an interrupt for the CPU to handle
    pub fn request_interrupt(&mut self, interrupt: Byte) {
        let mut request = self.read_byte(IF);
        request |= 1 << interrupt; // Sets the bit of the request
        // debug_println!("Writing Interrupt {}", request);
        self.write_byte(IF, request);
    }

    pub fn update_joypad_state(&mut self, buttons: Byte, directions: Byte) {
        self.joypad_buttons = buttons & 0x0F;
        self.joypad_directions = directions & 0x0F;
        self.recompute_joypad();
    }

    /// Loads the given ROM bytes into memory
    pub fn load_rom_data(&mut self, data: &[u8]) {
        self.mem.fill(0); // clear VRAM, WRAM, OAM, I/O mirrors
        self.rom.clear();
        self.rom.extend_from_slice(data);
        self.external_ram = [[0; 0x2000]; 4];

        self.rom_banks = CurrentRomBank::Bank(1);
        self.ram_banks = CurrentRamBank::Bank0;
        self.rom_bank_enable = true;
        self.ram_write_enable = false;
        self.refresh_rom_banking_type();
    }

    fn read_byte_internal(&self, addr: Word) -> Byte {
        // read from the always consistant rom bank
        if addr < 0x4000 {
            return self.read_rom_byte(addr as usize);
        }

        // map to rom banking
        if (0x4000..=0x7FFF).contains(&addr) {
            let relative = (addr - 0x4000) as usize;
            let offset = (self.rom_banks.value() as usize) * 0x4000;
            return self.read_rom_byte(offset + relative);
        }

        // map to ram banking
        if (0xA000..=0xBFFF).contains(&addr) {
            let offset = (addr - 0xA000) as usize;
            let bank = self.ram_banks as usize;
            return self.external_ram[bank][offset];
        }

        self.mem[addr as usize]
    }

    fn recompute_joypad(&mut self) {
        let prev = self.mem[INPUT_REGISTER as usize];

        let select_buttons = prev & 0x20 == 0;
        let select_directions = prev & 0x10 == 0;

        let mut lower = 0x0F;
        if select_buttons {
            lower &= self.joypad_buttons;
        }
        if select_directions {
            lower &= self.joypad_directions;
        }

        let next = (prev & 0x30) | 0xC0 | (lower & 0x0F);
        self.mem[INPUT_REGISTER as usize] = next;

        let prev_low = prev & 0x0F;
        let next_low = next & 0x0F;
        if (prev_low & !next_low) != 0 {
            self.request_interrupt(4);
        }
    }

    fn read_rom_byte(&self, index: usize) -> Byte {
        if self.rom.is_empty() {
            if index < self.mem.len() {
                self.mem[index]
            } else {
                0
            }
        } else {
            let len = self.rom.len();
            let masked = index % len;
            self.rom[masked]
        }
    }

    #[allow(dead_code)]
    /// Enables an interrupt for the CPU to handle
    pub fn enable_interrupt(&mut self, interrupt: Byte) {
        // Use the current value of the IE register to preserve any already
        // enabled interrupts. Reading from IF here would incorrectly overwrite
        // the existing enables.
        let mut enabled = self.read_byte(IE);
        enabled |= 1 << interrupt; // Sets the bit of the request
        // debug_println!("Enabling Interrupt {}", enabled);
        self.write_byte(IE, enabled);
    }

    /// Sets sprite into the sprite ram
    fn dma_transfer(&mut self, value: Byte) {
        let address: Word = (value as Word) << 8; // source address is data * 100
        for i in 0..0xA0 {
            let memory = self.read_byte(address + i);
            self.write_byte(SPRITE_RAM + i, memory);
        }
    }

    pub fn get_color(&self, color_num: Byte, addr: Word) -> Color {
        let palette = self.read_byte(addr);

        let hi;
        let lo;

        match color_num {
            0 => {
                hi = 1;
                lo = 0
            }
            1 => {
                hi = 3;
                lo = 2
            }
            2 => {
                hi = 5;
                lo = 4
            }
            3 => {
                hi = 7;
                lo = 6
            }
            _ => {
                panic!("Color number invalid")
            } // this should not be possible
        }

        let color = (((palette & (1 << hi)) >> hi) << 1) | ((palette & (1 << lo)) >> lo);

        match color {
            0 => Color::White,
            1 => Color::LightGrey,
            2 => Color::DarkGrey,
            3 => Color::Black,
            _ => panic!("Invalid color found"), // this should not be possible
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ntest::timeout;

    #[test]
    #[timeout(10)]
    fn test_mem_startup() {
        let mut mem: Memory = Memory::new();

        mem.ram_startup();

        // Selects a couple of memory addresses to check
        assert_eq!(mem.read_byte(0xFF11), 0xBF);
        assert_eq!(mem.read_byte(0xFF19), 0xBF);
        assert_eq!(mem.read_byte(0xFF24), 0x77);
    }

    #[test]
    #[timeout(10)]
    fn test_read_write_ram() {
        let mut mem: Memory = Memory::new();

        //Writing to valid memory space
        mem.write_byte(0xD000, 0x9);
        assert_eq!(0x9, mem.read_byte(0xD000));

        //Writing to valid memory space
        mem.write_byte(0xD010, 0x9);
        assert_eq!(0x9, mem.read_byte(0xD010));

        //Writing to echo memory space
        mem.write_byte(0xE000, 0x9);
        assert_eq!(0x9, mem.read_byte(0xE000));
        assert_eq!(0x9, mem.read_byte(0xE000 - 0x2000));
    }

    #[test]
    #[timeout(10)]
    fn test_read_word_wraps_at_end() {
        let mut mem: Memory = Memory::new();
        mem.write_byte_forced(0xFFFF, 0xAA);
        mem.write_byte_forced(0x0000, 0xBB);
        assert_eq!(mem.read_word(0xFFFF), 0xBBAA);
    }

    #[test]
    #[timeout(10)]
    fn test_write_word_high_region() {
        let mut mem: Memory = Memory::new();
        mem.write_word(0xFFFE, 0xBEEF);
        assert_eq!(mem.read_byte(0xFFFE), 0xEF);
        assert_eq!(mem.read_byte(0xFFFF), 0xBE);
    }

    #[test]
    #[timeout(10)]
    fn test_invalid_write() {
        let mut mem: Memory = Memory::new();

        //Writing to invalid memory space
        mem.write_byte(0x0, 0x9);
        assert_ne!(0x9, mem.read_byte(0x0));

        //Writing to invalid memory space
        mem.write_byte(0x10, 0x9);
        assert_ne!(0x9, mem.read_byte(0x10));

        //Writing to invalid memory space
        mem.write_byte(0xFEA0, 0x9);
        assert_ne!(0x9, mem.read_byte(0xFEA0));
    }

    #[test]
    #[timeout(10)]
    fn test_echo_mem() {
        let mut mem: Memory = Memory::new();

        //Writing to echo mem space
        mem.write_byte(0xE000, 0x9);
        assert_eq!(0x9, mem.read_byte(0xE000));
        assert_eq!(0x9, mem.read_byte(0xE000 - 0x2000));

        mem.write_byte(0xF100, 0x8);
        assert_eq!(0x8, mem.read_byte(0xF100));
        assert_eq!(0x8, mem.read_byte(0xF100 - 0x2000));
    }

    #[test]
    #[timeout(10)]
    fn test_enabling_ram() {
        let mut mem: Memory = Memory::new();
        mem.ram_startup();

        //Sets MBC1
        mem.write_byte_forced(0x147, 1);
        mem.refresh_rom_banking_type();
        assert_eq!(mem.rom_banking_type, RomBankingType::MBC1);

        mem.write_byte(0x1, 0xA);
        println!("{}", mem.ram_write_enable);
        assert!(mem.ram_write_enable);

        mem.write_byte(0x1, 0x0);
        println!("{}", mem.ram_write_enable);
        assert!(!mem.ram_write_enable);
    }

    #[test]
    #[timeout(10)]
    fn test_mbc1() {
        let mut mem: Memory = Memory::new();
        mem.ram_startup();

        //Sets MBC1
        mem.write_byte_forced(0x147, 1);
        mem.refresh_rom_banking_type();
        assert_eq!(mem.rom_banking_type, RomBankingType::MBC1);

        //Turn ram banks on
        mem.write_byte(0x1, 0xA);
        println!("{}", mem.ram_write_enable);
        assert!(mem.ram_write_enable);

        //Turn ram banks off
        mem.write_byte(0x1, 0x0);
        println!("{}", mem.ram_write_enable);
        assert!(!mem.ram_write_enable);

        //Change rom bank
        // debug_println!("\nCORRECTLY SET BANKS");
        mem.write_byte(0x2001, 0x0);
        assert_eq!(mem.rom_banks, CurrentRomBank::Bank(1));
        mem.write_byte(0x2001, 0x1);
        assert_eq!(mem.rom_banks, CurrentRomBank::Bank(1));
        mem.write_byte(0x2001, 0x2);
        assert_eq!(mem.rom_banks, CurrentRomBank::Bank(2));
        mem.write_byte(0x2001, 0x3);
        assert_eq!(mem.rom_banks, CurrentRomBank::Bank(3));

        //Turn on ROM banking
        mem.write_byte(0x6000, 0);
        assert!(mem.rom_bank_enable);
        assert_eq!(mem.ram_banks, CurrentRamBank::Bank0);
        mem.write_byte(0x4001, 0x20);
        assert_eq!(mem.rom_banks, CurrentRomBank::Bank(35));

        //Test banking set failure
        // debug_println!("\nINCORRECTLY SET BANKS");
        mem.write_byte(0x2001, 0x40);
        assert_eq!(mem.rom_banks, CurrentRomBank::Bank(32));

        //Turn on RAM Banking
        mem.write_byte(0x6000, 1);
        assert!(!mem.rom_bank_enable);
        assert_eq!(mem.ram_banks, CurrentRamBank::Bank0);
        mem.write_byte(0x4000, 0x2);
        assert_eq!(mem.ram_banks, CurrentRamBank::Bank2);
    }

    #[test]
    #[timeout(10)]
    fn test_mbc2() {
        let mut mem: Memory = Memory::new();
        mem.ram_startup();

        //Sets MBC1
        mem.write_byte_forced(0x147, 5);
        mem.refresh_rom_banking_type();
        assert_eq!(mem.rom_banking_type, RomBankingType::MBC2);

        mem.write_byte(0x1, 0xA);
        println!("{}", mem.ram_write_enable);
        assert!(mem.ram_write_enable);

        mem.write_byte(0x1, 0x0);
        println!("{}", mem.ram_write_enable);
        assert!(!mem.ram_write_enable);

        mem.write_byte(0x11, 0xA);
        println!("{}", mem.ram_write_enable);
        assert!(!mem.ram_write_enable);
    }

    #[test]
    #[timeout(10)]
    fn test_get_color() {
        let mut mem = Memory::new();
        mem.write_byte_forced(0xFF47, 0xE4); // 1110_0100

        assert_eq!(mem.get_color(0, 0xFF47), Color::White);
        assert_eq!(mem.get_color(1, 0xFF47), Color::LightGrey);
        assert_eq!(mem.get_color(2, 0xFF47), Color::DarkGrey);
        assert_eq!(mem.get_color(3, 0xFF47), Color::Black);
    }

    #[test]
    #[timeout(100)]
    fn test_clock_frequency_values() {
        let mut mem = Memory::new();

        mem.write_byte_forced(TMC, 0);
        mem.set_clock_frequency();
        assert_eq!(mem.timer_counter, 1024);

        mem.write_byte_forced(TMC, 1);
        mem.set_clock_frequency();
        assert_eq!(mem.timer_counter, 16);

        mem.write_byte_forced(TMC, 2);
        mem.set_clock_frequency();
        assert_eq!(mem.timer_counter, 64);

        mem.write_byte_forced(TMC, 3);
        mem.set_clock_frequency();
        assert_eq!(mem.timer_counter, 256);
    }

    #[test]
    #[timeout(100)]
    fn test_dma_transfer() {
        let mut mem = Memory::new();

        // Fill source memory region with known values
        for i in 0..0xA0 {
            mem.write_byte_forced(0xC000 + i, i as Byte);
        }

        // Trigger DMA transfer from 0xC000 to sprite RAM
        mem.write_byte(DMA_REG, 0xC0);

        for i in 0..0xA0 {
            assert_eq!(mem.read_byte(SPRITE_RAM + i), i as Byte);
        }
    }

    #[test]
    #[timeout(100)]
    fn test_set_clock_frequency() {
        let mut mem = Memory::new();

        let tests = [(0x0, 1024), (0x1, 16), (0x2, 64), (0x3, 256)];

        for (val, expected) in tests {
            mem.write_byte_forced(TMC, val);
            mem.set_clock_frequency();
            assert_eq!(mem.timer_counter, expected);
        }
    }

    #[test]
    #[timeout(100)]
    fn test_request_enable_interrupt() {
        let mut mem = Memory::new();
        mem.request_interrupt(1);
        assert_eq!(mem.read_byte(IF), 0x2);
        mem.request_interrupt(2);
        assert_eq!(mem.read_byte(IF), 0x6);

        let mut mem2 = Memory::new();
        mem2.enable_interrupt(1);
        assert_eq!(mem2.read_byte(IE), 0x2);
        mem2.request_interrupt(1); // preserve previous bit via IF register
        mem2.enable_interrupt(2);
        assert_eq!(mem2.read_byte(IE), 0x6);
    }

    #[test]
    #[timeout(100)]
    fn test_interrupt_bit_ops() {
        let mut mem = Memory::new();

        mem.request_interrupt(4);
        assert_eq!(mem.read_byte(IF), 1 << 4);

        mem.enable_interrupt(4);
        assert_eq!(mem.read_byte(IE), 1 << 4);
    }

    #[test]
    #[timeout(10)]
    fn test_joypad_write_preserves_select_and_recomputes() {
        let mut mem = Memory::new();
        mem.write_byte_forced(INPUT_REGISTER, 0xFF);
        mem.update_joypad_state(0x0E, 0x0F); // A pressed

        mem.write_byte(INPUT_REGISTER, 0x10); // select buttons
        assert_eq!(mem.read_byte(INPUT_REGISTER), 0xDE);
        assert_eq!(mem.read_byte(IF) & (1 << 4), 1 << 4);
    }

    #[test]
    #[timeout(10)]
    fn test_load_rom_data_small() {
        let mut mem = Memory::new();
        let mut data = vec![0u8; 0x200];
        for (i, b) in data.iter_mut().enumerate() {
            *b = i as u8;
        }
        data[0x147] = 2; // MBC1

        mem.load_rom_data(&data);

        for (i, b) in data.iter().enumerate() {
            assert_eq!(mem.read_byte(i as Word), *b);
        }
        assert_eq!(mem.rom_banking_type, RomBankingType::MBC1);
        assert_eq!(mem.read_byte(data.len() as Word), data[0]);
    }

    #[test]
    #[timeout(10)]
    fn test_load_rom_data_truncate() {
        let mut mem = Memory::new();
        let mut data = vec![0u8; 0x9000];
        for (i, b) in data.iter_mut().enumerate() {
            *b = (i & 0xFF) as u8;
        }
        data[0x147] = 1; // MBC1

        mem.load_rom_data(&data);

        for i in 0..0x8000 {
            assert_eq!(mem.read_byte(i as Word), data[i]);
        }
        assert_eq!(mem.read_byte(0x8000), 0);
    }
}
