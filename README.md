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
From: 18 June 2025 - To: 25 June 2025

Total Time: 8 hrs 38 mins

Rust       7 hrs 30 mins   █████████████████████▓░░░   86.88 %
C++        22 mins         █░░░░░░░░░░░░░░░░░░░░░░░░   04.43 %
Markdown   11 mins         ▓░░░░░░░░░░░░░░░░░░░░░░░░   02.28 %
C          4 mins          ▒░░░░░░░░░░░░░░░░░░░░░░░░   00.83 %
Other      0 secs          ░░░░░░░░░░░░░░░░░░░░░░░░░   00.00 %
```

<!--END_SECTION:waka-->

## License

MIT
