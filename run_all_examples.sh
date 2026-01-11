#!/bin/bash
for ex in $(ls examples/*.rs); do
    name=$(basename "$ex" .rs)
    echo "=== Exécution de $name ==="
    cargo run --example "$name" --all-features || break
done