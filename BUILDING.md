# Building MicroTeX Rust Bindings

This document explains how to build and develop the MicroTeX Rust bindings.

## Prerequisites

### System Requirements

- **Rust 1.70+** (via rustup)
- **C++ compiler** (g++, clang, or MSVC)
- **CMake 3.10+** (for building MicroTeX C++ library)
- **pkg-config** (for finding system libraries)

### macOS

```bash
brew install rust cmake pkg-config
```

### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install rustc cargo cmake pkg-config build-essential
```

### Fedora

```bash
sudo dnf install rust cargo cmake pkg-config gcc g++
```

## Build Modes

### 1. Default Build (System Libraries)

The default build uses system-installed graphics libraries:

```bash
cargo build
```

This requires:
- libcairo2-dev (or libcairo-devel)
- libpango-1.0-0 and libpango1.0-dev
- libfontconfig1-dev
- freetype, harfbuzz

**macOS**:
```bash
brew install cairo pango fontconfig pkg-config
cargo build
```

**Ubuntu/Debian**:
```bash
sudo apt-get install libcairo2-dev libpango-1.0-0 libpango1.0-dev libfontconfig1-dev pkg-config
cargo build
```

### 2. Vendored Build (Static Dependencies)

Build and statically link all graphics dependencies:

```bash
cargo build --features vendored-cairo
```

This automatically downloads and compiles:
- Cairo 1.18.4
- Pixman
- FreeType
- HarfBuzz
- FontConfig
- Pango (optional, with `vendored-pango` feature)

**Note**: First-time vendored builds take 5-10 minutes. Subsequent builds are faster due to caching.

### 3. Release Build

For production use:

```bash
cargo build --release
```

Add `--features vendored-cairo` for a fully static build.

## Development

### Setting Up Development Environment

```bash
# Clone and navigate to project
cd microtex_rs

# Run tests
cargo test --lib
cargo test --test integration_tests

# Build examples
cargo build --examples

# Run specific example
cargo run --example simple_formula
cargo run --example render_to_file
cargo run --example batch_render
```

### Building Documentation

```bash
# Generate and open documentation
cargo doc --open
```

### Code Quality

```bash
# Run clippy linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check for unused code
cargo check
```

## Build System Details

The build process is controlled by `build.rs`:

1. **Dependency Detection**: Checks if system graphics libraries are available via pkg-config
2. **Vendored Build** (if enabled): Downloads and builds Cairo and optionally Pango
3. **CMake Configuration**: Builds the MicroTeX C++ library with:
   - `HAVE_CWRAPPER=ON` (enables C FFI wrapper)
   - `BUILD_STATIC=ON` (static library output)
   - `CAIRO=ON` (enables Cairo support for SVG rendering)
   - `Profile=Release` (optimized build)
4. **Bindgen**: Generates Rust FFI bindings from C++ headers
5. **Font Embedding**: Scans and embeds all CLM (math font) files at compile time

### Environment Variables

Control build behavior with:

```bash
# Force rebuild of vendored dependencies
export MICROTEX_VENDORED_CAIRO_FORCE_REBUILD=1 cargo build --features vendored-cairo

# Prefer system libraries even if vendored feature is enabled
export MICROTEX_USE_SYSTEM_CAIRO=1 cargo build

# Verify downloaded tarball hash
export MICROTEX_CAIRO_SHA256=abc123... cargo build --features vendored-cairo

# Skip Pango vendoring
export MICROTEX_VENDORED_PANGO=0 cargo build --features vendored-cairo
```

## Building Specific Binaries

### Build the CLI Tool

```bash
cargo build --bin microtex --release
./target/release/microtex 'E = mc^2' --output formula.svg
```

### Build Examples

```bash
# Simple formula rendering
cargo run --example simple_formula

# Render multiple formulas to files
cargo run --example render_to_file

# Batch rendering with different configurations
cargo run --example batch_render
```

## Testing

### Unit and Integration Tests

```bash
# All tests
cargo test

# Only library tests
cargo test --lib

# Only integration tests
cargo test --test integration_tests

# Specific test
cargo test test_embedded_fonts_available

# With output
cargo test -- --nocapture
```

### C++ Test Suite

The C++ library includes its own tests:

```bash
cd c++/mini_tests
./test_math_svg
```

Expected output includes rendering of a complex divergence theorem formula.

## Troubleshooting

### "pkg-config not found"

**Solution**: Install pkg-config
- macOS: `brew install pkg-config`
- Linux: `sudo apt-get install pkg-config` (Ubuntu) or `sudo dnf install pkg-config` (Fedora)

### "cairo not found" when not using vendored

**Solution**: Install Cairo development files
- macOS: `brew install cairo pango fontconfig`
- Ubuntu: `sudo apt-get install libcairo2-dev libpango-1.0-0 libpango1.0-dev libfontconfig1-dev`

### Build errors with C++ compiler

**Solution**: Ensure C++ compiler is installed and in PATH
- macOS: Xcode Command Line Tools: `xcode-select --install`
- Ubuntu: `sudo apt-get install build-essential`
- Fedora: `sudo dnf install gcc gcc-c++`

### "meson" or "ninja" not found (for vendored builds)

**Solution**: The build script will attempt to install these automatically, or:
```bash
pip install meson ninja
```

Or on macOS:
```bash
brew install meson ninja
```

### CMake errors

**Solution**: Ensure CMake is installed:
```bash
# macOS
brew install cmake

# Ubuntu
sudo apt-get install cmake

# Fedora
sudo dnf install cmake
```

And is version 3.10 or higher:
```bash
cmake --version
```

## Platform-Specific Notes

### macOS

The build script automatically:
- Uses `libc++` (Clang's C++ library)
- Handles Frameworks for system libraries
- Respects the `SDKROOT` environment variable if set

### Linux

The build script:
- Uses `libstdc++` (GCC's C++ library)
- Properly handles library search paths for system libraries

### Windows

Currently not officially tested. Users interested in Windows support should:
1. Install MinGW or MSVC toolchain
2. Install CMake
3. Install required libraries or use vendored features
4. Report any issues

## Continuous Integration

The project includes configuration for running tests in CI:

```bash
# GitHub Actions compatible commands
cargo test --lib
cargo test --test integration_tests
cargo build --release
cargo build --release --features vendored-cairo
```

## Performance Notes

- **Debug build**: ~1-2 seconds
- **Release build**: ~20-30 seconds
- **Vendored build (first time)**: 5-10 minutes
- **Vendored build (cached)**: ~30 seconds
- **Test suite**: ~0.5 seconds
- **Formula rendering**: 10-50ms depending on complexity

## See Also

- [README.md](README.md) - Project overview
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [c++/README.md](c++/README.md) - MicroTeX C++ library documentation
