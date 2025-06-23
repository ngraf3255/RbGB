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
pub type LCD = [Byte; (SCREEN_HEIGHT * SCREEN_WIDTH * 3) as usize];

// Timer and CPU constants
pub const TIMA: Word = 0xFF05;
pub const TMA: Word = 0xFF06;
pub const TMC: Word = 0xFF07;
pub const DIVIDER_REGISTER: Word = 0xFF04;
pub const CLOCKSPEED: u32 = 4194304;

// Interrupt Constants
pub const IE: Word = 0xFFFF; // Interrupt enabled register
pub const IF: Word = 0xFF0F; // Interrupt request register

// Screen Constants
pub const SCREEN_HEIGHT: u32 = 144;
pub const SCREEN_WIDTH: u32 = 160;
pub const CURRENT_SCANLINE: Word = 0xFF44;
pub const LCD_STATUS: Word = 0xFF41;

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
