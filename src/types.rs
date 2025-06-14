#![allow(dead_code)]

use lazy_static::lazy_static;

pub type Byte = u8;
pub type SignedByte = i8;
pub type Word = u16;
pub type SignedWord = i16;

lazy_static! {
    pub static ref cartridge_memory: [Byte; 0x20000] = [0; 0x20000];
}
