# AGENTS.md

This repository is an embedded Rust firmware for a 4-motor controller using
Embassy on STM32. It is `no_std`, async, and configured for an ARM Cortex-M
target. Use the guidance below when working in this repo.

## Quick commands

Build (default target is set in `.cargo/config.toml`):
- `cargo build`
- `cargo build --release`

Flash/run (uses probe-rs runner from `.cargo/config.toml`):
- `cargo run`

Format:
- `cargo fmt`

Lint (clippy):
- `cargo clippy --all-targets --all-features`

Tests:
- `cargo test` (note: no tests exist and embedded targets may not support tests)
- `cargo test <test_name>` for a single test when host tests are added
- `cargo test <test_name> -- --nocapture` to see logs for a single test

## Build environment

- Target: `thumbv7em-none-eabi` (set in `.cargo/config.toml`).
- Runner: `probe-rs run --chip STM32F411Ceux` (set in `.cargo/config.toml`).
- The firmware uses Embassy (`embassy-*` crates) and `defmt` for logging.
- The project is `no_std` and `no_main`; assume no allocator unless explicitly
  added.

## Code layout

- `src/main.rs`: hardware init, embassy tasks, SPI handling, main loop.
- `src/car_ctrl.rs`: motor control logic and state tracking.
- `src/registry.rs`: register map, registry storage, and validation.
- `src/spi_proto.rs`: SPI protocol encoding/decoding and CRC helpers.

## Style and conventions

### Formatting
- Use `cargo fmt` and default rustfmt rules; avoid manual alignment.
- Keep line lengths reasonable; prefer wrapping long function calls.

### Imports
- Group imports by crate: std/core (if used), external crates, then local crate.
- Use `use crate::...` for local modules; avoid glob imports except for prelude
  style crates already used (`defmt::*` in `main.rs`).
- Keep nested imports aligned and alphabetized within the group when practical.

### Naming
- `snake_case` for functions, variables, and modules.
- `UpperCamelCase` for types and enums (e.g., `ControlMode`, `RegisterID`).
- `SCREAMING_SNAKE_CASE` for constants (e.g., `MOTOR_MAX_PWM_DUTY_CYCLE`).
- Hardware-related constants should be `u32`/`i32` depending on usage and match
  protocol types.

### Types and representation
- Protocol-facing enums use explicit `#[repr(u8)]` or `#[repr(u32)]`.
- Prefer explicit integer sizes for data on the wire (`u8`, `u16`, `u32`, `i32`).
- Use `#[repr(C)]` for structs/unions exchanged as bytes.
- When converting bytes, keep endianness explicit (`to_le_bytes`, `from_le_bytes`).

### Ownership and borrowing
- Keep borrow scopes small around `Mutex` locks (`REGISTRY.lock().await`), then
  copy the required data out.
- Prefer immutable borrows where possible; only lock mutably when required.

### Error handling
- Use `Result<T, ErrorCode>` for protocol and validation errors.
- Convert between primitive values and enums via `TryFrom` and return specific
  `ErrorCode` values on validation errors.
- For recoverable runtime errors, log with `defmt::error!` and continue.
- Use `assert!` for programmer errors or invariant checks (e.g., buffer sizes).

### Logging
- Use `defmt::{info, error, trace}` macros; avoid `println!`.
- Keep logs short; include context like register ids or motor names.
- Avoid logging in tight loops unless gated or clearly necessary.

### Async tasks
- Tasks are declared with `#[embassy_executor::task]`.
- Do not block inside tasks; use `Ticker` or async I/O as in `main.rs`.
- Maintain periodic control loops via `Ticker::every` and `await`.

### Concurrency and shared state
- Shared state lives in `REGISTRY` (a `Mutex<ThreadModeRawMutex, Registry>`).
- Hold the lock only for the duration needed to update or read data.
- Do not keep references to registry data across await points.

### Protocol and registry
- SPI protocol uses a header with reg/len/crc and data with CRC.
- Validate CRC and register ranges before mutating registry.
- Keep register access permissions in sync with `ControlMode` updates.
- Use `ErrorCode` to reflect protocol errors; update last error register.

### Hardware init
- All pin configuration and peripheral setup happens in `build_car_hw_cfg()`.
- Maintain a clear mapping of timers/pins to motors; update comments if changes.
- Ensure PWM channels and QEI timers match the MCU pinout.

## Repo rules and tooling

- No Cursor rules found in `.cursor/rules/` or `.cursorrules`.
- No GitHub Copilot instructions found in `.github/copilot-instructions.md`.

## Notes for contributors

- This is embedded firmware; host-based tests are not currently present.
- If adding tests, keep them `#[cfg(test)]` and consider `cargo test` on host
  with mocked HALs or feature-gated modules.
- When adding new registers, update `RegisterID`, `RegisterConfig`, and
  the range validation logic in `registry.rs`.
- Keep `REGISTERS_COUNT` and `REGISTERS_SIZE_BYTES` consistent with the
  registry layout in `registry.rs` and protocol expectations.
