# Docker Build Guide - MicroTeX Rust

Ce répertoire contient les fichiers Docker pour construire et tester MicroTeX Rust sur Ubuntu 24.04 LTS.

## Fichiers

- **`Dockerfile.ubuntu-lts`** - Dockerfile pour Ubuntu 24.04 LTS
  - Installe toutes les dépendances système
  - Compile le projet avec `--all-features`
  - Exécute tous les tests
  - Point d'entrée: CLI MicroTeX

- **`build-and-test.sh`** - Script d'automatisation
  - Construit l'image Docker
  - Valide les tests
  - Teste le rendu LaTeX simple

## Utilisation Rapide

### Option 1: Script d'automatisation (recommandé)

```bash
cd docker
````markdown
# Docker Build Guide — MicroTeX Rust

This directory contains the Docker files used to build and test MicroTeX Rust on Ubuntu 24.04 LTS.

## Files

- **`Dockerfile.ubuntu-lts`** — Dockerfile for Ubuntu 24.04 LTS
  - Installs all system dependencies
  - Builds the project with `--all-features`
  - Runs the test suite
  - Entrypoint: MicroTeX CLI

- **`build-and-test.sh`** — Automation script
  - Builds the Docker image
  - Runs the test suite
  - Performs a simple LaTeX render test

## Quick start

### Option 1: Automation script (recommended)

```bash
cd docker
chmod +x build-and-test.sh
./build-and-test.sh
```

### Option 2: Manual build

```bash
# Build the image
docker build -f docker/Dockerfile.ubuntu-lts -t microtex-rs:ubuntu-24.04 .

# Run the tests
docker run --rm microtex-rs:ubuntu-24.04 cargo test --all-features

# Use the CLI
docker run --rm microtex-rs:ubuntu-24.04 '\[E = mc^2\]'
```

## Build details

### Installed dependencies

**Build tools:**
- build-essential (gcc, g++, make)
- cmake
- git
- curl

**Graphics libraries:**
- libcairo2-dev
- libpango1.0-dev
- libfontconfig1-dev
- libfreetype6-dev
- libharfbuzz-dev
- libglib2.0-dev

**Other:**
- pkg-config
- python3
- ca-certificates (for HTTPS)

### Build process

1. Installation of system dependencies (~30–60 seconds)
2. Installation of Rust (~20–40 seconds)
3. Project compilation (varies depending on Docker cache, 2–10 minutes)
   - Downloads and builds Cairo
   - Builds Pango
   - Builds the MicroTeX library
   - Builds the Rust bindings
4. Running the tests (30–60 seconds)
   - Unit tests
   - Integration tests
   - Doc tests

### Usage examples

```bash
# Show help
docker run --rm microtex-rs:ubuntu-24.04 --help

# Render a simple formula
docker run --rm microtex-rs:ubuntu-24.04 '\[E = mc^2\]'

# Render the divergence theorem
docker run --rm microtex-rs:ubuntu-24.04 '\[\iiint\limits_{V} \nabla \cdot \vec{F} \, dV = \iint\limits_{S} \vec{F} \cdot \vec{n} \, dS\]'

# Interactive shell for exploration
docker run --rm -it microtex-rs:ubuntu-24.04 /bin/bash

# Run specific tests
docker run --rm microtex-rs:ubuntu-24.04 cargo test --lib

# Generate documentation
docker run --rm microtex-rs:ubuntu-24.04 cargo doc --all-features --no-deps --open
```

## Troubleshooting

### Build takes too long

This is expected for the first build because:
- Cairo must be compiled (especially components that differ by platform)
- Pango and its dependencies are compiled

Subsequent builds use Docker cache and will be much faster.

### Memory limit issues

If you encounter "out of memory" errors, increase the memory allocated to Docker:
- Docker Desktop: Settings → Resources → Memory (recommended: 4 GB minimum)

### "Cannot find cairo headers" error

Ensure all system dependencies are installed in the Dockerfile. Update `/etc/apt/sources.list` if required.

## CI/CD integration

For GitHub Actions:

```yaml
name: Docker Test
on: [push, pull_request]

jobs:
  docker-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: docker/setup-buildx-action@v2
      - uses: docker/build-push-action@v4
        with:
          file: docker/Dockerfile.ubuntu-lts
          push: false
          load: true
          tags: microtex-rs:ubuntu-24.04
```

## Notes

- Ubuntu 24.04 LTS has long-term support (5 years)
- All dependencies are up to date as of 2024
- The Dockerfile uses layer ordering optimised for Docker caching
- The `--all-features` tests include vendored dependencies (Cairo, Pango)

## Licence

Same as the MicroTeX Rust project (MIT OR Apache-2.0)

````
