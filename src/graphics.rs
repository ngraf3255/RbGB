#![allow(dead_code)]

pub struct Screen {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u8>, // Each pixel is a byte (0-3 for Game Boy palettes)
}

impl Screen {
    pub const WIDTH: usize = 160;
    pub const HEIGHT: usize = 144;

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

    pub fn update_screen(&self) {
        //TODO: Create function for updating screen at 60Hz
        unimplemented!();
    }

    // TODO: Add more methods for drawing, sprites, etc.
}
