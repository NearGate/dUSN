#!/bin/bash
set -e
cd "`dirname $0`"/../ft
cargo build --all --target wasm32-unknown-unknown --release --features mainnet
cd ..
cp ft/target/wasm32-unknown-unknown/release/d_usn.wasm ./res/d_usn.wasm

cd "`dirname $0`"/../ft
cargo build --all --target wasm32-unknown-unknown --release --features testnet
cd ..
cp ft/target/wasm32-unknown-unknown/release/d_usn.wasm ./res/d_usn_testnet.wasm
