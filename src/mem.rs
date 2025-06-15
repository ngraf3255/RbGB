use std::ops::BitAnd;

/// Functions and storage for operating on device memory
use crate::types::*;

pub struct Memory {
    pub mem: Ram,
    rom_banking_type: RomBankingType,
    rom_banks: CurrentRomBank,
    ram_banks: CurrentRamBank,
    ram_write_enable: bool,
    rom_bank_enable: bool,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            mem: [0; MEM_SIZE],
            rom_banking_type: RomBankingType::None,
            rom_banks: CurrentRomBank::Bank1,
            ram_banks: CurrentRamBank::Bank0,
            ram_write_enable: false,
            rom_bank_enable: false,
        }
    }

    // Wrapper for memory read functionality
    pub fn read_byte(&self, addr: Word) -> Byte {
        // are we reading from the rom memory bank?
        if (0x4000..0x7FFF).contains(&addr) {
            let addr = addr as usize;
            return self.mem[addr + ((self.rom_banks as usize) * 0x4000)];
        } else if (0xA000..=0xBFFF).contains(&addr) {
            // RAM bank storage and indexing
            let addr = addr as usize;
            return self.mem[addr + (self.ram_banks as usize) * 0x2000];
        }
        // Else return memory
        self.mem[addr as usize]
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
                let addr = addr as usize;
                self.mem[addr + (self.ram_banks as usize * 0x2000)] = value;
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
        } else {
            self.mem[addr as usize] = value;
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
        else if (0x6000..0x8000).contains(&addr) {
            if self.rom_banking_type == RomBankingType::MBC1 {
                self.change_banking_mode(value);
            }
        }
    }

    fn enable_ram_banking(&mut self, addr: Word, value: Byte) {
        // If the mdoe is MBC2 we don't need to change anything
        if self.rom_banking_type == RomBankingType::MBC2
            && self.read_byte(addr).bitand(0x10) == 0x10
        {
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
        let current = self.rom_banks as u8;
        let masked = (current & 224) | lower5;
        self.rom_banks = CurrentRomBank::from(masked);
        if self.rom_banks == CurrentRomBank::Bank0 {
            self.rom_banks = CurrentRomBank::Bank1;
        }
    }
    fn change_high_rom_banking(&mut self, value: Byte) {
        //turns off the upper 3 bits of the banks and the lower 5 of the data
        let current = self.rom_banks as u8;
        let masked = (current & 31) | (value & 224);
        self.rom_banks = CurrentRomBank::from(masked);
        if self.rom_banks == CurrentRomBank::Bank0 {
            self.rom_banks = CurrentRomBank::Bank1;
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

    /// Returns the rom banking type of the current game
    pub fn identify_banking_type(self) -> RomBankingType {
        match self.read_byte(0x147) {
            1..3 => RomBankingType::MBC1,
            5..6 => RomBankingType::MBC2,
            _ => RomBankingType::None,
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
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mem_startup() {
        let mut mem: Memory = Memory::new();

        mem.ram_startup();

        // Selects a couple of memory addresses to check
        assert_eq!(mem.read_byte(0xFF11), 0xBF);
        assert_eq!(mem.read_byte(0xFF19), 0xBF);
        assert_eq!(mem.read_byte(0xFF24), 0x77);
    }

    #[test]
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
}
