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
    cargo run --release -- path/to/rom.gb
    ```

Working on a clearer deployment system, likely by implementing the github release feature.

## Requirements

- Rust (latest stable)

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
- Add DOCKERFILE to containerize (annoyed me when sdl2 wasn't installing)
- Trim workflow to reflect new docker container
- Implement custom cpu opcode handling

## Acknowledgements

Much of this was written and based off of [Codeslinger Gameboy](http://www.codeslinger.co.uk/pages/projects/gameboy/beginning.html). I needed this guide to get through most of this. Additionally, I learned about how the RZ80 worked from [RZ80](https://floooh.github.io/) and modified the CPU implementation of the LR35902 from  [rboy](https://github.com/mvdnes/rboy). Getting the CPU emulation to work was the hardest part of this project.

## License

MIT
