#!/bin/bash

cargo build --target wasm32-unknown-unknown --release
wasm-strip target/wasm32-unknown-unknown/release/b.wasm
wasm-opt -o docs/blockwars.wasm -Oz target/wasm32-unknown-unknown/release/b.wasm
ls -lh docs
python3 -m http.server --directory docs 1334


