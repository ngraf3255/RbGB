use std::{thread, time::{Duration, Instant}};

use crate::{types::{SCREEN_HEIGHT, SCREEN_WIDTH}, Emulator};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
};

// Window size multiplier so original 160x144 framebuffer is easier to see
const WINDOW_SCALE: u32 = 5;

pub struct SdlApp {
    _sdl_context: sdl2::Sdl,
    event_pump: sdl2::EventPump,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
}

impl SdlApp {
    pub fn new() -> Result<Self, String> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "Gameboy Emulator",
                SCREEN_WIDTH * WINDOW_SCALE,
                SCREEN_HEIGHT * WINDOW_SCALE,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .build()
            .map_err(|e| e.to_string())?;

        let event_pump = sdl_context.event_pump()?;

        Ok(Self {
            _sdl_context: sdl_context,
            event_pump,
            canvas,
        })
    }

    pub fn run(&mut self, emulator: &mut Emulator) -> Result<(), String> {
        // Create the streaming texture once per run so we can push raw RGB data to it
        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|e| e.to_string())?;

        'running: loop {
            let frame_start = Instant::now();

            // Process all queued SDL events before running a frame
            while let Some(event) = self.event_pump.poll_event() {
                if !Self::handle_event(event, emulator) {
                    break 'running;
                }
            }

            // Advance the emulator state, copy the LCD buffer into SDL, then render
            emulator.update();
            emulator.blit_rgb_bytes_to_texture(&mut texture)?;

            self.draw(emulator.is_paused(), &texture)?;
            self.limit_frame_rate(frame_start);
        }

        Ok(())
    }

    // Returns false when the emulator should stop running (e.g. window closed)
    fn handle_event(event: Event, emulator: &mut Emulator) -> bool {
        match event {
            Event::Quit { .. } => false,
            Event::KeyDown {
                keycode: Some(Keycode::P),
                ..
            } => {
                emulator.toggle_pause();
                true
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
                true
            }
            Event::KeyDown {
                keycode: Some(Keycode::O),
                ..
            } => {
                emulator.dump_lcd_mem();
                true
            }
            _ => true,
        }
    }

    fn draw(&mut self, paused: bool, texture: &sdl2::render::Texture) -> Result<(), String> {
        self.canvas.clear();
        self.canvas.copy(
            texture,
            None,
            Some(Rect::new(
                0,
                0,
                SCREEN_WIDTH * WINDOW_SCALE,
                SCREEN_HEIGHT * WINDOW_SCALE,
            )),
        )?;

        if paused {
            self.canvas.set_draw_color(Color::RGB(50, 50, 50));
            self.canvas.fill_rect(Rect::new(
                0,
                0,
                SCREEN_WIDTH * WINDOW_SCALE,
                SCREEN_HEIGHT * WINDOW_SCALE,
            ))?;
        }

        self.canvas.present();
        Ok(())
    }

    // Simple 60 FPS limiter so SDL doesn't run as fast as possible
    fn limit_frame_rate(&self, frame_start: Instant) {
        let frame_duration = frame_start.elapsed();
        if frame_duration < Duration::from_millis(16) {
            thread::sleep(Duration::from_millis(16) - frame_duration);
        }
    }
}
