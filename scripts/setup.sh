#!/usr/bin/env bash

sudo apt update
sudo apt install -y build-essential pkg-config libfontconfig1-dev cmake m4

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"

# Set compiler version
rustup default nightly
rustup target add riscv32i-unknown-none-elf

# Build Phantom
git clone git@github.com:phantomzone-org/phantom.git && cd phantom && cargo build

# Run template example
cd compiler-tests/template/
PHANTOM_THREADS=32 PHANTOM_DEBUG=false PHANTOM_VERBOSE_TIMINGS=true cargo run --release