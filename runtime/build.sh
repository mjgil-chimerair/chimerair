#!/bin/bash
# Build script for Chimera runtime
# Compiles C source files into object files and a static library

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
SRC_DIR="$SCRIPT_DIR/src"
INCLUDE_DIR="$SCRIPT_DIR/include"

echo "=== Building Chimera Runtime ==="

# Create build directory
mkdir -p "$BUILD_DIR"

# Source files to compile
SRC_FILES=(
    "chimera_error.c"
    "chimera_allocator.c"
    "chimera_sanitizers.c"
)

# Compiler flags
CFLAGS="-Wall -Wextra -O2 -I$INCLUDE_DIR -fPIC"

# Compile each source file
for src in "${SRC_FILES[@]}"; do
    echo "  Compiling $src..."
    gcc $CFLAGS -c "$SRC_DIR/$src" -o "$BUILD_DIR/${src%.c}.o"
done

# Create static library
echo "  Creating libchimera-rt.a..."
ar rcs "$BUILD_DIR/libchimera-rt.a" "$BUILD_DIR"/*.o

echo "  Build complete: $BUILD_DIR"
echo ""
echo "Files in build directory:"
ls -la "$BUILD_DIR/"