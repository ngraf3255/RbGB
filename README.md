# rbgb

A simple Game Boy emulator written in Rust.

## Features

- CPU emulation (Sharp LR35902)
- Shared Memory bank
- Basic graphics rendering
- ROM loading

## Repo Basics

1. Clone the repository:

    ```bash
    git clone https://github.com/Hyphen325/rbgb.git
    cd rbgb
    ```

2. Build the project:

    ```bash
    cargo build --release
    ```

3. Run the emulator with a ROM:

    ```bash
    cargo run --release
    ```

4. Select a ROM \
Load a rom by pressing L on the opened window for the ROM path prompt

## Requirements

- Rust (latest stable)
- SDL2 or Docker

## Running Tests

To build and run tests locally:

1. Clone the repository:

    ```bash
    git clone https://github.com/Hyphen325/rbgb.git
    cd rbgb
    ```

2. Run the tests:

    ```bash
    cargo test
    ```

## Outstanding Work

- Implement handling of poisoned Mutexes
- Implement full multithreading
- Modify register system to allow them to be set more gracefully
- Implement library features for emulator and drop sdl to allow for apps to implement on top of the emulator

## Acknowledgements

Much of this was written and based off of [Codeslinger Gameboy](http://www.codeslinger.co.uk/pages/projects/gameboy/beginning.html). I needed this guide to get through most of this. Additionally, I learned about how the RZ80 worked from [RZ80](https://floooh.github.io/) and modified the CPU implementation of the LR35902 from  [rboy](https://github.com/mvdnes/rboy). Getting the CPU emulation to work was the hardest part of this project.

## License

MIT
