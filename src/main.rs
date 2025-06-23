#![allow(dead_code)]

mod cpu;
mod graphics;
mod mem;
mod sound;
mod types;

///Main entry point to gameboy simulation
fn main() {
    println!("Hello, world!");
    println!("I am testing the workflow demo!!!")
}

struct Emulator {
    screen: graphics::Screen,
}

impl Emulator {
    /// Calls the needed functions once a frame
    ///
    const MAXCYCLES: u32 = 69905;

    pub fn new() -> Self {
        Emulator {
            screen: graphics::Screen::new(),
        }
    }

    pub fn update(&self) {
        let mut num_cycles: u32 = 0;
        while num_cycles < Self::MAXCYCLES {
            let cycles: u32 = self.execute_next_opcode();
            num_cycles += cycles;
            self.update_timers();
            self.update_graphics();
            self.handle_interrupts();
        }
        self.screen.update_screen(num_cycles);
    }

    fn execute_next_opcode(&self) -> u32 {
        //TODO: Build opcode execution code
        unimplemented!();
    }

    fn update_timers(&self) {
        //TODO: Creation functionality to update hardware timers
        unimplemented!();
    }

    fn update_graphics(&self) {
        //TODO: Link this to graphics library update_screen func
        unimplemented!();
    }

    fn handle_interrupts(&self) {
        //TODO: Handle all the interrupts
        unimplemented!();
    }
}

#[cfg(test)]
mod test {
    use crate::Emulator;

    #[test]
    #[should_panic]
    fn test_unimplemented_main_loop() {
        let emulator: Emulator = Emulator::new();
        emulator.update();
    }
}
