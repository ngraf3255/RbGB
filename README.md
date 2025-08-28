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
From: 18 June 2025 - To: 27 August 2025

Total Time: 18 hrs 23 mins

Rust             14 hrs 48 mins  ███████████████████▓░░░░░   79.14 %
C++              33 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.94 %
Markdown         31 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.81 %
C                27 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.45 %
JSON             23 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.07 %
Other            19 mins         ▒░░░░░░░░░░░░░░░░░░░░░░░░   01.77 %
```

<!--END_SECTION:waka-->

## License

MIT
