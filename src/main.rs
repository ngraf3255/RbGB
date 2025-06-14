#![allow(dead_code)]
use crate::types::*;

mod cpu;
mod graphics;
mod sound;
mod types;

///Main entry point to gameboy simulation
fn main() {
    println!("Hello, world!");
    println!("I am testing the workflow demo!!!")
}

struct Emulator {
    cartridge: cartridge_memory,
}

impl Emulator {
    /// Calls the needed functions once a frame
    fn update() {
        let _cycle: Byte = 0;
    }
}
