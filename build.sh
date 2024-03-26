#!/bin/bash

cargo build --target wasm32-unknown-unknown --release
wasm-strip target/wasm32-unknown-unknown/release/blockwars.wasm
wasm-opt -o docs/blockwars.wasm -Oz target/wasm32-unknown-unknown/release/blockwars.wasm
ls -lh docs/blockwars.wasm | awk '{print "\033[0;95mwasm size: " $5 "\033[0m"}'

# testing something
# base64 -i docs/blockwars.wasm -o blockwars.wasm.base64
