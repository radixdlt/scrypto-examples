#!/bin/bash

set -x
set -e

(cd core/cross-blueprint-call; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd core/flat-admin; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd core/gumball-machine; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd core/hello-nft; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd core/hello-world; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd core/managed-access; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd core/no-std-lib; cargo build --target wasm32-unknown-unknown --release)
(cd nft/magic-card; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd nft/sporting-event; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd defi/auto-lend; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd defi/x-perp-futures; cargo build --target wasm32-unknown-unknown --release; cargo test --release)
(cd defi; ./demo.sh)

echo "Congrats! All tests passed."
