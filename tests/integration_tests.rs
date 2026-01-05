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
    assert!(!data.unwrap().is_empty(), "XITS font data should not be empty");
}

#[test]
fn test_firamath_font_accessible() {
    // Verify FiraMath font is embedded
    let data = get_embedded_clm("FiraMath-Regular.clm2");
    assert!(data.is_some(), "FiraMath-Regular.clm2 should be embedded");
    assert!(!data.unwrap().is_empty(), "FiraMath font data should not be empty");
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
