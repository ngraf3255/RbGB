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

    fn draw_scanline(&mut self) {}

    fn set_lcd_status(&mut self) {
        // Gets lock on memory
        let mut mem = self.device_memory.lock().unwrap();

        let mut status = mem.read_byte(LCD_STATUS);

        // Behavior when lcd is disabled
        if !self.is_lcd_enabled() {
            // sets the mode to 1 when the lcd is disabled and resets the scanline
            self.scanline_counter = 456;
            mem.write_byte_forced(CURRENT_SCANLINE, 0); // resets scanline

            status &= 0xFC;
            status |= 0x1;
            mem.write_byte(LCD_STATUS, status);
            return;
        }

        let current_line = mem.read_byte(CURRENT_SCANLINE);
        let current_mode = status & 0x3;

        let mode;
        let mut require_interrupt = false;

        // in vblank so mode is set to 1
        if current_line >= 144 {
            mode = 1;
            status |= 0x1;
            status &= !0x2;
            require_interrupt = status & (1 << 4) != 0;
        }
        // mode 2
        else if self.scanline_counter >= MODE_2_BOUNDS {
            mode = 2;
            status |= 0x2;
            status &= !0x1;
            require_interrupt = status & (1 << 5) != 0;
        }
        // mode 3
        else if self.scanline_counter >= MODE_3_BOUNDS {
            mode = 3;
            status |= 0x3;
        }
        // mode 0
        else {
            mode = 0;
            status &= !0x3;
            require_interrupt = status & (1 << 3) != 0;
        }

        // we have no entered a new mode
        if require_interrupt && (mode != current_mode) {
            mem.request_interrupt(1);
        }

        // check the conicidence flag
        if current_line == mem.read_byte(COINCIDENCE_FLAG) {
            status |= 0x4;
            if status & (1 << 6) != 0 {
                mem.request_interrupt(1);
            }
        } else {
            status &= !0x4;
        }

        // Write the new status
        mem.write_byte(LCD_STATUS, status);
    }

    fn is_lcd_enabled(&self) -> bool {
        // Check bit 7 of LCD Control register (0xFF40)
        self.device_memory.lock().unwrap().read_byte(LCD_CONTROL) & (1 << 7) != 0
    }

    // TODO: Add more methods for drawing, sprites, etc.
}
