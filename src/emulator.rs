use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::types::{GameInput, KeyState};
use debug_print::debug_println;

mod cpu;
mod graphics;
mod joypad;
mod mem;
mod sound;

/// High-level Game Boy emulator coordinator.
///
/// Owns the CPU, memory, graphics, and input subsystems and drives the
/// per-frame execution loop. The emulator starts paused and must be unpaused
/// (or a ROM loaded) before it will execute instructions.
///
/// This type exposes the core API used by the frontend to load ROMs, advance
/// frames, provide input, and read the display buffer.
pub struct Emulator {
    screen: graphics::Screen,
    cpu: cpu::CPU,
    joypad: joypad::Joypad,
    memory: mem::SharedMemory,
    paused: bool,
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Emulator {
    /// Maximum CPU cycles executed per frame.
    ///
    /// This matches the approximate number of cycles a Game Boy runs in one
    /// video frame. The update loop runs until this budget is reached.
    const MAXCYCLES: u32 = 69905;
    /// Target duration for a single frame (approx. 59.7 Hz).
    ///
    /// The update loop sleeps to align with this duration after processing
    /// enough CPU cycles.
    const FRAME_DURATION: Duration = Duration::from_nanos(16_741_000);

    /// Create a new emulator instance with initialized subsystems.
    ///
    /// Memory is initialized to its startup state and the emulator begins
    /// paused until a ROM is loaded or `toggle_pause` is called.
    ///
    /// Returns a ready-to-use `Emulator` in the paused state.
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

    /// Execute one frame of emulation if not paused.
    ///
    /// Runs CPU instructions, updates timers and graphics, and handles
    /// interrupts until the frame's cycle budget is consumed. The function then
    /// sleeps to maintain the target frame rate.
    ///
    /// Returns `()` and has no effect if the emulator is paused.
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

    /// Toggle the paused state.
    ///
    /// When paused, `update` returns immediately without advancing emulation.
    ///
    /// Takes `&mut self` and returns `()`.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Check whether emulation is currently paused.
    ///
    /// Returns `true` if the emulator will skip work in `update`.
    ///
    /// Takes `&self` and returns a `bool` indicating pause state.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Load a ROM from disk and reset the CPU and memory state.
    ///
    /// The ROM contents are copied into memory, memory is reinitialized, and
    /// the CPU is reset. On success, the emulator is unpaused.
    ///
    /// Parameters:
    /// - `path`: filesystem path to the ROM file.
    ///
    /// Returns `Ok(())` on success or a string I/O error on failure.
    pub fn load_rom(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let mut mem = self.memory.lock().unwrap();
        mem.load_rom_data(&data);
        mem.ram_startup();
        self.cpu.reset();

        self.paused = false;
        Ok(())
    }

    /// Borrow the current display buffer for rendering.
    ///
    /// The buffer contains raw pixel data produced by the graphics subsystem.
    /// Its length and format are determined by `graphics::Screen`.
    ///
    /// Returns a borrowed `&[u8]` slice tied to the emulator's lifetime.
    pub fn get_display_buffer(&self) -> &[u8] {
        &self.screen.buffer
    }

    /// Dump key LCD and input registers for debugging.
    ///
    /// This is only compiled in debug builds and logs directly to stdout via
    /// `debug_println`. If not compiled for debug this function does nothing.
    ///
    /// Takes `&self` and returns `()`. All details are dumped to the console
    pub fn dump_lcd_mem(&self) {
        #[cfg(debug_assertions)]
        let mem = self.memory.lock().unwrap();

        debug_println!("IDK: {:X}", mem.read_byte_forced(0xFF26));
        debug_println!(
            "LCD Control: {:X}",
            mem.read_byte_forced(crate::types::LCD_CONTROL)
        );
        debug_println!("Scroll Y: {:X}", mem.read_byte_forced(0xFF42));
        debug_println!("Scroll X: {:X}", mem.read_byte_forced(0xFF43));
        debug_println!("BG Palette: {:X}", mem.read_byte_forced(0xFF47));
        debug_println!("OBJ palette: {:X}", mem.read_byte_forced(0xFF48));
        debug_println!(
            "Current Scanline: {:X}",
            mem.read_byte_forced(crate::types::CURRENT_SCANLINE)
        );
        debug_println!(
            "LCD Control: {:X}",
            mem.read_byte_forced(crate::types::LCD_CONTROL)
        );
        debug_println!(
            "Joystick Register: 0x{:X}",
            mem.read_byte_forced(crate::types::INPUT_REGISTER)
        );
    }

    /// Handle input for the emulator's joypad.
    ///
    /// Updates the joypad state and forwards the input to memory-mapped input
    /// registers.
    ///
    /// Parameters:
    /// - `input`: which Game Boy control was pressed or released.
    /// - `val`: the key state for that control.
    ///
    /// Returns `()`.
    pub fn game_input(&mut self, input: GameInput, val: KeyState) {
        self.joypad.log_input(input, val)
    }
}
