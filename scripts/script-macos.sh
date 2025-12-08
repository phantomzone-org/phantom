#!/usr/bin/env bash

set -e

echo "Setting up Phantom on macOS..."

# Check if Homebrew is installed
if ! command -v brew &>/dev/null; then
  echo "Homebrew not found. Installing Homebrew..."
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install system dependencies
echo "Installing system dependencies via Homebrew..."
brew install cmake pkg-config fontconfig m4

# Check if rustup is already installed
if command -v rustup &>/dev/null; then
  echo "rustup is already installed."
else
  # Check if Homebrew Rust is installed
  if brew list rust &>/dev/null 2>&1; then
    echo "WARNING: Homebrew Rust detected. This project requires rustup."
    echo "Please uninstall Homebrew Rust first:"
    echo "  brew uninstall rust rust-analyzer"
    echo "Then re-run this script."
    exit 1
  fi

  # Install Rust via rustup
  echo "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

# Set compiler version to nightly
echo "Setting Rust toolchain to nightly..."
rustup default nightly

# Add RISC-V target (required for compiling guest programs)
echo "Adding RISC-V target..."
rustup target add riscv32i-unknown-none-elf

# Clone repo and build
echo "Adding RISC-V target..."
git clone https://github.com/phantomzone-org/phantom.git && cd phantom && cargo build

# Run template example
cd compiler-tests/template/
PHANTOM_THREADS=$(sysctl -n hw.ncpu) PHANTOM_DEBUG=false PHANTOM_VERBOSE_TIMINGS=true cargo run --release
