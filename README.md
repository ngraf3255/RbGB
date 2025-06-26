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

## Coding Statistics

<!--START_SECTION:waka-->

```rust
From: 18 June 2025 - To: 24 June 2025

Total Time: 7 hrs 34 mins

Rust       6 hrs 45 mins   ██████████████████████▒░░   89.13 %
Markdown   11 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.60 %
C++        4 mins          ▒░░░░░░░░░░░░░░░░░░░░░░░░   00.99 %
C          4 mins          ▒░░░░░░░░░░░░░░░░░░░░░░░░   00.94 %
```

<!--END_SECTION:waka-->

## License

MIT
