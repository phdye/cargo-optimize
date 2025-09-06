#!/bin/bash

# MVP Demo Script - Shows the core concept working
# This demonstrates what cargo-optimize will do automatically
# Works with Windows Rust from Cygwin

echo "=== cargo-optimize MVP Demo ==="
echo "Demonstrating automatic linker optimization"
echo

# Check if we're in a Rust project
if [ ! -f "Cargo.toml" ]; then
    echo "ERROR: Run this from a Rust project root"
    exit 1
fi

# Detect the Rust target platform (not the shell platform)
detect_rust_target() {
    # Try both rustc and rustc.exe (for Cygwin compatibility)
    local rust_cmd="rustc"
    if ! command -v rustc >/dev/null 2>&1; then
        if command -v rustc.exe >/dev/null 2>&1; then
            rust_cmd="rustc.exe"
        fi
    fi
    
    # Get the default target from rustc
    local target=$($rust_cmd --version --verbose 2>/dev/null | grep "host:" | cut -d' ' -f2)
    
    if [[ "$target" == *"windows"* ]]; then
        echo "windows"
    elif [[ "$target" == *"linux"* ]]; then
        echo "linux"
    elif [[ "$target" == *"darwin"* ]]; then
        echo "macos"
    else
        # Fallback to OS detection
        if [[ "$OS" == "Windows_NT" ]] || [[ "$(uname -s)" == CYGWIN* ]] || [[ "$(uname -s)" == MINGW* ]]; then
            echo "windows"
        else
            echo "linux"
        fi
    fi
}

# Function to detect best linker
detect_linker() {
    local target=$(detect_rust_target)
    
    if [ "$target" == "windows" ]; then
        # On Windows, rust-lld is available if Rust is installed
        # Check for rustc or rustc.exe
        if command -v rustc >/dev/null 2>&1 || command -v rustc.exe >/dev/null 2>&1; then
            echo "rust-lld"
        elif command -v lld-link.exe >/dev/null 2>&1 || where lld-link.exe >/dev/null 2>&1; then
            echo "lld-link"
        else
            echo "default"
        fi
    else
        # Linux linkers
        if command -v mold >/dev/null 2>&1; then
            echo "mold"
        elif command -v lld >/dev/null 2>&1; then
            echo "lld"
        elif command -v gold >/dev/null 2>&1; then
            echo "gold"
        else
            echo "default"
        fi
    fi
}

# Function to create config
create_config() {
    local linker=$1
    local target=$(detect_rust_target)
    
    mkdir -p .cargo
    
    if [ "$target" == "windows" ]; then
        # Windows configurations
        case $linker in
            rust-lld)
                cat > .cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-msvc]
linker = "rust-lld"
EOF
                echo "✓ Configured rust-lld linker for Windows (fast!)"
                ;;
            lld-link)
                cat > .cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-msvc]
linker = "lld-link.exe"
EOF
                echo "✓ Configured lld-link linker for Windows (fast!)"
                ;;
            *)
                echo "ℹ Using default MSVC linker (no fast linker found)"
                return 1
                ;;
        esac
    else
        # Linux configurations
        case $linker in
            mold)
                cat > .cargo/config.toml << 'EOF'
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
EOF
                echo "✓ Configured mold linker (fastest!)"
                ;;
            lld)
                cat > .cargo/config.toml << 'EOF'
[target.x86_64-unknown-linux-gnu]
linker = "clang"  
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
EOF
                echo "✓ Configured lld linker (very fast)"
                ;;
            gold)
                cat > .cargo/config.toml << 'EOF'
[target.x86_64-unknown-linux-gnu]
linker = "gold"
EOF
                echo "✓ Configured gold linker (fast)"
                ;;
            *)
                echo "ℹ Using default linker (no fast linker found)"
                return 1
                ;;
        esac
    fi
    
    return 0
}

# Get the rust command that works
get_rust_cmd() {
    if command -v rustc >/dev/null 2>&1; then
        echo "rustc"
    elif command -v rustc.exe >/dev/null 2>&1; then
        echo "rustc.exe"
    else
        echo ""
    fi
}

# Main logic
echo "1. Detecting Rust target and available linkers..."
TARGET=$(detect_rust_target)
echo "   Rust target platform: $TARGET"

# Show the actual Rust host triple for clarity
RUST_CMD=$(get_rust_cmd)
if [ -n "$RUST_CMD" ]; then
    RUST_HOST=$($RUST_CMD --version --verbose 2>/dev/null | grep "host:" | cut -d' ' -f2)
    echo "   Rust host triple: $RUST_HOST"
fi

LINKER=$(detect_linker)
echo "   Best available linker: $LINKER"

if [ "$LINKER" != "default" ]; then
    echo
    echo "2. Creating optimized configuration..."
    if create_config "$LINKER"; then
        echo
        echo "3. Configuration created! Next steps:"
        echo "   cargo clean"
        echo "   cargo build --release"
        echo
        echo "=== Success! ==="
        echo "Your project is now configured to use the $LINKER linker."
        echo "Future builds will be significantly faster!"
        echo
        echo "To compare with default linker:"
        echo "  1. mv .cargo/config.toml .cargo/config.toml.optimized"
        echo "  2. cargo clean && time cargo build --release"
        echo "  3. mv .cargo/config.toml.optimized .cargo/config.toml"
        echo "  4. cargo clean && time cargo build --release"
    fi
else
    echo
    if [ "$TARGET" == "windows" ]; then
        echo "No fast linker found. To install:"
        echo "  - rust-lld: rustup component add llvm-tools-preview"
        echo "  - lld: winget install LLVM.LLVM"
    else
        echo "No fast linker found. Install one for better performance:"
        echo "  - mold: https://github.com/rui314/mold"
        echo "  - lld: apt/yum install lld"
        echo "  - gold: apt/yum install binutils-gold"
    fi
fi
