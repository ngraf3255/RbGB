#![allow(dead_code)]

extern crate sdl2;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cpu::CPU;
use debug_print::debug_println;
use mem::{Memory, SharedMemory};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, pixels::PixelFormatEnum, rect::Rect};
use types::{SCREEN_HEIGHT, SCREEN_WIDTH};

mod bus;
mod cpu;
mod ctc;
mod graphics;
mod mem;
mod registers;
mod sound;
mod types;

///Main entry point to gameboy simulation
fn main() -> Result<(), String> {
    println!("Hello, world!");

    let mut emulator = Emulator::new();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut event_pump = sdl_context.event_pump()?;

    let window = video_subsystem
        .window("Gameboy Emulator", SCREEN_WIDTH * 5, SCREEN_HEIGHT * 5)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
        .map_err(|e| e.to_string())?;

    'running: loop {
        let frame_start = Instant::now();

        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    emulator.toggle_pause();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::L),
                    ..
                } => {
                    println!("Enter path to ROM:");
                    let path = "roms/kirby.gb";

                    if let Err(e) = emulator.load_rom(path.trim()) {
                        println!("Failed to load ROM: {e}");
                    } else {
                        println!("ROM loaded");
                    }
                }
                _ => {}
            }
        }

        // Emulator main loop
        emulator.update();

        // Update the texture with new pixel data
        emulator.blit_rgb_bytes_to_texture(&mut texture)?;

        // Draw
        canvas.clear();
        canvas.copy(
            &texture,
            None,
            Some(Rect::new(0, 0, SCREEN_WIDTH * 5, SCREEN_HEIGHT * 5)),
        )?;

        if emulator.paused {
            canvas.set_draw_color(Color::RGB(50, 50, 50));
            canvas.fill_rect(Rect::new(0, 0, SCREEN_WIDTH * 5, SCREEN_HEIGHT * 5))?;
        }

        canvas.present();

        // Frame limiting to 60 FPS
        let frame_duration = frame_start.elapsed();
        if frame_duration < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - frame_duration);
        }
    }

    Ok(())
}

struct Emulator {
    screen: graphics::Screen,
    cpu: CPU,
    memory: SharedMemory,
    paused: bool,
}

impl Emulator {
    /// Calls the needed functions once a frame
    ///
    const MAXCYCLES: u32 = 69905;

    pub fn new() -> Self {
        let mem = Arc::new(Mutex::new(Memory::new()));
        mem.lock().unwrap().ram_startup();
        Emulator {
            screen: graphics::Screen::new(Arc::clone(&mem)),
            cpu: CPU::new(Arc::clone(&mem)),
            memory: mem,
            paused: true,
        }
    }

    pub fn update(&mut self) {
        if self.paused {
            return;
        }
        // debug_println!("Main Loop");
        let mut num_cycles: u32 = 0;
        while num_cycles < Self::MAXCYCLES {
            debug_println!("Program Counter: 0x{:X}", self.cpu.registers.val_pc());
            let cycles = self.cpu.execute_next_opcode(false);
            num_cycles += cycles as u32;
            self.cpu.timers.update_timers(cycles as i32);
            self.screen.update_screen(cycles as i32);
            self.cpu.handle_interrupts();
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let mut mem = self.memory.lock().unwrap();
        mem.load_rom_data(&data);
        mem.ram_startup();
        drop(mem);
        self.cpu.reset();
        self.paused = false;
        Ok(())
    }

    pub fn blit_rgb_bytes_to_texture(
        &self,
        texture: &mut sdl2::render::Texture,
    ) -> Result<(), String> {
        let data = &self.screen.buffer;
        let pitch = SCREEN_WIDTH * 3; // 3 bytes per pixel

        if data.len() != (pitch * SCREEN_HEIGHT) as usize {
            return Err(format!(
                "Expected {} bytes, but got {}",
                pitch * SCREEN_HEIGHT,
                data.len()
            ));
        }

        // // debug_println!("Starting blit... ");
        texture
            .update(None, &data[..], pitch as usize)
            .map_err(|e| e.to_string())?; // possibly replace with ?
        // // debug_println!("Blit successful. ");
        Ok(())
    }
}

#[cfg(test)]
mod test {}
