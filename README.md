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
From: 18 June 2025 - To: 17 November 2025

Total Time: 77 hrs 13 mins

C                29 hrs 46 mins  █████████▒░░░░░░░░░░░░░░░   37.72 %
Rust             19 hrs 31 mins  ██████▒░░░░░░░░░░░░░░░░░░   24.74 %
C++              5 hrs 24 mins   █▓░░░░░░░░░░░░░░░░░░░░░░░   06.85 %
Python           4 hrs 38 mins   █▒░░░░░░░░░░░░░░░░░░░░░░░   05.89 %
HTML             3 hrs 17 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   04.17 %
Markdown         3 hrs 3 mins    █░░░░░░░░░░░░░░░░░░░░░░░░   03.88 %
JSON             2 hrs 48 mins   █░░░░░░░░░░░░░░░░░░░░░░░░   03.55 %
JavaScript       2 hrs 36 mins   ▓░░░░░░░░░░░░░░░░░░░░░░░░   03.31 %
Other            1 hr 42 mins    ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.17 %
```

<!--END_SECTION:waka-->

## License

MIT
