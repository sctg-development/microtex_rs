# MicroTeX Rust Bindings
![](https://tokeisrv.sctg.eu.org/b1/github/sctg-development/microtex_rs?type=Rust,C,CHeader,Cpp&category=code)
![](https://tokeisrv.sctg.eu.org/b1/github/sctg-development/microtex_rs?type=Rust,C,CHeader,Cpp&category=comments)
[![codecov](https://codecov.io/gh/sctg-development/microtex_rs/branch/main/graph/badge.svg)](https://codecov.io/gh/sctg-development/microtex_rs)

Safe Rust bindings for [MicroTeX](https://github.com/NanoMichael/MicroTeX), a lightweight LaTeX interpreter that can render mathematical formulas to SVG format.

## Features

- **Lightweight**: MicroTeX is a minimal LaTeX implementation focused on mathematical rendering
- **SVG Output**: Convert LaTeX formulas directly to scalable vector graphics
- **Embedded Fonts**: Includes multiple math fonts (XITS, FiraMath, TeX Gyre, Latin Modern)
- **Static Linking**: Optionally build and statically link all dependencies (Cairo, Pango, FontConfig)
- **Type-Safe**: Safe Rust wrapper around the C FFI interface
- **Complete Documentation**: Full rustdoc with executable examples

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies.microtex_rs]
git = "https://github.com/sctg-development/microtex_rs"
branch = "main"
```

### System Dependencies

By default, `microtex_rs` will attempt to use system-installed graphics libraries. The CI workflow installs the following packages per-platform; install the equivalent on your system:

macOS (Homebrew — ARM and Intel):

```bash
# macOS (arm64 or intel)
brew install cairo pango fontconfig pkg-config lzo libffi zlib bzip2 graphite2 libpng freetype harfbuzz pixman pcre2

# If you need to build amd64 (intel) binaries on an arm64 mac, use the x86_64 Homebrew and prefix commands with Rosetta:
# arch -x86_64 /usr/local/bin/brew install <pkg>
```

Ubuntu / Debian:

```bash
sudo apt-get update && sudo apt-get install -y libcairo2-dev libpango-1.0-0 libpango1.0-dev libfontconfig1-dev pkg-config
```

Notes for CI/coverage builds: the CI sometimes also installs `pkg-config`, `libssl-dev`, `clang` and `llvm` for certain tooling (coverage, tarpaulin).

Fedora:

```bash
sudo dnf install cairo-devel pango-devel fontconfig-devel pkg-config
```

Windows (vcpkg + MSVC):

```powershell
# Example using vcpkg (run from PowerShell / Admin if needed)
git clone https://github.com/Microsoft/vcpkg.git C:\vcpkg
C:\vcpkg\bootstrap-vcpkg.bat
C:\vcpkg\vcpkg.exe install cairo:x64-windows pango:x64-windows fontconfig:x64-windows pkgconf:x64-windows libpng:x64-windows freetype:x64-windows harfbuzz:x64-windows pixman:x64-windows libffi:x64-windows pcre2:x64-windows zlib:x64-windows bzip2:x64-windows lzo:x64-windows
```

Common build tools (required by the CI):

```bash
# meson and ninja (installed in CI via pip):
python3 -m pip install --break-system-packages meson ninja

# Rust toolchain (use rustup to install 'stable')
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

If you prefer to use vendored dependency bundles instead of system libraries, see the **Dependency Bundles** section below.

### Dependency Bundles

The `main` branch prefers system libraries or prebuilt dependency bundles. To create a macOS Intel bundle locally:

1. Install required packages with Homebrew:

```bash
brew install cairo pango fontconfig pkg-config lzo libffi zlib bzip2 graphite2 libpng freetype harfbuzz pixman pcre2
```

2. Run the helper script to collect the installed files into a bundle:

```bash
scripts/create_bundle_macos.sh
```

This will produce `dependencies_bundle/macos/intel` (or use `MICROTEX_BUNDLE_DIR` to point elsewhere). Commit the bundle or point the build to it via `MICROTEX_BUNDLE_DIR`.
## Quick Start

```rust
use microtex_rs::{MicroTex, RenderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new renderer with embedded fonts
    let renderer = MicroTex::new()?;
    
    // Use default rendering configuration
    let config = RenderConfig::default();
    
    // Render a LaTeX formula
    let latex = r#"\[E = mc^2\]"#;
    let svg = renderer.render(latex, &config)?;
    
    // Write SVG to file
    std::fs::write("formula.svg", svg)?;
    
    Ok(())
}
```

## Advanced Configuration

```rust
use microtex_rs::{MicroTex, RenderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let renderer = MicroTex::new()?;
    
    // Customize rendering parameters
    let config = RenderConfig {
        dpi: 1440,                    // Higher DPI for better quality
        line_width: 30.0,             // Wider lines
        line_height: 10.0,            // Custom line height
        text_color: 0xffffffff,       // White text
        render_glyph_use_path: true,  // Use path rendering for glyphs
        ..Default::default()
    };
    
    let latex = r#"
        \[
        \frac{\partial^2 u}{\partial t^2} = 
        c^2 \nabla^2 u
        \]
    "#;
    
    let svg = renderer.render(latex, &config)?;
    std::fs::write("wave_equation.svg", svg)?;
    
    Ok(())
}
```

## Examples

Run included examples:

```bash
# Simple formula
cargo run --example simple_formula

# Write SVG to file
cargo run --example render_to_file

# Batch rendering
cargo run --example batch_render
```

## Supported LaTeX

MicroTeX supports a substantial subset of LaTeX, including:

- Mathematical formulas and symbols
- Environments: `equation`, `align`, `displaymath`, `math`
- Commands: `\frac`, `\sqrt`, `\sum`, `\int`, `\limits`, and many more
- Greek letters and mathematical symbols
- Subscripts and superscripts
- Complex nested structures

## Architecture

The crate is structured as follows:

1. **FFI Layer** (`ffi` module): Raw C bindings generated by bindgen
2. **Safe API** (`MicroTex`, `RenderConfig`): Rust-safe wrappers
3. **Build System**: Static compilation of MicroTeX C++ library and optional dependency vendoring

### Build Process

The `build.rs` script:

1. Verifies or builds vendored dependencies (Cairo, Pango, FontConfig)
2. Builds the MicroTeX C++ library with CMake
3. Generates bindgen FFI bindings from C headers
4. Embeds all available CLM (math font) files at compile time

## Performance

- Compilation with system libraries: ~20 seconds
- Compilation with vendored dependencies: ~5-10 minutes (first time only)
- Rendering typical formulas: ~10-50ms

## Thread Safety

Each `MicroTex` instance initializes and releases its own MicroTeX context.
The renderer is **not** thread-safe; create separate instances per thread if needed.

## Error Handling

All rendering operations return `Result<String, RenderError>` with detailed error variants:

- `InitializationFailed`: Font initialization failed
- `ParseRenderFailed`: LaTeX parsing or rendering failed
- `EmptyOutput`: Rendering produced no output
- `InvalidUtf8`: Output encoding error

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

Most dependencies maintain their original licenses (Cairo: LGPL, Pango: LGPL, MicroTeX: Apache 2.0).
