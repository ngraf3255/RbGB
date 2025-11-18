use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::types::{
    CURRENT_SCANLINE, GameInput, KeyState, LCD_CONTROL, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use debug_print::debug_println;

mod cpu;
mod graphics;
mod joypad;
mod mem;
mod registers;
mod sound;

pub struct Emulator {
    screen: graphics::Screen,
    cpu: cpu::CPU,
    joypad: joypad::Joypad,
    memory: mem::SharedMemory,
    paused: bool,
}

impl Emulator {
    /// Calls the needed functions once a frame
    const MAXCYCLES: u32 = 69905;
    const FRAME_DURATION: Duration = Duration::from_nanos(16_741_000);

    pub fn new() -> Self {
        let mem = Arc::new(Mutex::new(mem::Memory::new()));
        mem.lock().unwrap().ram_startup();
        Emulator {
            screen: graphics::Screen::new(Arc::clone(&mem)),
            cpu: cpu::CPU::new(Arc::clone(&mem)),
            joypad: joypad::Joypad::new(Arc::clone(&mem)),
            memory: mem,
            paused: true,
        }
    }

    pub fn update(&mut self) {
        if self.paused {
            return;
        }
        let frame_start = Instant::now();
        // debug_println!("Main Loop");
        let mut num_cycles: u32 = 0;
        while num_cycles < Self::MAXCYCLES {
            //debug_println!("Program Counter: 0x{:X}", self.cpu.registers.val_pc());
            let cycles = self.cpu.execute_next_opcode(false);
            num_cycles += cycles as u32;
            self.cpu.timers.update_timers(cycles as i32);
            self.screen.update_screen(cycles as i32);
            self.cpu.handle_interrupts();
        }
        if let Some(remaining) = Self::FRAME_DURATION.checked_sub(frame_start.elapsed()) {
            std::thread::sleep(remaining);
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let mut mem = self.memory.lock().unwrap();
        mem.load_rom_data(&data);
        mem.ram_startup();
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

    pub fn dump_lcd_mem(&self) {
        let mem = self.memory.lock().unwrap();

        debug_println!("IDK: {:X}", mem.read_byte_forced(0xFF26));
        debug_println!("LCD Control: {:X}", mem.read_byte_forced(LCD_CONTROL));
        debug_println!("Scroll Y: {:X}", mem.read_byte_forced(0xFF42));
        debug_println!("Scroll X: {:X}", mem.read_byte_forced(0xFF43));
        debug_println!("BG Palette: {:X}", mem.read_byte_forced(0xFF47));
        debug_println!("OBJ palette: {:X}", mem.read_byte_forced(0xFF48));
        debug_println!(
            "Current Scanline: {:X}",
            mem.read_byte_forced(CURRENT_SCANLINE)
        );
        debug_println!("LCD Control: {:X}", mem.read_byte_forced(LCD_CONTROL));
    }

    /// Handle input to the emulator
    pub fn game_input(&mut self, input: GameInput, val: KeyState) {
        self.joypad.log_input(input, val)
    }
}
