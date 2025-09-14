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
From: 18 June 2025 - To: 13 September 2025

Total Time: 31 hrs 33 mins

Rust             14 hrs 49 mins  ███████████▒░░░░░░░░░░░░░   45.22 %
C                6 hrs 45 mins   █████░░░░░░░░░░░░░░░░░░░░   20.62 %
JSON             2 hrs 40 mins   ██░░░░░░░░░░░░░░░░░░░░░░░   08.16 %
Python           2 hrs 33 mins   ██░░░░░░░░░░░░░░░░░░░░░░░   07.80 %
Other            1 hr 13 mins    █░░░░░░░░░░░░░░░░░░░░░░░░   03.71 %
```

<!--END_SECTION:waka-->

## License

MIT
