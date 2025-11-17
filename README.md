# rbgb

A simple Game Boy emulator written in Rust.

## Features

- CPU emulation (Sharp LR35902)
- Memory management
- Basic graphics rendering
- ROM loading

## Getting Started

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

## Coding Statistics

<!--START_SECTION:waka-->

```rust
From: 18 June 2025 - To: 26 June 2025

Total Time: 10 hrs 17 mins

Rust       8 hrs 51 mins   █████████████████████▒░░░   85.97 %
C++        22 mins         █░░░░░░░░░░░░░░░░░░░░░░░░   03.72 %
Markdown   15 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.59 %
C          4 mins          ▒░░░░░░░░░░░░░░░░░░░░░░░░   00.69 %
Other      0 secs          ░░░░░░░░░░░░░░░░░░░░░░░░░   00.00 %
```

<!--END_SECTION:waka-->

## Acknowledgements

Much of this was written and based off of [Codeslinger Gameboy](http://www.codeslinger.co.uk/pages/projects/gameboy/beginning.html). I needed this guide to get through most of this. Additionally, code for cpu instructions was modified from [RZ80](https://floooh.github.io/) <https://github.com/mvdnes/rboy>

## License

MIT
