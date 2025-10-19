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
From: 18 June 2025 - To: 18 October 2025

Total Time: 62 hrs 55 mins

C                24 hrs 59 mins  █████████▓░░░░░░░░░░░░░░░   38.80 %
Rust             17 hrs 38 mins  ███████░░░░░░░░░░░░░░░░░░   27.39 %
HTML             3 hrs 13 mins   █▒░░░░░░░░░░░░░░░░░░░░░░░   05.01 %
Python           3 hrs 1 min     █▒░░░░░░░░░░░░░░░░░░░░░░░   04.70 %
JSON             2 hrs 43 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   04.24 %
C++              2 hrs 38 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   04.12 %
JavaScript       2 hrs 36 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   04.06 %
Markdown         1 hr 31 mins    ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.37 %
Other            1 hr 27 mins    ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.26 %
```

<!--END_SECTION:waka-->

## License

MIT
