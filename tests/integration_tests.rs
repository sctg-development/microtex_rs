/// Integration tests for MicroTeX rendering.
///
/// These tests demonstrate basic functionality. Due to C++ exception handling limitations,
/// rendering tests are best verified through the examples or C++ test suite.
use microtex_rs::*;

#[test]
fn test_embedded_fonts_available() {
    let fonts = available_embedded_clms();
    assert!(!fonts.is_empty(), "No embedded CLM fonts found");
}

#[test]
fn test_xits_font_accessible() {
    // Verify XITS font is embedded and accessible
    let data = get_embedded_clm("XITS-Regular.clm2");
    assert!(data.is_some(), "XITS-Regular.clm2 should be embedded");
    assert!(
        !data.unwrap().is_empty(),
        "XITS font data should not be empty"
    );
}

#[test]
fn test_firamath_font_accessible() {
    // Verify FiraMath font is embedded
    let data = get_embedded_clm("FiraMath-Regular.clm2");
    assert!(data.is_some(), "FiraMath-Regular.clm2 should be embedded");
    assert!(
        !data.unwrap().is_empty(),
        "FiraMath font data should not be empty"
    );
}

#[test]
fn test_render_config_defaults() {
    let config = RenderConfig::default();
    assert_eq!(config.dpi, 720);
    assert_eq!(config.line_width, 20.0);
    assert!(config.render_glyph_use_path);
    assert!(!config.has_background);
}

#[test]
fn test_render_config_customization() {
    let mut config = RenderConfig::default();
    config.dpi = 300;
    config.line_width = 15.0;
    config.text_color = 0xffffffff;

    assert_eq!(config.dpi, 300);
    assert_eq!(config.line_width, 15.0);
    assert_eq!(config.text_color, 0xffffffff);
}

#[test]
fn test_render_error_display() {
    let error = RenderError::InitializationFailed;
    assert!(format!("{}", error).contains("font metadata"));

    let error = RenderError::ParseRenderFailed;
    assert!(format!("{}", error).contains("parse and render"));

    let error = RenderError::EmptyOutput;
    assert!(format!("{}", error).contains("empty"));
}

/// This test reproduces the SIGSEGV crash reported when calling render() multiple times.
/// It attempts to render multiple LaTeX formulas using the same MicroTex instance.
/// The crash manifests with "microtex_render_to_svg: after finish, vec.size=5965" message.
///
/// See: https://github.com/issue-tracker/microtex_rs/issues/XXX
#[test]
// Ignore by default - only run manually to test C++ behavior
fn test_multiple_renders_real_c_plus_plus() {
    // Create a renderer instance
    let renderer = match MicroTex::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize MicroTex: {}", e);
            panic!("Cannot run test: {}", e);
        }
    };

    let config = RenderConfig::default();

    // First render
    println!("First render...");
    let result1 = renderer.render(r#"\[x^2 + y^2 = z^2\]"#, &config);
    match &result1 {
        Ok(svg) => println!("First render succeeded, SVG size: {} bytes", svg.len()),
        Err(e) => println!("First render failed: {}", e),
    }

    // Second render on the SAME instance - this may trigger SIGSEGV
    println!("Second render on same instance...");
    let result2 = renderer.render(r#"\[E = mc^2\]"#, &config);
    match &result2 {
        Ok(svg) => println!("Second render succeeded, SVG size: {} bytes", svg.len()),
        Err(e) => println!("Second render failed: {}", e),
    }

    // Third render - verify behavior persists
    println!("Third render on same instance...");
    let result3 = renderer.render(r#"\[\int_0^\infty e^{-x} dx = 1\]"#, &config);
    match &result3 {
        Ok(svg) => println!("Third render succeeded, SVG size: {} bytes", svg.len()),
        Err(e) => println!("Third render failed: {}", e),
    }

    // At least the first should succeed if no crash
    assert!(result1.is_ok(), "First render should succeed");
}
