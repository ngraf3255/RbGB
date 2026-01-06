//! Helper file for annoying and long IO functions

use crate::{
    Emulator,
    types::{GameInput, KeyState},
};
use sdl2::{event::Event, keyboard::Keycode};

pub fn handle_joystick_input(event: Event, emulator: &mut Emulator) {
    match event {
        // Register key down inputs
        Event::KeyDown {
            keycode: Some(key), ..
        } => emulator.game_input(register_key(key), KeyState::Pressed),

        // Register key up inputs
        Event::KeyUp {
            keycode: Some(key), ..
        } => emulator.game_input(register_key(key), KeyState::Released),

        // otherwise do nothing
        _ => (),
    }
}

fn register_key(key: Keycode) -> GameInput {
    match key {
        // Go up
        Keycode::W => GameInput::Up,

        // Go down
        Keycode::S => GameInput::Down,

        // Go left
        Keycode::A => GameInput::Left,

        // Go right
        Keycode::D => GameInput::Right,

        // Go stop
        Keycode::Q => GameInput::Select,

        // Go start
        Keycode::E => GameInput::Start,

        // Go A
        Keycode::Z => GameInput::A,
        // Go B
        Keycode::X => GameInput::B,

        // gets ignored
        _ => GameInput::Unknown,
    }
}
