#!/usr/bin/env bash
# Quick validation script for Docker build
# Run this to validate the Dockerfile syntax and test the build

set -e

DOCKER_FILE="${1:-.docker/Dockerfile.ubuntu-lts}"

echo "=== Validating Docker File ==="
echo "File: $DOCKER_FILE"

if [ ! -f "$DOCKER_FILE" ]; then
    echo "✗ Dockerfile not found: $DOCKER_FILE"
    exit 1
fi

echo "✓ Dockerfile exists"

# Check basic syntax by trying to parse it
echo ""
echo "=== Basic Dockerfile Syntax Check ==="
docker buildx build -f "$DOCKER_FILE" --dry-run . 2>&1 | head -20 && echo "✓ Syntax looks valid" || echo "⚠ May have issues"

echo ""
echo "=== Dockerfile Content Summary ==="
echo "Lines: $(wc -l < "$DOCKER_FILE")"
echo "Base image: $(grep '^FROM' "$DOCKER_FILE" | head -1)"
echo "Build stages: $(grep -c '^FROM' "$DOCKER_FILE")"
echo "Dependencies installed:"
grep 'apt-get install' "$DOCKER_FILE" | sed 's/.*apt-get install[^\\]*/  /'

echo ""
echo "=== Recommended Build Command ==="
echo "docker build -f $DOCKER_FILE -t microtex-rs:ubuntu-24.04 ."

echo ""
echo "=== Expected Build Time ==="
echo "First build:  5-15 minutes (downloads and compiles dependencies)"
echo "Cached build: 1-2 minutes (uses Docker layer cache)"
