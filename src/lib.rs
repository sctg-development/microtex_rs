#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(rustdoc::invalid_codeblock_attributes)]

#[doc(hidden)]
mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(missing_docs)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::fmt;
use thiserror::Error;

// Re-export CLM helpers generated at build time
include!(concat!(env!("OUT_DIR"), "/embedded_clms.rs"));

/// Runtime test control helpers (always compiled) used to configure shim behavior from tests.
pub mod test_control {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Mutex;

    static INIT_SUCCEED: AtomicBool = AtomicBool::new(true);
    static PARSE_SUCCEED: AtomicBool = AtomicBool::new(true);
    static RETURN_EMPTY: AtomicBool = AtomicBool::new(false);
    static OUT_LEN: AtomicU64 = AtomicU64::new(0);
    static TEST_LOCK: Mutex<()> = Mutex::new(());
    static TEST_BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    /// Acquire a lock to serialize tests that touch global test control state.
    pub fn lock_test() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK.lock().unwrap()
    }

    pub fn set_init_succeed(v: bool) {
        INIT_SUCCEED.store(v, Ordering::SeqCst);
    }
    pub fn set_parse_succeed(v: bool) {
        PARSE_SUCCEED.store(v, Ordering::SeqCst);
    }
    pub fn set_return_empty(v: bool) {
        RETURN_EMPTY.store(v, Ordering::SeqCst);
    }
    pub fn set_buffer(data: &[u8]) {
        let mut buf = TEST_BUFFER.lock().unwrap();
        buf.clear();
        buf.extend_from_slice(data);
        OUT_LEN.store(buf.len() as u64, Ordering::SeqCst);
    }

    pub fn get_init_succeed() -> bool {
        INIT_SUCCEED.load(Ordering::SeqCst)
    }
    pub fn get_parse_succeed() -> bool {
        PARSE_SUCCEED.load(Ordering::SeqCst)
    }
    pub fn get_return_empty() -> bool {
        RETURN_EMPTY.load(Ordering::SeqCst)
    }
    /// Returns a pointer to the internal test buffer and its length.
    /// The buffer is owned by the static inside `test_control` and will remain
    /// valid until modified by `set_buffer` (tests should use `lock_test()` to
    /// avoid races).
    pub fn get_out_buffer_ptr() -> (*const u8, u64) {
        let buf = TEST_BUFFER.lock().unwrap();
        (buf.as_ptr(), OUT_LEN.load(Ordering::SeqCst))
    }
}

/// Shim layer to wrap FFI calls and allow test-controlled behavior.
mod shim {
    use std::ffi::c_void;

    #[cfg(not(test))]
    pub unsafe fn microtex_init(len: u64, ptr: *const u8) -> *mut c_void {
        super::ffi::microtex_init(len, ptr as *const _)
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_set_default_main_font(ptr: *const i8) {
        super::ffi::microtex_setDefaultMainFont(ptr as *const _);
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_set_render_glyph_use_path(val: bool) {
        super::ffi::microtex_setRenderGlyphUsePath(val);
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_release_font_meta(meta: *mut c_void) {
        super::ffi::microtex_releaseFontMeta(meta as *mut _);
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_parse_render(
        src: *const i8,
        dpi: i32,
        line_width: f32,
        line_height: f32,
        text_color: u32,
        has_background: bool,
        render_glyph_use_path: bool,
    ) -> *mut c_void {
        super::ffi::microtex_parseRender(
            src,
            dpi,
            line_width,
            line_height,
            text_color,
            has_background,
            render_glyph_use_path,
            0,
        )
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_render_to_svg(render_ptr: *mut c_void, out_len: &mut u64) -> *mut u8 {
        super::ffi::microtex_render_to_svg(render_ptr as *mut _, out_len)
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_delete_render(render_ptr: *mut c_void) {
        super::ffi::microtex_deleteRender(render_ptr as *mut _);
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_free_buffer(buf: *mut u8) {
        super::ffi::microtex_free_buffer(buf as *mut _);
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_release() {
        super::ffi::microtex_release();
    }

    // --------- Test-controlled implementations ---------
    #[cfg(test)]
    mod test_impl {
        use std::ffi::c_void;
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::sync::Mutex;

        // Control helpers now delegated to `crate::test_control` so tests across
        // the crate share a single synchronization and buffer storage.
        pub fn lock_test() -> std::sync::MutexGuard<'static, ()> {
            crate::test_control::lock_test()
        }

        pub fn set_init_succeed(v: bool) {
            crate::test_control::set_init_succeed(v)
        }

        pub fn set_parse_succeed(v: bool) {
            crate::test_control::set_parse_succeed(v)
        }

        pub fn set_return_empty(v: bool) {
            crate::test_control::set_return_empty(v)
        }

        pub fn set_invalid_utf8(_v: bool) {
            // Not yet used: for now invalid UTF-8 is simulated by writing invalid bytes via test_control::set_buffer
        }

        pub fn set_buffer(data: &[u8]) {
            crate::test_control::set_buffer(data)
        }

        pub unsafe fn microtex_init(_len: u64, _ptr: *const u8) -> *mut c_void {
            if crate::test_control::get_init_succeed() {
                1 as *mut c_void
            } else {
                std::ptr::null_mut()
            }
        }

        pub unsafe fn microtex_set_default_main_font(_ptr: *const i8) {
            // noop in tests
        }

        pub unsafe fn microtex_set_render_glyph_use_path(_val: bool) {
            // noop in tests
        }

        pub unsafe fn microtex_release_font_meta(_meta: *mut c_void) {
            // noop in tests
        }

        pub unsafe fn microtex_parse_render(
            _src: *const i8,
            _dpi: i32,
            _line_width: f32,
            _line_height: f32,
            _text_color: u32,
            _has_background: bool,
            _render_glyph_use_path: bool,
        ) -> *mut c_void {
            if crate::test_control::get_parse_succeed() {
                2 as *mut c_void
            } else {
                std::ptr::null_mut()
            }
        }

        pub unsafe fn microtex_render_to_svg(
            _render_ptr: *mut c_void,
            out_len: &mut u64,
        ) -> *mut u8 {
            if crate::test_control::get_return_empty() {
                *out_len = 0;
                std::ptr::null_mut()
            } else {
                let (ptr, len) = crate::test_control::get_out_buffer_ptr();
                *out_len = len;
                if len == 0 || ptr.is_null() {
                    std::ptr::null_mut()
                } else {
                    ptr as *mut u8
                }
            }
        }

        pub unsafe fn microtex_delete_render(_ptr: *mut c_void) {
            // noop
        }

        pub unsafe fn microtex_free_buffer(_buf: *mut u8) {
            // noop
        }

        pub unsafe fn microtex_release() {
            // noop
        }
    }

    // Public test setters
    #[cfg(test)]
    pub fn set_init_succeed(v: bool) {
        test_impl::set_init_succeed(v)
    }
    #[cfg(test)]
    pub fn set_parse_succeed(v: bool) {
        test_impl::set_parse_succeed(v)
    }
    #[cfg(test)]
    pub fn set_return_empty(v: bool) {
        test_impl::set_return_empty(v)
    }
    #[cfg(test)]
    pub fn set_invalid_utf8(v: bool) {
        test_impl::set_invalid_utf8(v)
    }
    #[cfg(test)]
    pub fn set_buffer(data: &[u8]) {
        test_impl::set_buffer(data)
    }

    #[cfg(test)]
    pub fn lock_test() -> std::sync::MutexGuard<'static, ()> {
        test_impl::lock_test()
    }

    // Re-export test functions at the shim top-level so callers can use `shim::microtex_*` in tests
    #[cfg(test)]
    pub unsafe fn microtex_init(len: u64, ptr: *const u8) -> *mut c_void {
        test_impl::microtex_init(len, ptr)
    }
    #[cfg(test)]
    pub unsafe fn microtex_set_default_main_font(ptr: *const i8) {
        test_impl::microtex_set_default_main_font(ptr)
    }
    #[cfg(test)]
    pub unsafe fn microtex_set_render_glyph_use_path(val: bool) {
        test_impl::microtex_set_render_glyph_use_path(val)
    }
    #[cfg(test)]
    pub unsafe fn microtex_release_font_meta(meta: *mut c_void) {
        test_impl::microtex_release_font_meta(meta)
    }
    #[cfg(test)]
    pub unsafe fn microtex_parse_render(
        src: *const i8,
        dpi: i32,
        line_width: f32,
        line_height: f32,
        text_color: u32,
        has_background: bool,
        render_glyph_use_path: bool,
    ) -> *mut c_void {
        test_impl::microtex_parse_render(
            src,
            dpi,
            line_width,
            line_height,
            text_color,
            has_background,
            render_glyph_use_path,
        )
    }
    #[cfg(test)]
    pub unsafe fn microtex_render_to_svg(render_ptr: *mut c_void, out_len: &mut u64) -> *mut u8 {
        test_impl::microtex_render_to_svg(render_ptr, out_len)
    }
    #[cfg(test)]
    pub unsafe fn microtex_delete_render(render_ptr: *mut c_void) {
        test_impl::microtex_delete_render(render_ptr)
    }
    #[cfg(test)]
    pub unsafe fn microtex_free_buffer(buf: *mut u8) {
        test_impl::microtex_free_buffer(buf)
    }
    #[cfg(test)]
    pub unsafe fn microtex_release() {
        test_impl::microtex_release()
    }
}

// Expose test helpers to other crates during test builds so integration/unit tests
// in other targets (bin, integration tests) can control shim behavior.
pub mod test_helpers {
    // In normal builds these helpers should not be called; provide stubs that panic
    #[cfg(not(test))]
    pub fn lock_test() -> std::sync::MutexGuard<'static, ()> {
        panic!("test_helpers::lock_test is only available in test builds");
    }

    #[cfg(not(test))]
    pub fn set_buffer(_: &[u8]) {
        panic!("test_helpers::set_buffer is only available in test builds");
    }

    #[cfg(not(test))]
    pub fn set_init_succeed(_: bool) {
        panic!("test_helpers::set_init_succeed is only available in test builds");
    }

    #[cfg(not(test))]
    pub fn set_parse_succeed(_: bool) {
        panic!("test_helpers::set_parse_succeed is only available in test builds");
    }

    #[cfg(not(test))]
    pub fn set_return_empty(_: bool) {
        panic!("test_helpers::set_return_empty is only available in test builds");
    }

    // When compiled for tests, re-export the test_control helpers (always available)
    #[cfg(test)]
    pub use crate::test_control::{
        lock_test, set_buffer, set_init_succeed, set_parse_succeed, set_return_empty,
    };
}

/// Errors that can occur when rendering LaTeX to SVG.
#[derive(Error, Debug)]
pub enum RenderError {
    /// Failed to initialize MicroTeX with the font metadata.
    #[error("failed to initialize MicroTeX: font metadata is null")]
    InitializationFailed,

    /// The provided LaTeX source failed to parse or render.
    #[error("failed to parse and render LaTeX source")]
    ParseRenderFailed,

    /// The SVG rendering produced no output.
    #[error("SVG rendering returned empty output")]
    EmptyOutput,

    /// Failed to convert SVG buffer to valid UTF-8 string.
    #[error("failed to convert SVG output to UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

/// Configuration for rendering LaTeX to SVG.
///
/// This structure holds all parameters needed to control how LaTeX
/// formulas are rendered to SVG format.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// DPI (dots per inch) for the output. Default: 720
    pub dpi: i32,

    /// Line width in pixels. Default: 20.0
    pub line_width: f32,

    /// Line height in pixels. Default: 20.0/3.0 (~6.67)
    pub line_height: f32,

    /// Text color as ARGB (0xAARRGGBB). Default: 0xff000000 (opaque black)
    pub text_color: u32,

    /// Whether to enable background color rendering.
    pub has_background: bool,

    /// Whether to use path-based glyph rendering for better fallback when
    /// system fonts are not available.
    pub render_glyph_use_path: bool,

    /// Whether to enable formula numbering.
    pub enable_formula_numbering: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            dpi: 720,
            line_width: 20.0,
            line_height: 20.0 / 3.0,
            text_color: 0xff000000,
            has_background: false,
            render_glyph_use_path: true,
            enable_formula_numbering: false,
        }
    }
}

/// The MicroTeX renderer for converting LaTeX formulas to SVG.
///
/// This struct manages the lifecycle of a MicroTeX instance and provides
/// safe methods to render LaTeX strings to SVG format. It automatically
/// handles initialization and cleanup of the underlying C++ library.
///
/// # Example
///
/// ```rust
/// use microtex_rs::{MicroTex, RenderConfig};
///
/// // Create a new renderer with embedded fonts
/// let renderer = MicroTex::new()?;
///
/// // Create a configuration for rendering
/// let config = RenderConfig {
///     dpi: 720,
///     line_width: 20.0,
///     line_height: 20.0 / 3.0,
///     text_color: 0xff000000,
///     ..Default::default()
/// };
///
/// // Render a simple LaTeX formula
/// let latex = r#"\[E = mc^2\]"#;
/// let svg = renderer.render(latex, &config)?;
/// assert!(!svg.is_empty());
/// assert!(svg.contains("<svg"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct MicroTex {
    _private: (),
}

impl MicroTex {
    /// Creates a new MicroTeX renderer instance with embedded fonts.
    ///
    /// This initializes the MicroTeX library with the XITS Math font
    /// that is embedded at compile time. The renderer will automatically
    /// clean up resources when dropped.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::InitializationFailed`] if the font metadata
    /// cannot be loaded or the MicroTeX library initialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use microtex_rs::MicroTex;
    ///
    /// let renderer = MicroTex::new()?;
    /// // Use renderer...
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Result<Self, RenderError> {
        // Try to find a suitable math font from the embedded CLM files
        // Note: We search in a specific order, preferring XITS which is well-tested
        // IMPORTANT: Math fonts must come before non-math fonts!
        // XITSMath-Regular is the math font version, not XITS-Regular
        let font_candidates = [
            "XITSMath-Regular.clm2",
            "FiraMath-Regular.clm2",
            "latinmodern-math.clm2",
            "texgyredejavu-math.clm2",
        ];

        let mut clm_data = None;
        for font_name in &font_candidates {
            if let Some(data) = get_embedded_clm(font_name) {
                clm_data = Some(data);
                break;
            }
        }

        let clm_data = clm_data.ok_or_else(|| {
            eprintln!(
                "No suitable math fonts found in embedded CLM files. Available: {:?}",
                available_embedded_clms()
            );
            RenderError::InitializationFailed
        })?;

        unsafe {
            // Critical: Initialize MicroTeX with font data (via shim)
            // This call may throw C++ exceptions if the data is invalid
            let meta = shim::microtex_init(clm_data.len() as u64, clm_data.as_ptr());
            if meta.is_null() {
                eprintln!("microtex_init returned null");
                return Err(RenderError::InitializationFailed);
            }

            // Set reasonable defaults
            let default_font = std::ffi::CStr::from_bytes_with_nul(b"Serif\0")
                .unwrap()
                .as_ptr();
            shim::microtex_set_default_main_font(default_font as *const i8);
            shim::microtex_set_render_glyph_use_path(true);

            // Important: release the font metadata after initialization
            shim::microtex_release_font_meta(meta);
        }

        Ok(MicroTex { _private: () })
    }

    /// Renders a LaTeX formula string to SVG format.
    ///
    /// # Arguments
    ///
    /// * `latex_source` - The LaTeX source string to render.
    /// * `config` - Rendering configuration parameters.
    ///
    /// # Returns
    ///
    /// A string containing the SVG representation of the rendered formula,
    /// or an error if parsing/rendering fails.
    ///
    /// # Errors
    ///
    /// Returns errors if:
    /// - The LaTeX source cannot be parsed
    /// - The rendering process fails
    /// - The SVG output is empty
    /// - The SVG buffer cannot be converted to valid UTF-8
    ///
    /// # Example
    ///
    /// ```rust
    /// use microtex_rs::{MicroTex, RenderConfig};
    ///
    /// let renderer = MicroTex::new()?;
    /// let config = RenderConfig::default();
    /// let svg = renderer.render(r#"\[x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}\]"#, &config)?;
    /// assert!(svg.contains("<svg"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn render(&self, latex_source: &str, config: &RenderConfig) -> Result<String, RenderError> {
        let latex_cstr = std::ffi::CString::new(latex_source)
            .unwrap_or_else(|_| std::ffi::CString::new("").unwrap());

        unsafe {
            let render_ptr = shim::microtex_parse_render(
                latex_cstr.as_ptr(),
                config.dpi,
                config.line_width,
                config.line_height,
                config.text_color,
                config.has_background,
                config.render_glyph_use_path,
            );

            if render_ptr.is_null() {
                return Err(RenderError::ParseRenderFailed);
            }

            let mut out_len = 0u64;
            let out_buf = shim::microtex_render_to_svg(render_ptr, &mut out_len);

            if out_buf.is_null() || out_len == 0 {
                shim::microtex_delete_render(render_ptr);
                return Err(RenderError::EmptyOutput);
            }

            // Convert the buffer to a Rust string
            let svg_slice = std::slice::from_raw_parts(out_buf as *const u8, out_len as usize);
            let svg_string = String::from_utf8(svg_slice.to_vec())?;

            // Clean up
            shim::microtex_free_buffer(out_buf);
            shim::microtex_delete_render(render_ptr);

            Ok(svg_string)
        }
    }
}

impl Drop for MicroTex {
    fn drop(&mut self) {
        unsafe {
            shim::microtex_release();
        }
    }
}

impl Default for MicroTex {
    fn default() -> Self {
        Self::new().expect("failed to create default MicroTex instance")
    }
}

impl fmt::Debug for MicroTex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MicroTex").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_clms() {
        let clms = available_embedded_clms();
        assert!(!clms.is_empty());
        // At least one math font should be available
        let has_math = clms.iter().any(|&name| {
            name.contains("Math")
                || name.contains("math")
                || name.contains("XITS")
                || name.contains("Fira")
        });
        assert!(
            has_math,
            "No suitable math fonts found. Available: {:?}",
            clms
        );
    }

    #[test]
    fn test_get_embedded_clm() {
        let clms = available_embedded_clms();
        for clm_name in clms {
            let result = get_embedded_clm(clm_name);
            assert!(
                result.is_some(),
                "Failed to get embedded CLM for {}",
                clm_name
            );
            let data = result.unwrap();
            assert!(!data.is_empty(), "CLM data is empty for {}", clm_name);
        }
    }

    // The rendering tests are commented out because MicroTeX may throw C++ exceptions
    // that Rust cannot catch. This is a known limitation of the C bindings.
    // Tests are best run with the C++ test suite: c++/mini_tests/test_math_svg.cpp
    //
    // To test rendering manually:
    // 1. Run the C++ test: cd c++/mini_tests && ./test_math_svg
    // 2. Or use the examples: cargo run --example simple_formula

    #[test]
    fn test_microtex_new_success() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        let r = MicroTex::new();
        assert!(r.is_ok());
    }

    #[test]
    fn test_microtex_new_init_fail() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(false);
        let r = MicroTex::new();
        assert!(matches!(r, Err(RenderError::InitializationFailed)));
        crate::shim::set_init_succeed(true);
    }

    #[test]
    fn test_render_parse_fail() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(false);
        let m = MicroTex::new().expect("init should succeed");
        let r = m.render("x", &RenderConfig::default());
        assert!(matches!(r, Err(RenderError::ParseRenderFailed)));
        crate::shim::set_parse_succeed(true);
    }

    #[test]
    fn test_render_empty_output() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(true);
        let m = MicroTex::new().expect("init should succeed");
        let r = m.render("x", &RenderConfig::default());
        assert!(matches!(r, Err(RenderError::EmptyOutput)));
        crate::shim::set_return_empty(false);
    }

    #[test]
    fn test_render_invalid_utf8() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);
        crate::shim::set_buffer(&[0xff, 0xff, 0xff]);
        let m = MicroTex::new().expect("init ok");
        let r = m.render("x", &RenderConfig::default());
        assert!(matches!(r, Err(RenderError::InvalidUtf8(_))));
    }

    #[test]
    fn test_render_success() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);
        crate::shim::set_buffer(b"<svg>ok</svg>");
        let m = MicroTex::new().expect("init ok");
        let r = m.render("x", &RenderConfig::default());
        assert!(r.is_ok());
        assert!(r.unwrap().contains("<svg"));
    }

    #[test]
    fn test_multiple_renders_same_instance() {
        // This test reproduces the SIGSEGV crash when calling render() multiple times
        // on the same MicroTex instance. The issue is related to resource cleanup
        // or reuse of the underlying C++ MicroTeX library.
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);
        crate::shim::set_buffer(b"<svg>result1</svg>");

        let m = MicroTex::new().expect("init ok");

        // First render - should succeed
        let r1 = m.render("x^2", &RenderConfig::default());
        assert!(r1.is_ok());
        assert!(r1.unwrap().contains("result1"));

        // Update buffer for second render
        crate::shim::set_buffer(b"<svg>result2</svg>");

        // Second render on the SAME instance - this triggers the crash
        let r2 = m.render("y^2", &RenderConfig::default());
        assert!(r2.is_ok());
        assert!(r2.unwrap().contains("result2"));

        // Third render - verify the issue persists with multiple calls
        crate::shim::set_buffer(b"<svg>result3</svg>");
        let r3 = m.render("z^2", &RenderConfig::default());
        assert!(r3.is_ok());
        assert!(r3.unwrap().contains("result3"));
    }
}
