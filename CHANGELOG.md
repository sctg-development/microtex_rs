# Changelog

## Version 0.1.0 (Initial Release)

### Features

- **Safe Rust Bindings**: Complete wrapper around MicroTeX C FFI interface
- **LaTeX to SVG Conversion**: Convert LaTeX formulas to scalable vector graphics
- **Embedded Fonts**: Includes multiple math fonts:
  - XITS Math (primary, well-tested)
  - FiraMath
  - TeX Gyre DejaVu Math
  - Latin Modern Math
- **Static Compilation**: Build system automatically handles:
  - Static linking of MicroTeX C++ library
  - Optional vendoring of Cairo and dependencies
  - Font embedding at compile time
- **Configurable Rendering**: 
  - Adjustable DPI, line width, and line height
  - Custom text colors
  - Path-based glyph rendering
- **Complete Documentation**: Rustdoc with working examples
- **CLI Tool**: Command-line interface for formula rendering
- **Comprehensive Tests**: Integration tests and embedded font verification

### Known Limitations

- **C++ Exception Handling**: MicroTeX may throw C++ exceptions during parsing that Rust cannot catch. This is a known limitation of C/C++ interoperability. For rendering verification, use:
  - The C++ test suite: `c++/mini_tests/test_math_svg`
  - The provided examples
  - The CLI tool

### Architecture

```
microtex_rs (Rust crate)
├── src/lib.rs          (Safe API wrapper)
├── src/bin/microtex.rs (CLI tool)
├── examples/           (Usage examples)
├── tests/              (Integration tests)
└── build.rs            (Build system)
    └── FFI bindings from c++/lib/wrapper/cwrapper.h
        └── MicroTeX C++ library (c++/lib/)
            └── Cairo/Pango/FontConfig dependencies
```

### Usage Examples

```rust
// Simple rendering
use microtex_rs::{MicroTex, RenderConfig};

let renderer = MicroTex::new()?;
let svg = renderer.render(r#"\[E = mc^2\]"#, &RenderConfig::default())?;
```

```bash
# CLI usage
cargo run --bin microtex -- 'E = mc^2' --output formula.svg
```

### Building

```bash
# With system libraries
cargo build

# With vendored static dependencies
cargo build --features vendored-cairo
```

### Testing

```bash
# Unit and integration tests
cargo test

# Run examples
cargo run --example simple_formula
cargo run --example render_to_file

# C++ test suite
cd c++/mini_tests && ./test_math_svg
```

### Dependencies

**Runtime (embedded)**:
- MicroTeX C++ library
- Multiple embedded math fonts (CLM format)

**System libraries** (or vendored):
- Cairo (for SVG rendering)
- Pango (for text layout)
- FontConfig (for font management)
- FreeType, HarfBuzz (for glyph handling)

**Rust crates**:
- `clap` (CLI argument parsing)
- `thiserror` (error handling)
- `log` (logging)

### Contributing

The project follows Rust best practices:
- Full rustdoc documentation
- Comprehensive error types
- No unwrap() in public API
- Tests for all public functionality

### Future Enhancements

Possible improvements for future versions:
- Batch rendering API
- Custom font support
- Caching for repeated formulas
- Async rendering
- Additional output formats (PNG, PDF)
- C++ exception handling wrapper
- Thread-safe rendering with pooling
