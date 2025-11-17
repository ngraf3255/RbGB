#![allow(dead_code)]
#[allow(unused_imports)]
use debug_print::debug_println;

use crate::emulator::mem::*;
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
    pub fn new(mem: SharedMemory) -> Self {
        // TODO: Initialize the screen buffer
        Screen {
            buffer: [0; (SCREEN_HEIGHT * SCREEN_WIDTH * 3) as usize],

            scanline_counter: 456,
            device_memory: mem,
        }
    }

    pub fn clear(&mut self, color: u8) {
        let (r, g, b) = Self::color_to_rgb(color);
        for chunk in self.buffer.chunks_mut(3) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        if x >= SCREEN_WIDTH as usize || y >= SCREEN_HEIGHT as usize {
            return;
        }
        let idx = (y * SCREEN_WIDTH as usize + x) * 3;
        let (r, g, b) = Self::color_to_rgb(color);
        self.buffer[idx] = r;
        self.buffer[idx + 1] = g;
        self.buffer[idx + 2] = b;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        if x >= SCREEN_WIDTH as usize || y >= SCREEN_HEIGHT as usize {
            return 0;
        }
        let idx = (y * SCREEN_WIDTH as usize + x) * 3;
        let r = self.buffer[idx];
        let g = self.buffer[idx + 1];
        let b = self.buffer[idx + 2];
        Self::rgb_to_color(r, g, b)
    }
    fn color_to_rgb(color: u8) -> (u8, u8, u8) {
        match color {
            0 => (255, 255, 255),
            1 => (0xCC, 0xCC, 0xCC),
            2 => (0x77, 0x77, 0x77),
            _ => (0, 0, 0),
        }
    }

    fn rgb_to_color(r: u8, g: u8, b: u8) -> u8 {
        match (r, g, b) {
            (255, 255, 255) => 0,
            (0xCC, 0xCC, 0xCC) => 1,
            (0x77, 0x77, 0x77) => 2,
            _ => 3,
        }
    }

    pub fn update_screen(&mut self, cycles: i32) {
        // debug_println!("Screen update!");

        self.set_lcd_status();

        if self.is_lcd_enabled() {
            self.scanline_counter -= cycles;
        } else {
            // LCD is not enabled so do nothing
            //debug_println!("LCD Disabled");
            return;
        }

        let mut mem = self.device_memory.lock().unwrap();

        // debug_println!("Check scanlines!");
        if self.scanline_counter <= 0 {
            // Time to move onto the next scanline
            let scanline = mem.read_byte(CURRENT_SCANLINE).wrapping_add(1);
            mem.write_byte_forced(CURRENT_SCANLINE, scanline);

            self.scanline_counter = 456;

            // we are now in the vertical blank period
            // debug_println!("Current Scanline is {scanline}");
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
        let mem = self.device_memory.lock().unwrap();
        let control = mem.read_byte(LCD_CONTROL);
        drop(mem);

        if control & (1 << 7) != 0 {
            if control & 0x1 != 0 {
                self.render_tiles(control);
            }

            if control & 0x2 != 0 {
                self.render_sprites(control);
            }
        }
    }

    fn render_tiles(&mut self, control: Byte) {
        debug_println!("Rendering tile, control: {:b}", control);
        let mem = self.device_memory.lock().unwrap();

        let mut unsigned = true;
        let tile_data = if control & (1 << 4) != 0 {
            0x8000
        } else {
            unsigned = false;
            0x8800
        };

        // where to draw the visual area and the window
        let scroll_y = mem.read_byte(0xFF42);
        let scroll_x = mem.read_byte(0xFF43);
        let window_y = mem.read_byte(0xFF4A);
        let window_x = mem.read_byte(0xFF4B).wrapping_sub(7);
        let current_line = mem.read_byte(CURRENT_SCANLINE);
        let using_window = (control & (1 << 5) != 0) && window_y <= current_line;

        // determine which tile map to use
        let background_memory: Word = if using_window {
            if control & (1 << 6) != 0 {
                0x9C00
            } else {
                0x9800
            }
        } else if control & (1 << 3) != 0 {
            0x9C00
        } else {
            0x9800
        };

        let y_pos = if using_window {
            current_line.wrapping_sub(window_y)
        } else {
            scroll_y.wrapping_add(current_line)
        };

        let tile_row: Word = ((y_pos / 8) as Word) * 32;

        for pixel in 0..SCREEN_WIDTH as Byte {
            let mut x_pos = pixel.wrapping_add(scroll_x);
            if using_window && pixel >= window_x {
                x_pos = pixel.wrapping_sub(window_x);
            }

            let tile_column = (x_pos / 8) as Word;
            let tile_num = mem.read_byte(background_memory + tile_row + tile_column);

            let mut tile_location: Word = tile_data;
            if unsigned {
                tile_location += tile_num as Word * 16;
            } else {
                let signed = tile_num as i8 as i16;
                tile_location += ((signed + 128) as Word) * 16;
            }

            let line = (y_pos % 8) * 2;
            let data1 = mem.read_byte(tile_location + line as Word);
            let data2 = mem.read_byte(tile_location + line as Word + 1);

            let color_bit = 7 - (x_pos % 8);
            let color_num = (((data2 >> color_bit) & 1) << 1) | ((data1 >> color_bit) & 1);

            let color: Color = mem.get_color(color_num, 0xFF47);
            let (red, green, blue) = match color {
                Color::White => (255, 255, 255),
                Color::LightGrey => (0xCC, 0xCC, 0xCC),
                Color::DarkGrey => (0x77, 0x77, 0x77),
                Color::Black => (0, 0, 0),
            };

            if current_line as usize >= SCREEN_HEIGHT as usize
                || pixel as usize >= SCREEN_WIDTH as usize
            {
                continue;
            }

            let idx = (current_line as usize * SCREEN_WIDTH as usize + pixel as usize) * 3;
            //debug_println!("Writing idx: {idx}");
            self.buffer[idx] = red;
            self.buffer[idx + 1] = green;
            self.buffer[idx + 2] = blue;
        }
    }

    fn render_sprites(&mut self, control: Byte) {
        let mem = self.device_memory.lock().unwrap();

        // test if display is enabled
        if control & 0x2 != 0 {
            let mut use8x16 = false;
            if control & 0x4 != 0 {
                use8x16 = true;
            }

            for sprite in 0..40 {
                let index = sprite * 4;
                let y_pos = mem.read_byte(0xFE00 + index) as i32 - 16;
                let x_pos = mem.read_byte(0xFE00 + index + 1) as i32 - 8;
                let tile_location = mem.read_byte(0xFE00 + index + 2);
                let attributes = mem.read_byte(0xFE00 + index + 3);

                let y_flip = attributes & (1 << 6) != 0;
                let x_flip = attributes & (1 << 5) != 0;

                let scanline = mem.read_byte(CURRENT_SCANLINE) as i32;
                let mut y_size = 8;

                if use8x16 {
                    y_size = 16;
                }

                if (scanline >= y_pos) && (scanline < (y_pos + y_size)) {
                    let mut line = scanline - y_pos;

                    if y_flip {
                        line -= y_size;
                        line *= -1;
                    }

                    line *= 2;
                    let data1 =
                        mem.read_byte((0x8000 + (tile_location as Word * 16)) + line as Word);
                    let data2 =
                        mem.read_byte((0x8000 + (tile_location as Word * 16)) + line as Word + 1);

                    for tile_pixel in (0..=7).rev() {
                        let mut color_bit = tile_pixel;

                        if x_flip {
                            color_bit -= 7;
                            color_bit *= -1;
                        }
                        let color_num = (((data2 & (1 << color_bit)) >> color_bit) << 1)
                            | ((data1 & (1 << color_bit)) >> color_bit);

                        let addr = match attributes & (1 << 4) != 0 {
                            true => 0xFF49,
                            false => 0xFF48,
                        };

                        let color = mem.get_color(color_num, addr);

                        if color == Color::White {
                            continue;
                        }

                        let red;
                        let blue;
                        let green;

                        match color {
                            Color::White => {
                                red = 255;
                                green = 255;
                                blue = 255;
                            }
                            Color::LightGrey => {
                                red = 0xCC;
                                green = 0xCC;
                                blue = 0xCC;
                            }
                            Color::DarkGrey => {
                                red = 0x77;
                                green = 0x77;
                                blue = 0x77;
                            }
                            Color::Black => {
                                red = 0;
                                green = 0;
                                blue = 0;
                            }
                        }

                        let x_pix = 7 - tile_pixel;
                        let pixel = x_pos + x_pix;

                        if pixel < 0
                            || pixel >= SCREEN_WIDTH as i32
                            || scanline < 0
                            || scanline >= SCREEN_HEIGHT as i32
                        {
                            continue;
                        }

                        if attributes & (1 << 7) != 0 {
                            continue; // TODO: Add packground handling
                        }

                        let idx = (scanline as usize * SCREEN_WIDTH as usize + pixel as usize) * 3;
                        self.buffer[idx] = red;
                        self.buffer[idx + 1] = green;
                        self.buffer[idx + 2] = blue;
                    }
                }
            }
        }
    }

    fn set_lcd_status(&mut self) {
        // debug_println!("Updating LCD Status!");

        let lcd_enabled = self.is_lcd_enabled();
        // Gets lock on memory
        let mut mem = self.device_memory.lock().unwrap();

        let mut status = mem.read_byte(LCD_STATUS);

        // Behavior when lcd is disabled
        if !lcd_enabled {
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

        // debug_println!("Current scanline is {current_line} and mode is {current_mode}");

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

    fn is_lcd_enabled(&mut self) -> bool {
        // Check bit 7 of LCD Control register (0xFF40)
        // debug_println!("Checking if LCD is enabled");
        let mem = self.device_memory.lock().unwrap();
        // debug_println!("Lock aquired");
        mem.read_byte(LCD_CONTROL) & (1 << 7) != 0
        // debug_println!("Check complete");
    }

    // TODO: Add more methods for drawing, sprites, etc.
}

#[cfg(test)]
mod test {
    use super::*;
    use ntest::timeout;
    use std::sync::{Arc, Mutex};

    #[test]
    #[timeout(10)]
    fn test_is_lcd_enabled() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        {
            let mut m = mem.lock().unwrap();
            m.write_byte(LCD_CONTROL, 0x80);
        }
        let mut screen = Screen::new(Arc::clone(&mem));
        assert!(screen.is_lcd_enabled());

        {
            let mut m = mem.lock().unwrap();
            m.write_byte(LCD_CONTROL, 0x00);
        }
        assert!(!screen.is_lcd_enabled());
    }

    #[test]
    #[timeout(10)]
    fn test_render_tile_indexing() {
        let mem = Arc::new(Mutex::new(Memory::new()));
        mem.lock().unwrap().ram_startup();
        let mut screen = Screen::new(Arc::clone(&mem));

        {
            let mut m = mem.lock().unwrap();
            m.write_byte_forced(CURRENT_SCANLINE, 1);
            m.write_byte_forced(0xFF42, 0);
            m.write_byte_forced(0xFF43, 0);
            m.write_byte_forced(0xFF4A, 0);
            m.write_byte_forced(0xFF4B, 7);
            m.write_byte_forced(0xFF47, 0);
            m.write_byte_forced(0x9800, 0);
            m.write_byte_forced(0x8000, 0);
            m.write_byte_forced(0x8001, 0);
        }

        screen.render_tiles(0x31);

        let correct = (SCREEN_WIDTH as usize) * 3;
        assert_eq!(screen.buffer[correct], 255);
        assert_eq!(screen.buffer[1], 0);
    }
}
