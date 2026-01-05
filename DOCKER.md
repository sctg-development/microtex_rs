````markdown
# Docker Support for MicroTeX Rust

## Overview

This project now includes full Docker support for building and testing the project on **Ubuntu 24.04 LTS**.

## 📦 Docker files

```
docker/
├── Dockerfile.ubuntu-lts        # Docker image for Ubuntu 24.04 LTS
├── build-and-test.sh           # Automation script
├── validate.sh                 # Validation script
└── README.md                   # Detailed documentation
```

## 🚀 Quick Start

### Option 1: Automated script (Recommended)

```bash
cd docker
./build-and-test.sh
```

The script:
- ✅ Builds the Docker image
- ✅ Runs tests with `--all-features`
- ✅ Verifies LaTeX rendering
- ✅ Prints usage instructions

### Option 2: Manual build

```bash
# Build
docker build -f docker/Dockerfile.ubuntu-lts -t microtex-rs:ubuntu-24.04 .

# Test
docker run --rm microtex-rs:ubuntu-24.04 cargo test --all-features

# Use the CLI
docker run --rm microtex-rs:ubuntu-24.04 '\[E = mc^2\]'
```

## 📋 Contents of `Dockerfile.ubuntu-lts`

### Base image
- **Ubuntu 24.04 LTS** — long-term support (5 years)

### System dependencies installed
```
Build Tools:
  • build-essential (gcc, g++, make)
  • cmake
  • git, curl
  • pkg-config
  • python3

Graphics Libraries:
  • libcairo2-dev
  • libpango1.0-dev
  • libfontconfig1-dev
  • libfreetype6-dev
  • libharfbuzz-dev
  • libglib2.0-dev
```

### Build process
1. Install system dependencies
2. Install Rust (via rustup)
3. Compile the project with `--all-features`
   - Vendored Cairo
   - Vendored Pango
   - Auto-generated Rust bindings
4. Run tests
5. Generate documentation

### Entrypoint
- **Default command**: `./target/debug/microtex --help`
- **Purpose**: Render LaTeX formulae to SVG

## 💻 Usage examples

### Render a simple formula
```bash
docker run --rm microtex-rs:ubuntu-24.04 '\[E = mc^2\]'
```

### Render a complex formula
```bash
docker run --rm microtex-rs:ubuntu-24.04 \
  '\[\iiint\limits_{V} \nabla \cdot \vec{F} \, dV = \iint\limits_{S} \vec{F} \cdot \vec{n} \, dS\]'
```

### Interactive shell
```bash
docker run --rm -it microtex-rs:ubuntu-24.04 /bin/bash
```

### Extract the generated SVG
```bash
docker run --rm microtex-rs:ubuntu-24.04 '\[E = mc^2\]' > output.svg
cat output.svg
```

### Re-run tests
```bash
docker run --rm microtex-rs:ubuntu-24.04 cargo test --all-features
```

## ⏱️ Estimated build times

| Type | Time | Notes |
|------|------|-------|
| **First build** | 5–15 min | Full compilation of Cairo, Pango, MicroTeX |
| **Cached build** | 1–2 min | Docker layers reused |
| **Tests only** | 30–60 sec | No recompilation |

## 🔍 Validation

### Check Dockerfile syntax
```bash
./docker/validate.sh docker/Dockerfile.ubuntu-lts
```

### Check the built image
```bash
docker images | grep microtex-rs
```

### Inspect image contents
```bash
docker run --rm microtex-rs:ubuntu-24.04 ls -la /workspace/target/debug/
```

## 🐛 Troubleshooting

### Error: "Cannot connect to Docker daemon"
- Start Docker Desktop or the Docker service
- Ensure you have the correct permissions (add yourself to the `docker` group if required)

### Error: "Out of memory"
- Increase RAM allocated to Docker (recommended minimum: 4GB)
- Docker Desktop → Settings → Resources → Memory

### Very slow build
- This is normal for the first build
- Use Docker layer caching for subsequent builds
- Check your internet connection for downloads

### Error while compiling Cairo
- Cairo is a complex dependency; builds can sometimes fail
- The Dockerfile includes required system headers
- Ensure you have sufficient RAM and disk space

## 📚 Full documentation

See `docker/README.md` for:
- CI/CD integration instructions
- Advanced configuration
- Detailed troubleshooting
- GitHub Actions integration

## 🎯 Use cases

### Development
```bash
# Interactive build
docker run --rm -it -v $(pwd):/workspace microtex-rs:ubuntu-24.04 /bin/bash

# Continuous testing
docker run --rm microtex-rs:ubuntu-24.04 cargo test --all-features -- --nocapture
```

### CI/CD
```bash
# Unit tests
docker run --rm microtex-rs:ubuntu-24.04 cargo test --lib

# Integration tests
docker run --rm microtex-rs:ubuntu-24.04 cargo test --test '*'

# Linting
docker run --rm microtex-rs:ubuntu-24.04 cargo clippy --all-features
```

### Deployment
```bash
# Build a lightweight image
docker build -f docker/Dockerfile.ubuntu-lts -t microtex-rs:latest .

# Push to a registry
docker tag microtex-rs:ubuntu-24.04 myregistry/microtex-rs:latest
docker push myregistry/microtex-rs:latest
```

## 📝 Notes

- ✅ All tests with `--all-features` pass
- ✅ Vendored dependencies (Cairo, Pango) are compiled
- ✅ Dockerfile optimised for caching
- ✅ Compatible with Docker Desktop and Docker Engine
- ✅ Ubuntu 24.04 LTS provides long-term support

## 🔗 Resources

- [Docker Documentation](https://docs.docker.com/)
- [Ubuntu 24.04 LTS](https://releases.ubuntu.com/24.04/)
- [MicroTeX Project](https://github.com/NanoMichael/MicroTeX)

---

**Created for the MicroTeX Rust project — MIT OR Apache-2.0**

````
