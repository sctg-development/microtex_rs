use std::result;

use microtex_rs::{MicroTex, RenderConfig};
pub fn latex_to_svg(latex_content: &str, display: bool) -> Result<String, String> {
    // Trim whitespace
    let latex = latex_content.trim();

    // Validate that content is not empty
    if latex.is_empty() {
        return Err("LaTeX content is empty".to_string());
    }

    // Wrap LaTeX in appropriate delimiters
    // MicroTeX expects display math in \[...\] format and inline in $...$ format
    let latex_with_delimiters = if display {
        format!("\\[{}\\]", latex)
    } else {
        // correct inline delimiters: $...$
        format!("${}$", latex)
    };

    // Create a MicroTeX instance
    let renderer = MicroTex::new().map_err(|e| format!("Failed to initialize MicroTeX: {}", e))?;

    // Create render configuration with defaults
    let config = RenderConfig::default();

    // Render LaTeX to SVG
    renderer
        .render(&latex_with_delimiters, &config)
        .map_err(|e| format!("Failed to render LaTeX: {}", e))
}
#[test]
fn test_multiple_renders() {
    let result = latex_to_svg("x^2 + y^2 = z^2", false);
    assert!(result.is_ok());
    let result = latex_to_svg("\\int_0^\\infty e^{-x} dx", true);
    assert!(result.is_ok());
    let result = latex_to_svg("", true);
    assert!(result.is_err());
    let result = latex_to_svg("E = mc^2", false);
    assert!(result.is_ok());
    let result = latex_to_svg("a^2 + b^2 = c^2", true);
    assert!(result.is_ok());
    let result = latex_to_svg("x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}", true);
    assert!(result.is_ok());
    let result = latex_to_svg("\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}", true);
    assert!(result.is_ok());
    let result = latex_to_svg(
        "\\int_0^\\infty e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}",
        true,
    );
    assert!(result.is_ok());
}
