#!/bin/bash
set -e

echo "Building fmd ..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Determine installation method
INSTALL_METHOD="${1:-cargo}"

if [ "$INSTALL_METHOD" = "cargo" ]; then
    echo "Installing via 'cargo install --path .' to ~/.cargo/bin..."
    cargo install --path .
    echo ""
    echo "✓ Installation complete!"
    echo "Binary installed at: $HOME/.cargo/bin/fmd"
    echo "Try: fmd --help"
elif [ "$INSTALL_METHOD" = "local" ]; then
    # Build release binary
    echo "Compiling with optimizations..."
    cargo build --release

    # Install to ~/.local/bin
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"

    echo "Installing to $INSTALL_DIR/fmd..."
    cp ./target/release/fmd "$INSTALL_DIR/fmd"

    # Check if ~/.local/bin is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        echo "⚠️  $INSTALL_DIR is not in your PATH."
        echo "Add this line to your ~/.bashrc or ~/.zshrc:"
        echo ""
        echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
    else
        echo "✓ Installation complete!"
    fi

    echo ""
    echo "Binary installed at: $INSTALL_DIR/fmd"
    echo "Try: fmd --help"
else
    echo "Usage: $0 [cargo|local]"
    echo "  cargo (default): Install to ~/.cargo/bin using 'cargo install'"
    echo "  local:          Install to ~/.local/bin by copying binary"
    exit 1
fi
