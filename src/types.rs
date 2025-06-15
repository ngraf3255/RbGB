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
