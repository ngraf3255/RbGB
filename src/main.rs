#[cfg(feature = "gui")]
extern crate sdl2;

mod emulator;
mod sdl;
mod types;

use emulator::Emulator;
use sdl::SdlApp;

///Main entry point to gameboy simulation
fn main() -> Result<(), String> {
    println!("Starting emulator");

    let mut emulator = Emulator::new();
    let mut sdl_app = SdlApp::new()?;
    sdl_app.run(&mut emulator)
}

#[cfg(test)]
mod test {}
