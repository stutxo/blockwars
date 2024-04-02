#!/bin/bash

set -e # Stop on error

# Build the WASM module
echo "Building WASM module..."
cargo build --target wasm32-unknown-unknown --release

# Strip the WASM file to reduce its size
echo "Stripping WASM file..."
wasm-strip target/wasm32-unknown-unknown/release/blockwars.wasm

# Optimize the WASM file with wasm-opt
echo "Optimizing WASM file..."
wasm-opt -o target/wasm32-unknown-unknown/release/blockwars.wasm -Oz target/wasm32-unknown-unknown/release/blockwars.wasm


# Generate the base64-encoded WASM string
echo "Generating base64-encoded string..."
BASE64_WASM=$(base64 -i target/wasm32-unknown-unknown/release/blockwars.wasm)

# Path to your index.html
INDEX_HTML="docs/index.html"

# Update the index.html with the new base64 string
awk -v base64="$BASE64_WASM" '
    BEGIN {printme=1}
    /\/\/ Base64WasmStart/ {print;printme=0;print "    const base64Wasm = \x27" base64 "\x27;";next}
    /\/\/ Base64WasmEnd/ {printme=1}
    printme {print}
' "$INDEX_HTML" > "$INDEX_HTML".tmp && mv "$INDEX_HTML".tmp "$INDEX_HTML"

echo "Build complete! 🎉 ${BASE64_WASM: -10}"

# Output the file size

ls -lh target/wasm32-unknown-unknown/release/blockwars.wasm | awk '{print "\033[0;95mwasm size: " $5 "\033[0m"}'
