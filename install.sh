#!/bin/bash
set -e
echo "Welcome to the PIFMTX installer!"

echo "=== Updating system packages ==="
sudo apt update && sudo apt upgrade -y

echo "=== Installing dependencies ==="
sudo apt install -y sox libsox-fmt-mp3 git build-essential curl pkg-config libssl-dev

echo "=== Installing Rust ==="
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

echo "=== Detecting Raspberry Pi model ==="
PI_MODEL=$(tr -d '\0' < /proc/device-tree/model)
echo "Detected model: $PI_MODEL"

REPO_DIR="$HOME/PiFmRds"
if [ ! -d "$REPO_DIR" ]; then
    echo "Cloning PiFM-RDS repository..."
    git clone https://github.com/ChristopheJacquet/PiFmRds.git "$REPO_DIR"
fi

SRC_DIR="$REPO_DIR/src"
cd "$SRC_DIR" || { echo "Failed to enter repository src directory"; exit 1; }

if [[ "$PI_MODEL" == *"Zero 2 W"* ]]; then
    if [ -f Makefile ]; then
        echo "Patching Makefile for Zero 2 W (RPI_VERSION=3)"
        sed -i 's/^RPI_VERSION :=.*/RPI_VERSION = 3/' Makefile
    else
        echo "Makefile not found in src! Cannot patch."
        exit 1
    fi
fi

sed -i 's/^ARCH_CFLAGS.*$/ARCH_CFLAGS = -march=armv7-a -O3 -mtune=arm1176jzf-s -mfloat-abi=hard -mfpu=vfp -ffast-math/' Makefile
sed -i 's/^TARGET = .*$/TARGET = 3/' Makefile

echo "Compiling PiFM-RDS..."
make clean
make

echo "Installing PiFM-RDS to /usr/local/bin..."
sudo cp pi_fm_rds /usr/local/bin/
sudo chmod +x /usr/local/bin/pi_fm_rds

RUST_TX_DIR="$HOME/pifmtx/pifmtx"
RUST_REPO_URL="https://github.com/NotHavocc/pifmtx"

if [ ! -d "$RUST_TX_DIR" ]; then
    echo "Cloning Rust transmitter repository..."
    git clone "$RUST_REPO_URL" "$RUST_TX_DIR"
fi

cd "$RUST_TX_DIR"
echo "Building PIFMTX..."
cargo build --release

echo "Installing PIFMTX..."
sudo cp target/release/pifmtx /usr/local/bin/
sudo chmod +x /usr/local/bin/pifmtx

echo "=== Installation complete! ==="
echo "PiFM-RDS installed"
echo "PIFMTX installed: /usr/local/bin/pifmtx"
echo "You can now run:"
echo "  pifmtx"
