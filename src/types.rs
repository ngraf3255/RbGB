#![allow(dead_code)]

use lazy_static::lazy_static;
use std::fs::File;
use std::io;
use std::io::Read;
use std::sync::Mutex;

pub type Byte = u8;
pub type SignedByte = i8;
pub type Word = u16;
pub type SignedWord = i16;
pub type Cartridge = [Byte; FILESIZE];
pub type Ram = [Byte; MEM_SIZE];
#[allow(clippy::upper_case_acronyms)]
pub type LCD = [Byte; (SCREEN_HEIGHT * SCREEN_WIDTH * 3) as usize];

// Timer and CPU constants
pub const TIMA: Word = 0xFF05;
pub const TMA: Word = 0xFF06;
pub const TMC: Word = 0xFF07;
pub const DIVIDER_REGISTER: Word = 0xFF04;
pub const CLOCKSPEED: u32 = 4194304;
/// CPU carry flag
pub const CF: Byte = 1 << 0;
/// CPU add/subtract flag
pub const NF: Byte = 1 << 1;
/// CPU overflow flag (same as parity)
pub const VF: Byte = 1 << 2;
/// CPU parity flag (same as overflow)
pub const PF: Byte = 1 << 2;
/// CPU undocumented 'X' flag
pub const XF: Byte = 1 << 3;
/// CPU half carry flag
pub const HF: Byte = 1 << 4;
/// CPU undocumented 'Y' flag
pub const YF: Byte = 1 << 5;
/// CPU zero flag
pub const ZF: Byte = 1 << 6;
/// CPU sign flag
pub const SF: Byte = 1 << 7;

pub const BC: Byte = 0;
pub const DE: Byte = 2;
pub const HL: Byte = 4;
pub const AF: Byte = 6;
pub const IX: Byte = 8;
pub const IY: Byte = 10;
pub const SP: Byte = 12;
pub const WZ: Byte = 14;
pub const BC_: Byte = 16;
pub const DE_: Byte = 18;
pub const HL_: Byte = 20;
pub const AF_: Byte = 22;
pub const WZ_: Byte = 24;

pub const SP_TABLE: [Byte; 4] = [BC, DE, HL, SP]; //TODO: Change to localize as global so compiler doesn't inline the table
pub const AF_TABLE: [Byte; 4] = [BC, DE, HL, AF];

// Interrupt Constants
pub const IE: Word = 0xFFFF; // Interrupt enabled register
pub const IF: Word = 0xFF0F; // Interrupt request register

// Screen Constants
pub const SCREEN_HEIGHT: u32 = 144;
pub const SCREEN_WIDTH: u32 = 160;
pub const CURRENT_SCANLINE: Word = 0xFF44;
pub const LCD_STATUS: Word = 0xFF41;
pub const LCD_CONTROL: Word = 0xFF40;
pub const COINCIDENCE_FLAG: Word = 0xFF45;
pub const DMA_REG: Word = 0xFF46;
pub const SPRITE_RAM: Word = 0xFE00; // from 0xFE00 to OxFE9F
pub const MODE_2_BOUNDS: i32 = 456 - 80;
pub const MODE_3_BOUNDS: i32 = MODE_2_BOUNDS - 172;
pub const MEMORY_REGION: Word = 0x8000; // Graphics memory location
pub const SIZE_OF_TILE_IN_MEMORY: i32 = 16;
pub const OFFSET: i32 = 128;

pub enum GameInput {
    Up,
    Left,
    Right,
    Down,
    Start,
    Select,
    A,
    B,
    Unknown,
}

pub enum KeyState {
    Down = 0,
    Up = 1,
}

impl Default for KeyState {
    fn default() -> Self {
        KeyState::Up
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    LightGrey,
    DarkGrey,
    Black,
}

/// RAM  Device Memory
pub const MEM_SIZE: usize = 0x10000;

//Likely at some point will switch the RAM and ROM to be part of the Emulator struct

/// ROM Device memory
const FILESIZE: usize = 0x20000;
lazy_static! {
    pub static ref CARTRIDGE_MEMORY: Mutex<Cartridge> = Mutex::new([0; FILESIZE]);
}

/// Loads the contents of the given file into the cartridge_memory array.
/// Returns the number of bytes read or an io::Error.
pub fn load_cartridge<P: AsRef<std::path::Path>>(path: P) -> io::Result<usize> {
    let mut file = File::open(path)?;
    let mut memory = CARTRIDGE_MEMORY.lock().unwrap();
    let bytes_read = file.read(&mut *memory)?;
    Ok(bytes_read)
}

#[derive(PartialEq, Debug)]

pub enum RomBankingType {
    MBC1,
    MBC2,
    None,
}

// I thought I was smart and would implement the rom banks as enums but little did I know...
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CurrentRomBank {
    Bank(u8),
}

impl From<u8> for CurrentRomBank {
    fn from(val: u8) -> Self {
        CurrentRomBank::Bank(val)
    }
}

impl CurrentRomBank {
    pub fn value(self) -> u8 {
        match self {
            CurrentRomBank::Bank(val) => val,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CurrentRamBank {
    Bank0,
    Bank1,
    Bank2,
    Bank3,
}

#[cfg(test)]
mod test {
    use super::*;
    use ntest::timeout;
    use std::fs::{File, remove_file};
    use std::io::Write;

    #[test]
    #[timeout(10)]
    fn test_load_cartridge_success() {
        let path = "test_cart.gb";
        {
            let mut f = File::create(path).unwrap();
            f.write_all(&[1u8, 2, 3, 4]).unwrap();
        }

        let bytes = load_cartridge(path).unwrap();
        assert_eq!(bytes, 4);
        let mem = CARTRIDGE_MEMORY.lock().unwrap();
        assert_eq!(&mem[..4], &[1, 2, 3, 4]);
        drop(mem);
        remove_file(path).unwrap();
    }

    #[test]
    #[timeout(10)]
    fn test_load_cartridge_missing() {
        let res = load_cartridge("nonexistent.gb");
        assert!(res.is_err());
    }

    #[test]
    fn test_current_rom_bank_conversion() {
        let bank: CurrentRomBank = 5u8.into();
        assert_eq!(bank.value(), 5);
    }
}
