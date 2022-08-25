#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd core; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo build --release --target wasm32-unknown-unknown --manifest-path {}/Cargo.toml)
(cd defi; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo build --release --target wasm32-unknown-unknown --manifest-path {}/Cargo.toml)
(cd nft; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo build --release --target wasm32-unknown-unknown --manifest-path {}/Cargo.toml)

(cd core; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo test --manifest-path {}/Cargo.toml)
(cd defi; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo test --manifest-path {}/Cargo.toml)
(cd nft; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' cargo test --manifest-path {}/Cargo.toml)

echo "Congrats! All tests passed."
