#!/usr/bin/env bash

set -e

REPO_URL="https://github.com/phantomzone-org/poulpy"
CLONE_DIR="poulpy"

cd ..
echo "Cloning the poulpy repository..."
git clone "$REPO_URL" "$CLONE_DIR"
cd "$CLONE_DIR"
git checkout phantom-dev
git pull

echo "Initializing submodule: base2k/spqlios-arithmetic..."
git submodule update --init base2k/spqlios-arithmetic

echo "Building and installing spqlios-arithmetic..."

cd base2k/spqlios-arithmetic

# Detect platform
OS="$(uname -s)"

case "$OS" in
    Linux*)
        echo "Detected Linux"
        mkdir build
        cd build
        cmake .. -DENABLE_TESTING=off
        make
        ;;
    Darwin*)
        echo "Detected macOS"
        mkdir build
        cd build
        cmake .. -DENABLE_TESTING=off
        make
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Installation complete."
# cd ../../../phantom
