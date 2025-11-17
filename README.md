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

## Coding Statistics

<!--START_SECTION:waka-->

```rust
From: 18 June 2025 - To: 16 November 2025

Total Time: 74 hrs 16 mins

C                28 hrs 48 mins  █████████▒░░░░░░░░░░░░░░░   37.91 %
Rust             17 hrs 38 mins  █████▓░░░░░░░░░░░░░░░░░░░   23.20 %
C++              5 hrs 24 mins   █▓░░░░░░░░░░░░░░░░░░░░░░░   07.12 %
Python           4 hrs 38 mins   █▓░░░░░░░░░░░░░░░░░░░░░░░   06.12 %
HTML             3 hrs 17 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   04.33 %
Markdown         3 hrs           █░░░░░░░░░░░░░░░░░░░░░░░░   03.96 %
JSON             2 hrs 48 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   03.69 %
JavaScript       2 hrs 36 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   03.44 %
Other            1 hr 42 mins    ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.25 %
```

<!--END_SECTION:waka-->

## License

MIT
