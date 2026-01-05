#!/bin/bash

# MicroTeX Rust - Multi-platform Docker buildx test script
# Supports: linux/amd64, linux/arm64

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "╔════════════════════════════════════════════════════════════╗"
echo "║       MicroTeX Rust - Multi-Platform Docker Build          ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Platforms to build
PLATFORMS="linux/amd64,linux/arm64"

# Test configurations
CONFIGS=(
    "ubuntu|docker/Dockerfile.ubuntu-lts|microtex-rs:ubuntu-24.04"
    "alpine|docker/Dockerfile.alpine|microtex-rs:alpine-3.23"
)

echo "📦 Building for platforms: $PLATFORMS"
echo ""

for config in "${CONFIGS[@]}"; do
    IFS='|' read -r name dockerfile tag <<< "$config"
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🐳 Testing: $name"
    echo "📄 Dockerfile: $dockerfile"
    echo "🏷️  Tag: $tag"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Build for multiple platforms
    if docker buildx build \
        --file "$PROJECT_ROOT/$dockerfile" \
        --tag "$tag" \
        --platform "$PLATFORMS" \
        --progress=plain \
        "$PROJECT_ROOT"; then
        echo "✅ $name build successful"
    else
        echo "❌ $name build failed"
        exit 1
    fi
    echo ""
done

echo "╔════════════════════════════════════════════════════════════╗"
echo "║              ✅ All platforms built successfully            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "📝 Next steps:"
echo "  1. Test locally: docker run --rm microtex-rs:ubuntu-24.04 --help"
echo "  2. Test Alpine: docker run --rm microtex-rs:alpine-3.23 --help"
echo "  3. Load into Docker: docker buildx build --load -t microtex-rs:test ."
echo ""
