#![allow(dead_code)]

use crate::mem::*;
use crate::types::*;


/// Basic implementation and methods for the LCD Screen
pub struct Screen {
    scanline_counter: i32,
    device_memory: SharedMemory, // 64KB address space

    //buffer is of size (h * w * 3)
    //buffer can be indexed as (h + (w*3))
    pub buffer: LCD, // Each pixel is a byte (0-3 for Game Boy palettes)
}

impl Screen {
    pub fn new() -> Self {
        // TODO: Initialize the screen buffer
        unimplemented!()
    }

    pub fn clear(&mut self, _color: u8) {
        // TODO: Clear the screen buffer with the given color
        unimplemented!()
    }

    pub fn set_pixel(&mut self, _x: usize, _y: usize, _color: u8) {
        // TODO: Set a pixel in the buffer
        unimplemented!()
    }

    pub fn get_pixel(&self, _x: usize, _y: usize) -> u8 {
        // TODO: Get a pixel from the buffer
        unimplemented!()
    }

    pub fn update_screen(&mut self, cycles: i32) {
        //TODO: Create function for updating screen at 60Hz

        self.set_lcd_status();

        if self.is_lcd_enabled() {
            self.scanline_counter -= cycles;
        } else {
            // LCD is not enabled so do nothing
            return;
        }

        let mut mem = self.device_memory.lock().unwrap();

        if self.scanline_counter <= 0 {
            // Time to move onto the next scanline
            let scanline = mem.read_byte(CURRENT_SCANLINE) + 1;
            mem.write_byte_forced(CURRENT_SCANLINE, scanline);

            self.scanline_counter = 456;

            // we are now in the vertical blank period
            if scanline == 144 {
                mem.request_interrupt(0);
            } 
            // If we are past 153, we reset to 0
            else if scanline > 153 {
                mem.write_byte_forced(CURRENT_SCANLINE, 0);
            }

            // Otherwise we draw the current line
            else if scanline < 144 {

                // We are done with the lock
                drop(mem);
                self.draw_scanline();
            } 
        }

        
    }

    fn draw_scanline(&mut self) {

    }

    fn set_lcd_status(&self) {
        
    }

    fn is_lcd_enabled(&self) -> bool {
        todo!();
    }

    // TODO: Add more methods for drawing, sprites, etc.
}
