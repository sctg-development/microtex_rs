# Embedded Fonts

This document describes the math fonts embedded in microtex_rs.

## Available Fonts

### 1. XITS Math (Primary)

**Files**: `XITS-Regular.clm2`, `XITS-Bold.clm2`, `XITS-BoldItalic.clm2`, `XITS-Italic.clm2`, `XITSMath-Regular.clm2`, `XITSMath-Bold.clm2`

**Description**: XITS is a Times-like font for typesetting mathematics. It's based on STIX and is well-suited for scientific and mathematical documents.

**License**: Open Font License (OFL)

**Status**: ✓ Primary choice for rendering

### 2. FiraMath

**File**: `FiraMath-Regular.clm2`

**Description**: A sans-serif math font companion to the Fira font family. Good for modern, clean mathematical typesetting.

**License**: Open Font License (OFL)

**Characteristics**:
- Clean, modern appearance
- Good for presentations
- Sans-serif style

### 3. TeX Gyre DejaVu Math

**File**: `texgyredejavu-math.clm2`

**Description**: Part of the TeX Gyre font family, based on DejaVu. Provides extensive mathematical symbol support.

**License**: Open Font License (OFL)

**Characteristics**:
- Very comprehensive symbol coverage
- Professional appearance
- Good compatibility with standard LaTeX fonts

### 4. Latin Modern Math

**File**: `latinmodern-math.clm2`

**Description**: Latin Modern extended with math support. Matches the default TeX font family.

**License**: GUST Font License (GFL)

**Characteristics**:
- Traditional TeX appearance
- Maximum compatibility with TeX documents
- Conservative, proven design

## Font Selection

The Rust binding automatically selects fonts in this priority order:

1. XITS-Regular.clm2
2. XITSMath-Regular.clm2
3. FiraMath-Regular.clm2
4. latinmodern-math.clm2
5. texgyredejavu-math.clm2

This ensures a suitable font is always available.

## CLM Format

CLM files are compiled font metrics files used by MicroTeX. They contain:

- Font metric information
- Glyph outlines
- Kerning and ligature data
- Mathematical layout parameters

CLM files are binary and cannot be directly edited. They are generated from OpenType fonts (OTF) using the prebuilt conversion tools.

## Adding Custom Fonts

Currently, the binding uses only the embedded fonts. To add support for custom fonts:

1. Locate or create an OpenType math font (`.otf`)
2. Use the conversion tool: `c++/prebuilt/otf2clm.py` or `otf2clm.sh`
3. Place the resulting `.clm2` file in `c++/res/<fontname>/`
4. Rebuild the Rust crate

## Font Characteristics Summary

| Font | Style | Coverage | Traditional | Modern |
|------|-------|----------|-------------|--------|
| XITS | Serif | Excellent | Good | Good |
| FiraMath | Sans-serif | Good | Fair | Excellent |
| TeX Gyre DejaVu | Serif | Excellent | Excellent | Good |
| Latin Modern | Serif | Good | Excellent | Fair |

## Rendering Quality

The rendering quality depends on:

1. **DPI Setting**: Higher DPI (e.g., 1440) produces better quality but larger SVG files
2. **Font Choice**: Different fonts may render symbols differently
3. **Glyph Rendering Mode**: Path-based rendering provides better outline quality

### Example Configurations

**Screen Display (96 DPI)**:
```rust
RenderConfig {
    dpi: 96,
    line_width: 12.0,
    line_height: 4.0,
    render_glyph_use_path: false,
    ..Default::default()
}
```

**Print Quality (300 DPI)**:
```rust
RenderConfig {
    dpi: 300,
    line_width: 20.0,
    line_height: 6.67,
    render_glyph_use_path: true,
    ..Default::default()
}
```

**High Resolution (1440 DPI)**:
```rust
RenderConfig {
    dpi: 1440,
    line_width: 40.0,
    line_height: 13.33,
    render_glyph_use_path: true,
    ..Default::default()
}
```

## Font Size and Scaling

The default configuration (20.0 pixels line width at 720 DPI) produces formulas suitable for:

- Mathematical typesetting in documents
- Web display
- Scientific papers

Adjust the DPI and line width parameters for different use cases.

## Character Coverage

All embedded fonts support:

- **Latin letters and digits**
- **Greek letters** (α, β, γ, etc.)
- **Mathematical operators** (∑, ∫, √, ±, etc.)
- **Relational symbols** (=, <, >, ≤, ≥, etc.)
- **Large operators** (∫, ∮, ∯, etc.)
- **Accents and decorations** (^, ~, ̇, etc.)
- **Brackets and delimiters** (scalable parentheses, braces, etc.)

## Performance

Font loading and initialization:
- Initial: ~10-50ms (first creation)
- Subsequent: Minimal (shared resources)

Rendering with different fonts has negligible performance difference.

## License Summary

All embedded fonts are under permissive open-source licenses:

- **XITS**: OFL (Open Font License)
- **FiraMath**: OFL
- **TeX Gyre DejaVu**: OFL
- **Latin Modern**: GFL (GUST Font License)

These licenses permit:
- ✓ Embedding in documents
- ✓ Modification
- ✓ Commercial use
- ✓ Distribution

See individual font directories in `c++/res/` for detailed license files.
