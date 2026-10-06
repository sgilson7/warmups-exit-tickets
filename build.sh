#!/usr/bin/env bash
# Build the wasm module from crates/wasm and assemble the static site in dist/web.
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown -p warmups-wasm

rm -rf dist/web
mkdir -p dist/web/pkg dist/web/data
wasm-bindgen --target web --no-typescript --out-dir dist/web/pkg \
  target/wasm32-unknown-unknown/release/warmups_wasm.wasm
cp web/index.html web/app.js web/style.css dist/web/
cp data/*.json dist/web/data/

echo "Built dist/web:"
find dist/web -type f | sort
