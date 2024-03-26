#!/bin/bash

cargo build --target wasm32-unknown-unknown --release
wasm-strip target/wasm32-unknown-unknown/release/blockwars.wasm
wasm-opt -o docs/blockwars.wasm -Oz target/wasm32-unknown-unknown/release/blockwars.wasm
ls -lh docs/

# testing something
# base64 -i docs/blockwars.wasm -o blockwars.wasm.base64
