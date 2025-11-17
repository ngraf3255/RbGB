use crate::types::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::pixels::PixelFormatEnum;

pub fn sdl_init() -> Result<(), String> {
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

    Ok(())
}
