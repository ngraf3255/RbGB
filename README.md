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
From: 18 June 2025 - To: 23 June 2025

Total Time: 6 hrs 32 mins

Rust       5 hrs 44 mins   ██████████████████████░░░   87.74 %
Markdown   11 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   03.01 %
C          4 mins          ▒░░░░░░░░░░░░░░░░░░░░░░░░   01.09 %
C++        3 mins          ▒░░░░░░░░░░░░░░░░░░░░░░░░   00.79 %
```

<!--END_SECTION:waka-->

## License

MIT
