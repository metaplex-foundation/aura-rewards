# Development Setup

## Rust
* Built and developed using - Rust stable(`rustc 1.78.0`) or higher.
* If you are working on both `staking` and `rewards` contracts use: </br>
  `rustup override set <toolchain>` to get rid of manual version switching.

## Solana
* Built and developed using - Solana version `1.18.9` or higher.
* To switch Solana version use - `solana-install init <VERSION>`.

## Build and Test
* To build contract use `cargo build-bpf`.
* Run Rust based tests use - `cargo test-bpf`.

## Formating and Linting
* Run `cargo clippy --all-targets --all-features --workspace -- -D warnings` before pushing your changes.
* Run `cargo +nightly fmt` before pushing your changes.
