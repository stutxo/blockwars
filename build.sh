#!/bin/bash

cargo build --target wasm32-unknown-unknown --release
wasm-strip target/wasm32-unknown-unknown/release/blockwars.wasm
wasm-opt -o docs/blockwars.wasm -Oz target/wasm32-unknown-unknown/release/blockwars.wasm
ls -lh docs/blockwars.wasm


