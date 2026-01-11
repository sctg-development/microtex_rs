# DPI Metadata Enhancement for MicroTeX SVG Output

## Overview

This document describes the enhancement made to `microtex_rs` to embed DPI (dots per inch) metadata in generated SVG output. This metadata is essential for downstream processors (like PDF generators) to correctly size and position SVG content.

## Problem Statement

When MicroTeX renders LaTeX formulas to SVG, the generated SVG contains width and height attributes in pixels, but there was no indication of the DPI used during rendering. Downstream processors (like `genpdfi_extended`) had to make assumptions about the DPI, leading to incorrect sizing when the actual DPI differed from the assumed value.

For example:
- MicroTeX renders at 720 DPI (default)
- SVG generated with `width="120px"` (a 12pt formula)
- A PDF generator assuming 300 DPI would incorrectly calculate the formula's physical size
- Result: Formula appears ~2.4x larger than intended

## Solution

### New Public Function: `add_dpi_to_svg()`

A public function has been added to embed DPI metadata into SVG elements:

```rust
/// Adds DPI metadata to an SVG string as a `data-dpi` attribute.
pub fn add_dpi_to_svg(svg: &str, dpi: i32) -> String
```

**Behavior:**
- Locates the `<svg>` opening tag in the SVG content
- Injects a `data-dpi="XXX"` attribute before the closing `>`
- Returns the modified SVG with metadata embedded
- Returns original SVG unchanged if no `<svg>` tag is found

**Example:**

Input:
```xml
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50">
```

Output (with dpi=720):
```xml
<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" data-dpi="720">
```

### Modified Methods

The following public methods now automatically embed DPI metadata in their output:

#### 1. `MicroTex::render()`
- **Location:** `src/lib.rs:936-981`
- **Change:** SVG output is processed through `add_dpi_to_svg()` before being returned
- **Impact:** Every rendered SVG now contains the DPI value used during rendering

#### 2. `MicroTex::render_to_svg_with_metrics()`
- **Location:** `src/lib.rs:1038-1122`
- **Change:** SVG extracted from JSON response is processed through `add_dpi_to_svg()` before being included in `RenderResult`
- **Impact:** Both SVG rendering methods now consistently embed DPI metadata

## Implementation Details

### Function: `add_dpi_to_svg()`
- **File:** `src/lib.rs:813-852`
- **Type:** Public function
- **Algorithm:**
  1. Search for `<svg` opening tag in the SVG string
  2. If found, locate the closing `>` character
  3. Insert ` data-dpi="XXX"` before the `>`
  4. Return the modified string
  5. If no `<svg>` tag found or malformed, return original string unchanged

**Key Features:**
- Safe: Returns original string on any parsing error (no exceptions)
- Efficient: Single-pass string manipulation
- Robust: Handles XML declarations, namespaces, and attributes
- Preserves: All existing SVG content and structure

## Testing

### Unit Tests Added
Six comprehensive tests verify the `add_dpi_to_svg()` function:

```rust
test tests::test_add_dpi_to_svg_simple ... ok
test tests::test_add_dpi_to_svg_with_namespace ... ok
test tests::test_add_dpi_to_svg_different_dpi_values ... ok
test tests::test_add_dpi_to_svg_no_svg_tag ... ok
test tests::test_add_dpi_to_svg_malformed ... ok
test tests::test_add_dpi_to_svg_preserves_content ... ok
```

**Test Coverage:**
1. **Simple SVG**: Verifies basic attribute injection with standard namespace
2. **Namespace handling**: Tests with full SVG namespace declaration
3. **Multiple DPI values**: Confirms correct values (300, 720) are embedded
4. **No SVG tag**: Ensures graceful fallback when no `<svg>` tag exists
5. **Malformed SVG**: Validates error handling for incomplete tags
6. **Content preservation**: Confirms that SVG body content remains unchanged

### Test Results
All tests pass:
- **Unit tests:** 26 passed
- **Binary tests:** 3 passed
- **Integration tests:** 7 passed
- **Multiple renderer tests:** 1 passed
- **Example validation:** 7 passed
- **Total:** 44 tests passed, 0 failed

### Example Verification
The `simple_formula` example was updated to verify DPI embedding in real-world scenarios. Output confirms:
```
✓ data-dpi attribute found! (for Einstein's E=mc²)
✓ data-dpi attribute found! (for Pythagorean theorem)
✓ data-dpi attribute found! (for Quadratic formula)
✓ data-dpi attribute found! (for Sum notation)
✓ data-dpi attribute found! (for Integration)
```

## Impact on Downstream Processors

### For `genpdfi_extended`
The PDF generator can now:
1. Read the `data-dpi` attribute from SVG
2. Use the actual rendering DPI (not assumed values) for sizing calculations
3. Correctly position SVG content in PDF pages
4. Fix the issue where large formulas were incorrectly placed and cut off

### Example Usage in `genpdfi_extended`

```rust
// In src/elements/images.rs (ImageSource::intrinsic_size)
ImageSource::Svg(svg) => {
    let mmpi: f32 = 25.4;
    
    // Extract actual DPI from data-dpi attribute
    let actual_dpi = svg.dpi.unwrap_or_else(|| {
        // Fallback: try to extract from SVG string's data-dpi attribute
        extract_data_dpi_attribute(&svg_content)
            .unwrap_or(300.0)
    });
    
    let width_px = svg.width.map(|px| px.0 as f32).unwrap_or(100.0);
    let height_px = svg.height.map(|px| px.0 as f32).unwrap_or(100.0);
    
    Size::new(
        mmpi * (width_px / actual_dpi),
        mmpi * (height_px / actual_dpi),
    )
}
```

## Backward Compatibility

✅ **Fully backward compatible**

- The change only adds metadata; existing SVG functionality is not affected
- SVG files without the `data-dpi` attribute work as before
- Downstream processors can ignore the attribute if they don't need it
- No breaking changes to the public API
- All existing tests pass without modification

## Benefits

1. **Correct Sizing**: Downstream processors can now accurately determine SVG dimensions
2. **Self-Documenting**: SVG files contain metadata about their rendering context
3. **Robustness**: Eliminates guessing about DPI values
4. **Standardization**: Establishes a convention for encoding DPI in SVG
5. **Debugging**: Easier to troubleshoot sizing issues with explicit DPI information

## Files Modified

- **`src/lib.rs`**
  - Added public function `add_dpi_to_svg()` (lines 813-852)
  - Modified `MicroTex::render()` to use `add_dpi_to_svg()` (line 945)
  - Modified `MicroTex::render_to_svg_with_metrics()` to use `add_dpi_to_svg()` (line 1078)
  - Added 6 unit tests for `add_dpi_to_svg()` (lines 1513-1563)

- **`examples/simple_formula.rs`**
  - Updated to verify `data-dpi` attribute in output (lines 39-42)

## Testing Commands

```bash
# Run all tests in microtex_rs
cargo test

# Run only DPI-related tests
cargo test --lib add_dpi

# Run the simple formula example with verification
cargo run --example simple_formula
```

## Related Issues

This change addresses the core issue in `genpdfi_extended` where SVG formulas were incorrectly sized due to DPI assumptions. See: `genpdfi_extended/ANALYSIS_PDF_LAYOUT_DEFECT.md`

## Version

- **microtex_rs:** v0.1 (enhanced)
- **Date:** January 11, 2026
- **Status:** ✅ Complete and tested
