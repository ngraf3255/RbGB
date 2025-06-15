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
}

impl Emulator {
    /// Calls the needed functions once a frame
    /// 
    const MAXCYCLES: u32 = 69905;

    fn update() {
        let _cycle: Byte = 0;
    }
}


