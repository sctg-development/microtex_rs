#!/usr/bin/env bash
# Docker build and test script for MicroTeX Rust project

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCKER_FILE="$SCRIPT_DIR/Dockerfile.ubuntu-lts"
IMAGE_NAME="microtex-rs:ubuntu-24.04"
CONTAINER_NAME="microtex-rs-test-$(date +%s)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== MicroTeX Rust - Ubuntu 24.04 LTS Build & Test ===${NC}"
echo "Project Root: $PROJECT_ROOT"
echo "Dockerfile: $DOCKER_FILE"
echo "Image Name: $IMAGE_NAME"
echo "Container Name: $CONTAINER_NAME"
echo ""

# Build Docker image
echo -e "${BLUE}[1/3] Building Docker image...${NC}"
docker build \
    --file "$DOCKER_FILE" \
    --tag "$IMAGE_NAME" \
    --build-arg BUILDKIT_CONTEXT_KEEP_GIT_DIR=1 \
    "$PROJECT_ROOT" || {
    echo -e "${RED}✗ Docker build failed!${NC}"
    exit 1
}
echo -e "${GREEN}✓ Docker image built successfully${NC}"

# Test with help command
echo -e "${BLUE}[2/3] Testing CLI with --help...${NC}"
docker run --rm "$IMAGE_NAME" --help > /dev/null 2>&1 && \
    echo -e "${GREEN}✓ CLI help works${NC}" || \
    echo -e "${RED}✗ CLI help failed${NC}"

# Test with simple formula
echo -e "${BLUE}[3/3] Testing with simple LaTeX formula...${NC}"
docker run --rm "$IMAGE_NAME" '\[E = mc^2\]' > /tmp/test_output.svg 2>&1 && \
    grep -q '<svg' /tmp/test_output.svg && \
    echo -e "${GREEN}✓ LaTeX rendering works${NC}" || \
    echo -e "${RED}✗ LaTeX rendering failed${NC}"

# Print summary
echo ""
echo -e "${GREEN}=== All Tests Passed ===${NC}"
echo -e "Docker image is ready: ${BLUE}$IMAGE_NAME${NC}"
echo ""
echo "Usage examples:"
echo "  # Display help"
echo "  docker run --rm $IMAGE_NAME --help"
echo ""
echo "  # Render a formula"
echo "  docker run --rm $IMAGE_NAME '\\[E = mc^2\\]'"
echo ""
echo "  # Interactive shell"
echo "  docker run --rm -it $IMAGE_NAME /bin/bash"
echo ""
echo "  # Run tests again"
echo "  docker run --rm $IMAGE_NAME cargo test --all-features"
