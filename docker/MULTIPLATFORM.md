# MicroTeX Rust - Multi-Platform Build Guide
````markdown
# MicroTeX Rust — Multi-Platform Build Guide

## Supported platforms

| Platform | Architecture | Container | Status |
|----------|--------------|-----------|--------|
| **macOS** | Intel/ARM64 (M-series) | Native | ✅ Tested |
| **Ubuntu** | x86_64, ARM64 | ubuntu-lts | ✅ Validated |
| **Alpine** | x86_64, ARM64 | alpine-3.23 | ✅ Validated |

## Quick start

### 1. Build for all platforms (multi-arch)

```bash
# Requires: docker buildx
./docker/buildx-test.sh
```

This will build:
- `microtex-rs:ubuntu-24.04` for linux/amd64 and linux/arm64
- `microtex-rs:alpine-3.23` for linux/amd64 and linux/arm64

### 2. Build specific platforms

**macOS (local):**
```bash
cargo build --all-features
cargo test --all-features
```

**Ubuntu 24.04 (Docker):**
```bash
docker build -f docker/Dockerfile.ubuntu-lts -t microtex-rs:ubuntu .
docker run --rm microtex-rs:ubuntu --help
```

**Alpine 3.23 (Docker):**
```bash
docker build -f docker/Dockerfile.alpine -t microtex-rs:alpine .
docker run --rm microtex-rs:alpine --help
```

## Platform-specific notes

### macOS
- **Backend:** Quartz (native CoreText)
- **Frameworks:** CoreFoundation, CoreGraphics, CoreText
- **Deployment target:** 11.0 (Intel/ARM64 compatible)
- **Build time:** ~1–2 minutes (with cache)

### Ubuntu 24.04
- **Libc:** glibc
- **Backend:** No X11/XCB (headless rendering)
- **Libraries:** Vendored Cairo/Pango + system fontconfig, freetype
- **Build time:** ~5–15 minutes (first build), ~1–2 minutes (cached)

### Alpine 3.23
- **Libc:** musl
- **Backend:** No X11/XCB (headless rendering)
- **Libraries:** Vendored Cairo/Pango + system fontconfig, freetype
- **Spectre mitigations:** Disabled (for musl compatibility)
- **Build time:** ~5–15 minutes (first build), ~1–2 minutes (cached)

## Dependencies

### Build-time
```
build-essential/gcc/clang
cmake
clang/libclang-dev
meson
ninja
python3
pkg-config
```

### Graphics stack
```
libcairo (vendored on Linux)
libpango (vendored on Linux)
libfontconfig
libfreetype
libharfbuzz
libpixman
libpng
zlib
```

### Runtime
On macOS: all statically linked or framework-bundled.
On Linux: system libraries (glibc/musl compatible).

## Multi-architecture building

### Using Docker Buildx

Build simultaneously for multiple architectures:

```bash
# Build for both amd64 and arm64
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -f docker/Dockerfile.ubuntu-lts \
  -t myregistry/microtex-rs:ubuntu-24.04 \
  .
```

### For Raspberry Pi (ARM64)

```bash
docker buildx build \
  --platform linux/arm64 \
  -f docker/Dockerfile.ubuntu-lts \
  -t microtex-rs:rpi .
```

## Troubleshooting

### Cairo build fails on Linux

**Issue:** `error: could not find native static library 'cairo'`

**Solution:** Cairo libraries may be located in a `lib/x86_64-linux-gnu/` subdirectory. The `build.rs` script attempts to find them automatically, but if it fails:

```bash
find /path/to/cairo-install -name "libcairo.a"
```

### musl compatibility issues (Alpine)

**Issue:** Linker errors related to musl-specific features

**Solution:** The `build.rs` disables Spectre mitigations for musl. If you encounter other issues, please open an issue and attach the full error log.

### Docker build cache issues

```bash
# Force a clean rebuild
docker buildx build --no-cache -f docker/Dockerfile.ubuntu-lts .
```

## Performance notes

| Config | Time (cached) | Time (clean) | Size |
|--------|---------------|--------------|------|
| macOS build | <1 min | 2 min | 20 MB |
| Ubuntu Docker | 1–2 min | 10–15 min | 25 MB |
| Alpine Docker | 1–2 min | 10–15 min | 18 MB |
| Multi-arch (2) | 2–4 min | 20–30 min | N/A |

**Tips:**
- Use `--cache-to` and `--cache-from` with buildx for faster rebuilds
- Build the release version for distribution: `cargo build --release`
- Strip binaries for smaller size: `strip target/release/microtex`

## CI/CD integration

### GitHub Actions example

```yaml
name: Multi-Platform Build

on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        dockerfile: [
          'docker/Dockerfile.ubuntu-lts',
          'docker/Dockerfile.alpine'
        ]
        platform: [
          'linux/amd64',
          'linux/arm64'
        ]
    
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/build-push-action@v5
        with:
          file: ${{ matrix.dockerfile }}
          platforms: ${{ matrix.platform }}
          push: false
```

## Testing the build

After building, test with:

```bash
# Help text
docker run --rm microtex-rs:ubuntu-24.04 --help

# Test rendering
docker run --rm microtex-rs:ubuntu-24.04 "E = mc^2"

# Interactive shell
docker run --rm -it --entrypoint /bin/bash microtex-rs:ubuntu-24.04
```

## Development workflow

### Local development (macOS)

```bash
cargo build
cargo test
cargo doc --open
```

### Test Docker changes

```bash
# Build image
docker build -f docker/Dockerfile.ubuntu-lts -t test:latest .

# Run tests in container
docker run --rm test:latest cargo test --all-features

# Interactive debugging
docker run --rm -it --entrypoint /bin/bash test:latest
```

### Release build

```bash
# Optimise for distribution
cargo build --all-features --release

# Strip symbols (reduces size ~50%)
strip target/release/microtex

# Create an archive
tar czf microtex-rs-$(git describe --tags).tar.gz target/release/microtex
```

---

**For more information:**
- [Dockerfile.ubuntu-lts](Dockerfile.ubuntu-lts) — Production Ubuntu image
- [Dockerfile.alpine](Dockerfile.alpine) — Minimal Alpine image
- [buildx-test.sh](buildx-test.sh) — Multi-platform test script

````
