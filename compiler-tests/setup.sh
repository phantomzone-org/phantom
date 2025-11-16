#!/usr/bin/env bash

sudo apt update
sudo apt install -y build-essential pkg-config libfontconfig1-dev cmake m4

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
rustup default nightly
rustup target add riscv32i-unknown-none-elf

git clone git@github.com:phantomzone-org/poulpy.git
cd poulpy && git submodule update --init --recursive && cargo build && cd ..
git clone git@github.com:phantomzone-org/phantom.git
cd phantom && git checkout jay/improve-docs && cargo build
cd compiler-tests/template/
PHANTOM_THREADS=32 PHANTOM_VERBOSE_TIMINGS=true cargo run --release