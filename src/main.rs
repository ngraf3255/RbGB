#![allow(dead_code)]

extern crate sdl2;

mod types;

use sdl::SdlApp;

mod emulator;

use crate::emulator::Emulator;

mod sdl;

///Main entry point to gameboy simulation
fn main() -> Result<(), String> {
    println!("Starting emulator");

    let mut emulator = Emulator::new();
    let mut sdl_app = SdlApp::new()?;
    sdl_app.run(&mut emulator)
}

#[cfg(test)]
mod test {}
