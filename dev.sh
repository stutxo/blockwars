#!/bin/bash

cargo build --target wasm32-unknown-unknown --release
wasm-strip target/wasm32-unknown-unknown/release/blockwars.wasm
wasm-opt -o docs/blockwars.wasm -Oz target/wasm32-unknown-unknown/release/blockwars.wasm
ls -lh docs
python3 -m http.server --directory docs 1334


