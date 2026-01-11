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

use serde::{Deserialize, Serialize};
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

    // Non-test shim: convert Rust u64 length to the C `unsigned long` width
    // expected by the generated bindings. On Windows `unsigned long` is 32-bit,
    // on Unix it is typically 64-bit. Use conditional compilation to handle both.
    #[cfg(all(not(test), target_os = "windows"))]
    pub unsafe fn microtex_init(len: u64, ptr: *const u8) -> *mut c_void {
        // Convert to c_ulong (u32 on Windows). Panic if impossible (overflow).
        super::ffi::microtex_init(len.try_into().unwrap(), ptr as *const _)
    }

    #[cfg(all(not(test), not(target_os = "windows")))]
    pub unsafe fn microtex_init(len: u64, ptr: *const u8) -> *mut c_void {
        // On Unix-like systems c_ulong is typically 64-bit so pass through
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

    #[cfg(all(not(test), target_os = "windows"))]
    pub unsafe fn microtex_render_to_svg(render_ptr: *mut c_void, out_len: &mut u64) -> *mut u8 {
        // Windows uses 32-bit c_ulong; call the FFI with a local u32 and then
        // copy it back into the provided u64 reference.
        let mut len32: std::os::raw::c_ulong = 0;
        let ptr = super::ffi::microtex_render_to_svg(render_ptr as *mut _, &mut len32 as *mut _);
        *out_len = len32 as u64;
        ptr
    }

    #[cfg(all(not(test), not(target_os = "windows")))]
    pub unsafe fn microtex_render_to_svg(render_ptr: *mut c_void, out_len: &mut u64) -> *mut u8 {
        // On Unix-like systems the binding's c_ulong will match u64
        super::ffi::microtex_render_to_svg(render_ptr as *mut _, out_len)
    }

    /// Wrapper for microtex_render_to_svg_with_metrics.
    ///
    /// Calls the C++ FFI function that returns a JSON buffer containing SVG + metrics.
    /// On Windows, converts between 32-bit and 64-bit unsigned long types.
    #[cfg(all(not(test), target_os = "windows"))]
    pub unsafe fn microtex_render_to_svg_with_metrics(
        render_ptr: *mut c_void,
        out_len: &mut u64,
    ) -> *mut u8 {
        // Windows uses 32-bit c_ulong; call the FFI with a local u32 and then
        // copy it back into the provided u64 reference.
        let mut len32: std::os::raw::c_ulong = 0;
        let ptr = super::ffi::microtex_render_to_svg_with_metrics(
            render_ptr as *mut _,
            &mut len32 as *mut _,
        );
        *out_len = len32 as u64;
        ptr
    }

    /// Wrapper for microtex_render_to_svg_with_metrics on Unix-like systems.
    ///
    /// On Unix-like systems the binding's c_ulong will match u64, so we pass through directly.
    #[cfg(all(not(test), not(target_os = "windows")))]
    pub unsafe fn microtex_render_to_svg_with_metrics(
        render_ptr: *mut c_void,
        out_len: &mut u64,
    ) -> *mut u8 {
        // On Unix-like systems the binding's c_ulong will match u64
        super::ffi::microtex_render_to_svg_with_metrics(render_ptr as *mut _, out_len)
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_delete_render(render_ptr: *mut c_void) {
        super::ffi::microtex_deleteRender(render_ptr as *mut _);
    }

    #[cfg(not(test))]
    pub unsafe fn microtex_free_buffer(buf: *mut u8) {
        super::ffi::microtex_free_buffer(buf as *mut _);
    }

    /// Wrapper for microtex_get_key_char_metrics.
    ///
    /// Calls the C++ FFI function that returns a JSON buffer with key character metrics.
    /// On Windows, converts between 32-bit and 64-bit unsigned long types.
    #[cfg(all(not(test), target_os = "windows"))]
    pub unsafe fn microtex_get_key_char_metrics(
        render_ptr: *mut c_void,
        out_len: &mut u64,
    ) -> *mut u8 {
        let mut len32: std::os::raw::c_ulong = 0;
        let ptr =
            super::ffi::microtex_get_key_char_metrics(render_ptr as *mut _, &mut len32 as *mut _);
        *out_len = len32 as u64;
        ptr
    }

    /// Wrapper for microtex_get_key_char_metrics on Unix-like systems.
    ///
    /// On Unix-like systems the binding's c_ulong will match u64, so we pass through directly.
    #[cfg(all(not(test), not(target_os = "windows")))]
    pub unsafe fn microtex_get_key_char_metrics(
        render_ptr: *mut c_void,
        out_len: &mut u64,
    ) -> *mut u8 {
        super::ffi::microtex_get_key_char_metrics(render_ptr as *mut _, out_len)
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

        /// Test implementation of microtex_render_to_svg_with_metrics.
        ///
        /// Returns the buffer configured via test_control::set_buffer, which should
        /// contain JSON with SVG and metrics data.
        pub unsafe fn microtex_render_to_svg_with_metrics(
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

        /// Test implementation of microtex_get_key_char_metrics.
        ///
        /// Returns the buffer configured via test_control::set_buffer, which should
        /// contain JSON with key character metrics data.
        pub unsafe fn microtex_get_key_char_metrics(
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
    /// Test wrapper for microtex_render_to_svg_with_metrics.
    ///
    /// Delegates to the test_impl implementation which uses test_control::get_out_buffer_ptr().
    pub unsafe fn microtex_render_to_svg_with_metrics(
        render_ptr: *mut c_void,
        out_len: &mut u64,
    ) -> *mut u8 {
        test_impl::microtex_render_to_svg_with_metrics(render_ptr, out_len)
    }
    #[cfg(test)]
    /// Test wrapper for microtex_get_key_char_metrics.
    ///
    /// Delegates to the test_impl implementation which uses test_control::get_out_buffer_ptr().
    pub unsafe fn microtex_get_key_char_metrics(
        render_ptr: *mut c_void,
        out_len: &mut u64,
    ) -> *mut u8 {
        test_impl::microtex_get_key_char_metrics(render_ptr, out_len)
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

    /// Failed to parse the JSON metrics response from the C++ renderer.
    #[error("failed to parse JSON metrics: {0}")]
    ParseJsonFailed(String),
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

/// Dimensional metrics from rendering a LaTeX formula to SVG.
///
/// This structure contains the precise dimensional information of a rendered
/// formula, useful for proper scaling and positioning in PDF documents.
#[derive(Debug, Clone)]
pub struct RenderMetrics {
    /// The width of the rendered formula in pixels.
    pub width: i32,

    /// The total height of the rendered formula in pixels (height + depth).
    pub height: i32,

    /// The depth of the rendered formula below the baseline in pixels.
    pub depth: i32,

    /// The ascent of the rendered formula (height without depth) in pixels.
    pub ascent: i32,
}

impl RenderMetrics {
    /// Creates a new RenderMetrics instance with the specified dimensions.
    ///
    /// # Arguments
    /// * `width` - The width in pixels
    /// * `height` - The total height (height + depth) in pixels
    /// * `depth` - The depth below baseline in pixels
    /// * `ascent` - The ascent (height without depth) in pixels
    pub fn new(width: i32, height: i32, depth: i32, ascent: i32) -> Self {
        Self {
            width,
            height,
            depth,
            ascent,
        }
    }

    /// Returns the effective visual height of the rendered content.
    ///
    /// This is the total height including both ascent and descent.
    pub fn total_height(&self) -> f32 {
        self.height as f32
    }

    /// Returns the aspect ratio (width / height) of the rendered content.
    ///
    /// Useful for maintaining proportional scaling when resizing.
    pub fn aspect_ratio(&self) -> f32 {
        if self.height > 0 {
            self.width as f32 / self.height as f32
        } else {
            1.0
        }
    }

    /// Returns the baseline ratio (ascent / total height) of the rendered content.
    ///
    /// This indicates how much of the formula's height is above the baseline.
    /// - Values close to 1.0: tall formulas with many superscripts
    /// - Values close to 0.5: balanced formulas
    /// - Values close to 0.0: deep formulas with many subscripts or fractions
    pub fn baseline_ratio(&self) -> f32 {
        if self.height > 0 {
            self.ascent as f32 / self.height as f32
        } else {
            0.5
        }
    }
}

/// Result type containing both SVG content and dimensional metrics.
///
/// Returned by rendering functions that need to provide both the rendered
/// SVG string and precise dimensional information for further processing.
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// The SVG content as a UTF-8 string.
    pub svg: String,

    /// The dimensional metrics of the rendered formula.
    pub metrics: RenderMetrics,

    /// Metrics of key characters in the formula (optional).
    /// Available when rendering with KeyCharMetrics extraction.
    pub key_char_metrics: Option<KeyCharMetrics>,
}

impl RenderResult {
    /// Creates a new RenderResult with SVG content and metrics.
    pub fn new(svg: String, metrics: RenderMetrics) -> Self {
        Self {
            svg,
            metrics,
            key_char_metrics: None,
        }
    }

    /// Creates a new RenderResult with SVG content, metrics, and key character metrics.
    pub fn with_key_char_metrics(
        svg: String,
        metrics: RenderMetrics,
        key_char_metrics: KeyCharMetrics,
    ) -> Self {
        Self {
            svg,
            metrics,
            key_char_metrics: Some(key_char_metrics),
        }
    }
}

/// Metrics for key characters extracted from the formula's BOX TREE.
///
/// Contains the heights of actual character boxes at the top level of the
/// formula structure, excluding decorative elements and nested structures.
/// This is used to calculate more accurate scaling factors that account
/// for formula complexity (fractions, subscripts, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCharMetrics {
    /// Heights of individual key characters in the formula
    pub key_char_heights: Vec<i32>,

    /// Number of key characters found
    pub key_char_count: i32,

    /// Average height of key characters
    pub average_char_height: f32,

    /// Maximum character height
    pub max_char_height: i32,

    /// Minimum character height
    pub min_char_height: i32,

    /// Total height of BOX TREE root in MicroTeX units (used for normalization)
    pub box_tree_height: f32,
}

impl KeyCharMetrics {
    /// Creates new KeyCharMetrics from parsed JSON data.
    pub fn new(
        key_char_heights: Vec<i32>,
        key_char_count: i32,
        average_char_height: f32,
        max_char_height: i32,
        min_char_height: i32,
        box_tree_height: f32,
    ) -> Self {
        Self {
            key_char_heights,
            key_char_count,
            average_char_height,
            max_char_height,
            min_char_height,
            box_tree_height,
        }
    }

    /// Parses KeyCharMetrics from a JSON string returned from C++.
    pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let value: serde_json::Value = serde_json::from_str(json)?;

        let key_char_heights: Vec<i32> = value
            .get("key_char_heights")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_i64())
                    .map(|v| v as i32)
                    .collect()
            })
            .unwrap_or_default();

        let key_char_count = value
            .get("key_char_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        let average_char_height = value
            .get("average_char_height")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        let max_char_height = value
            .get("max_char_height")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        let min_char_height = value
            .get("min_char_height")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        let box_tree_height = value
            .get("box_tree_height")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;

        Ok(Self {
            key_char_heights,
            key_char_count,
            average_char_height,
            max_char_height,
            min_char_height,
            box_tree_height,
        })
    }
}

///
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

/// Adds DPI metadata to an SVG string as a `data-dpi` attribute.
///
/// This function injects the rendering DPI value into the SVG root element
/// as a `data-dpi` attribute. This metadata is useful for downstream processors
/// that need to know the DPI at which the SVG was rendered, particularly when
/// converting to other formats (e.g., PDF) where proper sizing depends on
/// knowing the original DPI.
///
/// # Arguments
///
/// * `svg` - The SVG content as a string
/// * `dpi` - The DPI value to embed (typically 720 for MicroTeX)
///
/// # Returns
///
/// A modified SVG string with the `data-dpi` attribute added to the `<svg>` element.
/// If the SVG doesn't contain an `<svg` opening tag, the original string is returned unchanged.
///
/// # Example
///
/// ```rust
/// use microtex_rs::add_dpi_to_svg;
///
/// let svg = r#"<svg width="100" height="50" xmlns="http://www.w3.org/2000/svg"></svg>"#;
/// let dpi = 720;
/// let modified = add_dpi_to_svg(svg, dpi);
/// assert!(modified.contains(r#"data-dpi="720""#));
/// ```
pub fn add_dpi_to_svg(svg: &str, dpi: i32) -> String {
    // Find the opening <svg tag
    if let Some(svg_start) = svg.find("<svg") {
        if let Some(close_bracket) = svg[svg_start..].find('>') {
            let insert_pos = svg_start + close_bracket;
            let mut result = String::with_capacity(svg.len() + 20);
            result.push_str(&svg[..insert_pos]);
            result.push_str(&format!(r#" data-dpi="{}""#, dpi));
            result.push_str(&svg[insert_pos..]);
            return result;
        }
    }
    // If no <svg tag found or malformed, return original
    svg.to_string()
}

/// Extracts all Y coordinates from SVG path elements, accounting for transformations.
///
/// This function parses all `<path>` elements in an SVG and extracts Y coordinates
/// from the path data (M, L, C, Q, etc. commands). It applies any `transform="matrix(...)"`
/// attributes to get the actual Y coordinates after transformation.
///
/// # Arguments
///
/// * `svg` - The SVG content as a string
///
/// # Returns
///
/// A vector of all Y coordinate values found in path data after applying transformations.
/// Returns an empty vector if no paths or coordinates are found.
///
/// # Example
///
/// ```rust
/// use microtex_rs::extract_y_coordinates;
///
/// let svg = r#"<svg><path d="M 10 20 L 30 40 Z"/></svg>"#;
/// let y_coords = extract_y_coordinates(svg);
/// assert!(y_coords.contains(&20.0));
/// assert!(y_coords.contains(&40.0));
/// ```
pub fn extract_y_coordinates(svg: &str) -> Vec<f32> {
    let mut y_coords = Vec::new();

    // Find all <path> elements
    let mut search_start = 0;
    while let Some(path_start) = svg[search_start..].find("<path") {
        let path_start = search_start + path_start;

        // Extract the transform matrix if present
        // Look for transform="matrix(a, b, c, d, e, f)"
        let transform_matrix =
            if let Some(transform_idx) = svg[path_start..].find(r#"transform="matrix("#) {
                let transform_start = path_start + transform_idx + 18; // Skip 'transform="matrix('
                if let Some(close_paren) = svg[transform_start..].find(')') {
                    let matrix_str = &svg[transform_start..transform_start + close_paren];
                    // Parse matrix values: a, b, c, d, e, f
                    let values: Vec<f32> = matrix_str
                        .split(',')
                        .filter_map(|s| s.trim().parse::<f32>().ok())
                        .collect();

                    if values.len() >= 6 {
                        Some((
                            values[0], values[1], values[2], values[3], values[4], values[5],
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

        // Find the d=" attribute
        if let Some(d_attr_start) = svg[path_start..].find(r#"d=""#) {
            let d_start = path_start + d_attr_start + 3; // Skip d="

            // Find the closing quote of the d attribute
            if let Some(d_end) = svg[d_start..].find('"') {
                let d_content = &svg[d_start..d_start + d_end];

                // Parse the path data
                let mut chars = d_content.chars().peekable();
                let mut current_num = String::new();
                let mut coords = Vec::new();

                while let Some(ch) = chars.next() {
                    match ch {
                        '0'..='9' | '-' | '.' => {
                            current_num.push(ch);
                        }
                        ' ' | ',' | '\n' | '\t' | '\r' => {
                            if !current_num.is_empty() {
                                if let Ok(num) = current_num.parse::<f32>() {
                                    coords.push(num);
                                }
                                current_num.clear();
                            }
                        }
                        'M' | 'L' | 'H' | 'V' | 'C' | 'S' | 'Q' | 'T' | 'A' | 'Z' | 'm' | 'l'
                        | 'h' | 'v' | 'c' | 's' | 'q' | 't' | 'a' | 'z' => {
                            if !current_num.is_empty() {
                                if let Ok(num) = current_num.parse::<f32>() {
                                    coords.push(num);
                                }
                                current_num.clear();
                            }
                        }
                        _ => {
                            if !current_num.is_empty() {
                                if let Ok(num) = current_num.parse::<f32>() {
                                    coords.push(num);
                                }
                                current_num.clear();
                            }
                        }
                    }
                }

                // Handle the last number if any
                if !current_num.is_empty() {
                    if let Ok(num) = current_num.parse::<f32>() {
                        coords.push(num);
                    }
                }

                // Parse coordinates based on SVG path commands
                // Most commands have Y coordinates at specific positions
                // For simplicity, we assume coordinates alternate X, Y in most cases
                // This is a heuristic approach - we collect every other coordinate as Y
                let mut i = 0;
                while i < coords.len() {
                    // Most path commands use X, Y pairs
                    // We extract Y coordinates (every second value in most cases)
                    if i + 1 < coords.len() {
                        let mut y = coords[i + 1]; // Y coordinate

                        // Apply transformation matrix if present
                        if let Some((a, b, c, d, e, f)) = transform_matrix {
                            let x = coords[i]; // X coordinate for transformation
                                               // y' = b*x + d*y + f
                            y = b * x + d * y + f;
                        }

                        y_coords.push(y);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }

                search_start = d_start + d_end + 1;
            } else {
                search_start = path_start + 1;
            }
        } else {
            search_start = path_start + 1;
        }
    }

    y_coords
}

/// Adjusts SVG height and viewBox, then centers content with a transform group.
///
/// This function analyzes the actual Y coordinates in the SVG, increases the height
/// if needed, and wraps the content in a `<g>` element with a vertical translation
/// to center the content. This prevents clipping of glyphs that exceed the declared height.
///
/// # Arguments
///
/// * `svg` - The SVG content as a string
///
/// # Returns
///
/// A modified SVG string with adjusted height/viewBox and centered content, or the
/// original SVG if max_y < 0.02 (within tolerance).
///
/// # Algorithm
///
/// 1. Extract all Y coordinates (accounting for transformations)
/// 2. Find max_y value
/// 3. If max_y < 0.02, return SVG unchanged (within tolerance)
/// 4. Otherwise:
///    - Calculate new_height = ceil(max_y)
///    - Update height and viewBox height attributes
///    - Wrap all path elements in a `<g>` with translate(0, -max_y/2)
/// 5. Return modified SVG
///
/// # Example
///
/// ```rust
/// use microtex_rs::adjust_svg_height_and_center;
///
/// let svg = r#"<svg width="188" height="39" viewBox="0 0 188 39">
///   <path d="M 10 20 L 30 39.121094 Z"/>
/// </svg>"#;
/// let adjusted = adjust_svg_height_and_center(svg);
/// // adjusted now has height="40" and viewBox="0 0 188 40"
/// // and content wrapped in <g transform="translate(0, -19.560547)">
/// ```
pub fn adjust_svg_height_and_center(svg: &str) -> String {
    use quick_xml::events::{BytesEnd, BytesStart, Event};
    use quick_xml::Reader;
    use quick_xml::Writer;
    use std::io::Cursor;

    // Extract Y coordinates and find max
    let y_coords = extract_y_coordinates(svg);
    if y_coords.is_empty() {
        return svg.to_string();
    }

    let max_y = y_coords.iter().copied().fold(f32::NEG_INFINITY, f32::max);

    // If max_y is within tolerance, return SVG unchanged
    if max_y < 0.02 {
        return svg.to_string();
    }

    // Calculate new height
    let new_height = max_y.ceil() as i32;
    let translate_y = (new_height as f32 - max_y) / 2.0;
    let height_str = new_height.to_string();
    let transform_str = format!("translate(0, {})", translate_y);

    // Parse and rebuild SVG with quick-xml
    let mut reader = Reader::from_str(svg);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buffer = Vec::new();
    let mut in_svg = false;
    let mut g_opened = false;
    let mut found_svg_end = false;

    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Text(e)) => {
                let _ = writer.write_event(Event::Text(e));
            }
            Ok(Event::Start(e)) => {
                let name = e.name();

                // Handle SVG tag
                if name.as_ref() == b"svg" {
                    in_svg = true;
                    let mut svg_start = BytesStart::new("svg");
                    let mut viewbox_new = String::new();

                    // Process attributes
                    for attr_result in e.attributes() {
                        if let Ok(attr) = attr_result {
                            let key_str = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            let value_str = std::str::from_utf8(&attr.value).unwrap_or("");

                            if key_str == "height" {
                                continue;
                            } else if key_str == "viewBox" {
                                let parts: Vec<&str> = value_str.split_whitespace().collect();
                                if parts.len() == 4 {
                                    viewbox_new = format!(
                                        "{} {} {} {}",
                                        parts[0], parts[1], parts[2], new_height
                                    );
                                    svg_start.push_attribute(("viewBox", viewbox_new.as_str()));
                                } else {
                                    svg_start.push_attribute((key_str, value_str));
                                }
                            } else {
                                svg_start.push_attribute((key_str, value_str));
                            }
                        }
                    }

                    svg_start.push_attribute(("height", height_str.as_str()));
                    let _ = writer.write_event(Event::Start(svg_start));
                } else if in_svg && !g_opened {
                    // Open <g> before first non-SVG child
                    let mut g_start = BytesStart::new("g");
                    g_start.push_attribute(("transform", transform_str.as_str()));
                    let _ = writer.write_event(Event::Start(g_start));
                    g_opened = true;

                    // Write the current element
                    let _ = writer.write_event(Event::Start(e));
                } else {
                    let _ = writer.write_event(Event::Start(e));
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();

                if in_svg && name.as_ref() == b"svg" {
                    // Close <g> before closing </svg>
                    if g_opened {
                        let _ = writer.write_event(Event::End(BytesEnd::new("g")));
                    }
                    let _ = writer.write_event(Event::End(e));
                    found_svg_end = true;
                    break; // Now we can break after processing </svg>
                } else {
                    let _ = writer.write_event(Event::End(e));
                }
            }
            Ok(event) => {
                match &event {
                    Event::Empty(e) => {
                        let name = e.name();

                        // If we haven't opened <g> yet and we're in SVG, open it now
                        if in_svg && !g_opened {
                            let mut g_start = BytesStart::new("g");
                            g_start.push_attribute(("transform", transform_str.as_str()));
                            let _ = writer.write_event(Event::Start(g_start));
                            g_opened = true;
                        }

                        let _ = writer.write_event(event);
                    }
                    _ => {
                        let _ = writer.write_event(event);
                    }
                }
            }
            Err(_) => break,
        }
    }

    let cursor = writer.into_inner();
    let bytes = cursor.into_inner();
    String::from_utf8_lossy(&bytes).to_string()
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
            let mut svg_string = String::from_utf8(svg_slice.to_vec())?;

            // Add DPI metadata to SVG
            svg_string = add_dpi_to_svg(&svg_string, config.dpi);

            // Adjust SVG height and center content to prevent glyph clipping
            svg_string = adjust_svg_height_and_center(&svg_string);

            // Clean up
            shim::microtex_free_buffer(out_buf);
            shim::microtex_delete_render(render_ptr);

            Ok(svg_string)
        }
    }

    /// Renders a LaTeX formula string to SVG format with dimensional metrics.
    ///
    /// This function is similar to [`render()`](Self::render), but also returns
    /// precise dimensional information (width, height, depth, ascent) extracted
    /// from the MicroTeX BOX TREE before SVG rendering. This is useful for
    /// accurate scaling and positioning of the rendered formula.
    ///
    /// # Arguments
    ///
    /// * `latex_source` - The LaTeX source string to render.
    /// * `config` - Rendering configuration parameters.
    ///
    /// # Returns
    ///
    /// A [`RenderResult`] containing both the SVG string and the metrics,
    /// or an error if parsing/rendering fails.
    ///
    /// # Errors
    ///
    /// Returns errors if:
    /// - The LaTeX source cannot be parsed
    /// - The rendering process fails
    /// - The output is empty
    /// - The SVG or metrics JSON cannot be parsed
    /// - Invalid UTF-8 is encountered
    ///
    /// # Example
    ///
    /// ```rust
    /// use microtex_rs::{MicroTex, RenderConfig};
    ///
    /// let renderer = MicroTex::new()?;
    /// let config = RenderConfig::default();
    /// let result = renderer.render_to_svg_with_metrics(r#"\[x^2\]"#, &config)?;
    /// println!("Width: {}, Height: {}", result.metrics.width, result.metrics.height);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn render_to_svg_with_metrics(
        &self,
        latex_source: &str,
        config: &RenderConfig,
    ) -> Result<RenderResult, RenderError> {
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
            let out_buf = shim::microtex_render_to_svg_with_metrics(render_ptr, &mut out_len);

            if out_buf.is_null() || out_len == 0 {
                shim::microtex_delete_render(render_ptr);
                return Err(RenderError::EmptyOutput);
            }

            // Convert the buffer to a Rust string
            let json_slice = std::slice::from_raw_parts(out_buf as *const u8, out_len as usize);
            let json_string = String::from_utf8(json_slice.to_vec())?;

            // Parse the JSON response from C++
            let json_value: serde_json::Value = serde_json::from_str(&json_string)
                .map_err(|e| RenderError::ParseJsonFailed(e.to_string()))?;

            // Extract SVG content
            let mut svg = json_value
                .get("svg")
                .and_then(|v| v.as_str())
                .ok_or_else(|| RenderError::ParseJsonFailed("missing 'svg' field".to_string()))?
                .to_string();

            // Add DPI metadata to SVG
            svg = add_dpi_to_svg(&svg, config.dpi);

            // Adjust SVG height and center content to prevent glyph clipping
            svg = adjust_svg_height_and_center(&svg);

            // Extract metrics
            let metrics_obj = json_value
                .get("metrics")
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    RenderError::ParseJsonFailed("missing 'metrics' field".to_string())
                })?;

            let width = metrics_obj
                .get("width")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    RenderError::ParseJsonFailed("missing or invalid 'width'".to_string())
                })? as i32;

            let height = metrics_obj
                .get("height")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    RenderError::ParseJsonFailed("missing or invalid 'height'".to_string())
                })? as i32;

            let depth = metrics_obj
                .get("depth")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    RenderError::ParseJsonFailed("missing or invalid 'depth'".to_string())
                })? as i32;

            let ascent = metrics_obj
                .get("ascent")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| {
                    RenderError::ParseJsonFailed("missing or invalid 'ascent'".to_string())
                })? as i32;

            let metrics = RenderMetrics::new(width, height, depth, ascent);

            // Try to extract key character metrics
            let key_char_metrics = get_key_char_metrics(render_ptr).ok();

            // Clean up
            shim::microtex_free_buffer(out_buf);
            shim::microtex_delete_render(render_ptr);

            let result = match key_char_metrics {
                Some(kcm) => RenderResult::with_key_char_metrics(svg, metrics, kcm),
                None => RenderResult::new(svg, metrics),
            };

            Ok(result)
        }
    }
}

/// Get metrics of key characters in a rendered formula.
///
/// This function extracts the heights of actual character boxes at the
/// top level of the formula structure, excluding decorative elements.
/// This is useful for calculating more accurate scaling factors that
/// account for formula complexity (fractions, subscripts, etc.).
///
/// # Arguments
///
/// * `render_ptr` - The render pointer from `parse_render`
///
/// # Returns
///
/// A `KeyCharMetrics` struct containing the heights of key characters
/// and statistical information about them.
///
/// # Errors
///
/// Returns [`RenderError`] if the rendering operation fails or the
/// JSON parsing fails.
pub fn get_key_char_metrics(
    render_ptr: *mut std::ffi::c_void,
) -> Result<KeyCharMetrics, RenderError> {
    if render_ptr.is_null() {
        return Err(RenderError::ParseRenderFailed);
    }

    unsafe {
        let mut out_len = 0u64;
        let out_buf = shim::microtex_get_key_char_metrics(render_ptr, &mut out_len);

        if out_buf.is_null() || out_len == 0 {
            return Err(RenderError::EmptyOutput);
        }

        // Convert the buffer to a Rust string
        let json_slice = std::slice::from_raw_parts(out_buf as *const u8, out_len as usize);
        let json_string = String::from_utf8(json_slice.to_vec())?;

        // Parse the JSON response
        let metrics = KeyCharMetrics::from_json(&json_string)
            .map_err(|e| RenderError::ParseJsonFailed(e.to_string()))?;

        // Clean up
        shim::microtex_free_buffer(out_buf);

        Ok(metrics)
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

    const COMPLEXE_SVG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="188" height="39" viewBox="0 0 188 39" data-dpi="720">
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 10.480469 23.28125 L 6.621094 14.480469 L 2.71875 23.28125 Z M 13.5 25.121094 L 0.960938 25.121094 L 6.941406 11.640625 L 7.339844 11.640625 Z M 13.5 25.121094 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 19.398438 16.378906 L 20.140625 16.378906 C 21.398438 16.378906 21.300781 14.839844 22.160156 13.558594 C 22.78125 12.621094 23.640625 11.761719 25.160156 11.761719 C 26.160156 11.761719 26.738281 12.238281 26.738281 12.941406 C 26.738281 13.5 26.320312 13.738281 25.921875 13.738281 C 25.359375 13.738281 25.21875 13.519531 25.21875 13.238281 C 25.21875 12.960938 25.398438 12.660156 25.398438 12.5 C 25.398438 12.398438 25.339844 12.339844 25.101562 12.339844 C 24.101562 12.339844 23.320312 13.421875 22.898438 14.980469 L 22.519531 16.378906 L 24.121094 16.378906 L 23.878906 17.140625 L 22.359375 17.140625 L 20.78125 23.261719 C 20.640625 23.800781 20.480469 24.558594 20.179688 25.300781 C 19.519531 26.960938 18.441406 28.859375 16.679688 28.859375 C 15.71875 28.859375 15.238281 28.378906 15.238281 27.78125 C 15.238281 27.300781 15.519531 26.800781 16.101562 26.800781 C 16.640625 26.800781 16.839844 27.160156 16.839844 27.460938 C 16.839844 27.839844 16.519531 27.859375 16.519531 28.101562 C 16.519531 28.21875 16.640625 28.28125 16.820312 28.28125 C 18.121094 28.28125 18.71875 25.121094 19.019531 23.980469 L 20.761719 17.140625 L 19.21875 17.140625 Z M 19.398438 16.378906 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 43.835938 22.71875 L 32.054688 22.71875 L 32.054688 21.398438 L 43.835938 21.398438 Z M 43.835938 18.71875 L 32.054688 18.71875 L 32.054688 17.398438 L 43.835938 17.398438 Z M 43.835938 18.71875 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 56.191406 8.898438 L 56.191406 2.039062 L 51.390625 8.898438 Z M 59.8125 10.179688 L 57.75 10.179688 L 57.75 13.519531 L 56.191406 13.519531 L 56.191406 10.179688 L 50.589844 10.179688 L 50.589844 8.898438 L 56.871094 0 L 57.75 0 L 57.75 8.898438 L 59.8125 8.898438 Z M 59.8125 10.179688 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 65.769531 8.078125 L 64.589844 7.28125 C 63.410156 8.238281 62.992188 9.058594 62.992188 10.359375 C 62.992188 12.199219 64.011719 13.238281 65.53125 13.238281 C 66.832031 13.238281 67.730469 12.339844 67.730469 11.039062 C 67.730469 9.800781 67.132812 9 65.769531 8.078125 Z M 67.449219 2.859375 C 67.449219 1.480469 66.632812 0.558594 65.269531 0.558594 C 63.929688 0.558594 63.070312 1.300781 63.070312 2.539062 C 63.070312 3.78125 63.8125 4.699219 65.570312 5.738281 C 66.929688 4.941406 67.449219 4.101562 67.449219 2.859375 Z M 69.25 10.421875 C 69.25 12.480469 67.710938 13.800781 65.3125 13.800781 C 63.050781 13.800781 61.472656 12.421875 61.472656 10.539062 C 61.472656 9.160156 61.929688 8.378906 64.070312 6.878906 C 62.011719 5.179688 61.589844 4.421875 61.589844 3.121094 C 61.589844 1.199219 63.25 0 65.472656 0 C 67.449219 0 68.832031 1.300781 68.832031 2.859375 C 68.832031 4.359375 68.132812 5.039062 66.152344 6.101562 C 68.609375 7.738281 69.25 8.820312 69.25 10.421875 Z M 69.25 10.421875 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 77.949219 7.019531 C 77.949219 2.859375 77.070312 0.519531 75.3125 0.519531 C 73.652344 0.519531 72.75 2.878906 72.75 6.941406 C 72.75 11 73.632812 13.28125 75.351562 13.28125 C 77.050781 13.28125 77.949219 10.980469 77.949219 7.019531 Z M 79.871094 6.921875 C 79.871094 10.359375 78.609375 13.800781 75.351562 13.800781 C 71.929688 13.800781 70.832031 10.078125 70.832031 6.800781 C 70.832031 3.261719 72.210938 0 75.429688 0 C 78.050781 0 79.871094 2.820312 79.871094 6.921875 Z M 79.871094 6.921875 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 87.949219 7.019531 C 87.949219 2.859375 87.070312 0.519531 85.3125 0.519531 C 83.652344 0.519531 82.75 2.878906 82.75 6.941406 C 82.75 11 83.632812 13.28125 85.351562 13.28125 C 87.050781 13.28125 87.949219 10.980469 87.949219 7.019531 Z M 89.871094 6.921875 C 89.871094 10.359375 88.609375 13.800781 85.351562 13.800781 C 81.929688 13.800781 80.832031 10.078125 80.832031 6.800781 C 80.832031 3.261719 82.210938 0 85.429688 0 C 88.050781 0 89.871094 2.820312 89.871094 6.921875 Z M 89.871094 6.921875 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 97.949219 7.019531 C 97.949219 2.859375 97.070312 0.519531 95.3125 0.519531 C 93.652344 0.519531 92.75 2.878906 92.75 6.941406 C 92.75 11 93.632812 13.28125 95.351562 13.28125 C 97.050781 13.28125 97.949219 10.980469 97.949219 7.019531 Z M 99.871094 6.921875 C 99.871094 10.359375 98.609375 13.800781 95.351562 13.800781 C 91.929688 13.800781 90.832031 10.078125 90.832031 6.800781 C 90.832031 3.261719 92.210938 0 95.429688 0 C 98.050781 0 99.871094 2.820312 99.871094 6.921875 Z M 99.871094 6.921875 "/>
<path fill="none" stroke-width="66" stroke-linecap="butt" stroke-linejoin="bevel" stroke="rgb(0%, 0%, 0%)" stroke-opacity="1" stroke-miterlimit="0" d="M 2517.578181 1006.05471 L 5017.578237 1006.05471 " transform="matrix(0.02, 0, 0, 0.02, 0, 0)"/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 61.191406 34.5 L 61.191406 27.640625 L 56.390625 34.5 Z M 64.8125 35.78125 L 62.75 35.78125 L 62.75 39.121094 L 61.191406 39.121094 L 61.191406 35.78125 L 55.589844 35.78125 L 55.589844 34.5 L 61.871094 25.601562 L 62.75 25.601562 L 62.75 34.5 L 64.8125 34.5 Z M 64.8125 35.78125 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 72.949219 32.621094 C 72.949219 28.460938 72.070312 26.121094 70.3125 26.121094 C 68.652344 26.121094 67.75 28.480469 67.75 32.539062 C 67.75 36.601562 68.632812 38.878906 70.351562 38.878906 C 72.050781 38.878906 72.949219 36.578125 72.949219 32.621094 Z M 74.871094 32.519531 C 74.871094 35.960938 73.609375 39.398438 70.351562 39.398438 C 66.929688 39.398438 65.832031 35.679688 65.832031 32.398438 C 65.832031 28.859375 67.210938 25.601562 70.429688 25.601562 C 73.050781 25.601562 74.871094 28.421875 74.871094 32.519531 Z M 74.871094 32.519531 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 82.589844 32.019531 L 82.589844 31.238281 C 82.589844 27.878906 81.691406 26.160156 79.949219 26.160156 C 79.351562 26.160156 78.832031 26.398438 78.492188 26.839844 C 78.089844 27.378906 77.792969 28.558594 77.792969 29.640625 C 77.792969 32.019531 78.75 33.519531 80.25 33.519531 C 81.132812 33.519531 82.589844 33.078125 82.589844 32.019531 Z M 76.53125 39.558594 L 76.472656 39.160156 C 79.511719 38.621094 81.75 36.519531 82.550781 33.238281 C 81.691406 34.078125 80.730469 34.378906 79.550781 34.378906 C 77.390625 34.378906 75.949219 32.761719 75.949219 30.320312 C 75.949219 27.621094 77.730469 25.601562 80.109375 25.601562 C 81.390625 25.601562 82.472656 26.160156 83.25 27.121094 C 84.050781 28.121094 84.53125 29.558594 84.53125 31.238281 C 84.53125 33.539062 83.730469 35.71875 82.132812 37.179688 C 80.429688 38.71875 79.132812 39.199219 76.53125 39.558594 Z M 76.53125 39.558594 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 92.910156 35.359375 C 92.910156 32.800781 92.070312 31.480469 90.210938 31.480469 C 89.070312 31.480469 87.890625 31.960938 87.890625 33.800781 C 87.890625 36.839844 88.851562 38.839844 90.730469 38.839844 C 92.171875 38.839844 92.910156 37.398438 92.910156 35.359375 Z M 94.269531 25.441406 L 94.3125 25.761719 C 91.171875 26.28125 88.929688 28.441406 88.390625 31.460938 C 89.371094 30.699219 90.050781 30.558594 90.929688 30.558594 C 93.269531 30.558594 94.710938 32.160156 94.710938 34.738281 C 94.710938 36.019531 94.351562 37.140625 93.691406 37.941406 C 92.949219 38.859375 91.832031 39.398438 90.511719 39.398438 C 88.929688 39.398438 87.671875 38.660156 86.972656 37.378906 C 86.410156 36.359375 86.03125 34.941406 86.03125 33.539062 C 86.03125 31.378906 86.792969 29.480469 88.210938 28.019531 C 89.929688 26.21875 91.511719 25.738281 94.269531 25.441406 Z M 94.269531 25.441406 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 117.988281 20.339844 L 118.648438 20.339844 C 118.527344 22.039062 117.445312 24.421875 115.488281 24.421875 C 113.566406 24.421875 111.425781 21.859375 109.988281 21.859375 C 108.585938 21.859375 107.847656 23.199219 107.527344 24.621094 L 106.867188 24.621094 C 106.964844 22.621094 108.167969 20.539062 110.125 20.539062 C 112.046875 20.539062 114.1875 23.101562 115.648438 23.101562 C 117.027344 23.101562 117.6875 21.761719 117.988281 20.339844 Z M 117.988281 15.621094 L 118.648438 15.621094 C 118.527344 17.320312 117.445312 19.699219 115.488281 19.699219 C 113.566406 19.699219 111.425781 17.140625 109.988281 17.140625 C 108.585938 17.140625 107.847656 18.480469 107.527344 19.898438 L 106.867188 19.898438 C 106.964844 17.898438 108.167969 15.820312 110.125 15.820312 C 112.046875 15.820312 114.1875 18.378906 115.648438 18.378906 C 117.027344 18.378906 117.6875 17.039062 117.988281 15.621094 Z M 117.988281 15.621094 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 133.042969 25.121094 L 127.523438 25.121094 L 127.523438 24.820312 C 129.003906 24.738281 129.421875 24.320312 129.421875 23.21875 L 129.421875 14.238281 C 129.421875 13.558594 129.242188 13.261719 128.820312 13.261719 C 128.621094 13.261719 128.28125 13.359375 127.921875 13.5 L 127.382812 13.699219 L 127.382812 13.421875 L 130.960938 11.601562 L 131.140625 11.660156 L 131.140625 23.601562 C 131.140625 24.460938 131.542969 24.820312 133.042969 24.820312 Z M 133.042969 25.121094 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 143.042969 25.121094 L 137.523438 25.121094 L 137.523438 24.820312 C 139.003906 24.738281 139.421875 24.320312 139.421875 23.21875 L 139.421875 14.238281 C 139.421875 13.558594 139.242188 13.261719 138.820312 13.261719 C 138.621094 13.261719 138.28125 13.359375 137.921875 13.5 L 137.382812 13.699219 L 137.382812 13.421875 L 140.960938 11.601562 L 141.140625 11.660156 L 141.140625 23.601562 C 141.140625 24.460938 141.542969 24.820312 143.042969 24.820312 Z M 143.042969 25.121094 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 148.78125 24.261719 C 148.78125 24.839844 148.261719 25.339844 147.664062 25.339844 C 147.042969 25.339844 146.5625 24.859375 146.5625 24.238281 C 146.5625 23.621094 147.0625 23.121094 147.683594 23.121094 C 148.261719 23.121094 148.78125 23.660156 148.78125 24.261719 Z M 148.78125 24.261719 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 159.140625 12.199219 L 154.902344 25.28125 L 153.601562 25.28125 L 157.5625 13.359375 L 153.261719 13.359375 C 152.101562 13.359375 151.761719 13.640625 150.921875 15 L 150.5625 14.820312 L 151.761719 11.878906 L 159.140625 11.878906 Z M 159.140625 12.199219 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 179.222656 25.121094 L 173.640625 25.121094 L 173.640625 24.738281 C 175.203125 24.640625 175.402344 24.320312 175.402344 22.640625 L 175.402344 18.820312 L 169.34375 18.820312 L 169.34375 22.859375 C 169.34375 24.320312 169.601562 24.660156 171.082031 24.738281 L 171.101562 25.121094 L 165.523438 25.121094 L 165.523438 24.738281 C 167.082031 24.640625 167.300781 24.398438 167.300781 22.679688 L 167.300781 14.160156 C 167.300781 12.601562 167.0625 12.378906 165.523438 12.261719 L 165.523438 11.878906 L 171.121094 11.878906 L 171.121094 12.261719 C 169.664062 12.378906 169.34375 12.601562 169.34375 14.160156 L 169.34375 17.941406 L 175.402344 17.941406 L 175.402344 14.160156 C 175.402344 12.578125 175.140625 12.378906 173.621094 12.261719 L 173.621094 11.878906 L 179.222656 11.878906 L 179.222656 12.261719 C 177.742188 12.378906 177.441406 12.621094 177.441406 14.160156 L 177.441406 22.898438 C 177.441406 24.320312 177.722656 24.621094 179.222656 24.738281 Z M 179.222656 25.121094 "/>
<path fill-rule="nonzero" fill="rgb(0%, 0%, 0%)" fill-opacity="1" d="M 187.960938 22.398438 L 187.664062 25.121094 L 180.140625 25.121094 L 180.140625 24.820312 L 185.460938 16.71875 L 182.761719 16.71875 C 181.503906 16.71875 181.203125 17.019531 181.023438 18.480469 L 180.664062 18.480469 L 180.742188 16.121094 L 187.664062 16.121094 L 187.664062 16.421875 L 182.28125 24.519531 L 184.941406 24.519531 C 186.101562 24.519531 186.78125 24.320312 187.0625 23.980469 C 187.34375 23.640625 187.402344 23.320312 187.601562 22.320312 Z M 187.960938 22.398438 "/>
</svg>
"#;

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

    #[test]
    fn test_render_to_svg_with_metrics_success() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);

        // Create a valid JSON response with SVG and metrics
        let json_response = br#"{
            "svg": "<svg>test formula</svg>",
            "metrics": {
                "width": 100,
                "height": 50,
                "depth": 10,
                "ascent": 40
            }
        }"#;

        crate::shim::set_buffer(json_response);

        let m = MicroTex::new().expect("init ok");
        let r = m.render_to_svg_with_metrics("x^2", &RenderConfig::default());

        assert!(r.is_ok());
        let result = r.unwrap();
        assert!(result.svg.contains("<svg"));
        assert_eq!(result.metrics.width, 100);
        assert_eq!(result.metrics.height, 50);
        assert_eq!(result.metrics.depth, 10);
        assert_eq!(result.metrics.ascent, 40);
    }

    #[test]
    fn test_render_to_svg_with_metrics_parse_fail() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(false);

        let m = MicroTex::new().expect("init should succeed");
        let r = m.render_to_svg_with_metrics("x", &RenderConfig::default());

        assert!(matches!(r, Err(RenderError::ParseRenderFailed)));
        crate::shim::set_parse_succeed(true);
    }

    #[test]
    fn test_render_to_svg_with_metrics_empty_output() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(true);

        let m = MicroTex::new().expect("init should succeed");
        let r = m.render_to_svg_with_metrics("x", &RenderConfig::default());

        assert!(matches!(r, Err(RenderError::EmptyOutput)));
        crate::shim::set_return_empty(false);
    }

    #[test]
    fn test_render_to_svg_with_metrics_invalid_json() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);
        crate::shim::set_buffer(b"not valid json");

        let m = MicroTex::new().expect("init ok");
        let r = m.render_to_svg_with_metrics("x", &RenderConfig::default());

        assert!(matches!(r, Err(RenderError::ParseJsonFailed(_))));
    }

    #[test]
    fn test_render_to_svg_with_metrics_missing_svg() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);

        // JSON missing "svg" field
        let json_response = br#"{
            "metrics": {
                "width": 100,
                "height": 50,
                "depth": 10,
                "ascent": 40
            }
        }"#;

        crate::shim::set_buffer(json_response);

        let m = MicroTex::new().expect("init ok");
        let r = m.render_to_svg_with_metrics("x", &RenderConfig::default());

        assert!(matches!(r, Err(RenderError::ParseJsonFailed(_))));
    }

    #[test]
    fn test_render_to_svg_with_metrics_missing_metrics() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);

        // JSON missing "metrics" field
        let json_response = br#"{
            "svg": "<svg>test</svg>"
        }"#;

        crate::shim::set_buffer(json_response);

        let m = MicroTex::new().expect("init ok");
        let r = m.render_to_svg_with_metrics("x", &RenderConfig::default());

        assert!(matches!(r, Err(RenderError::ParseJsonFailed(_))));
    }

    #[test]
    fn test_render_to_svg_with_metrics_missing_width() {
        let _g = crate::shim::lock_test();
        crate::shim::set_init_succeed(true);
        crate::shim::set_parse_succeed(true);
        crate::shim::set_return_empty(false);

        // JSON with metrics missing "width" field
        let json_response = br#"{
            "svg": "<svg>test</svg>",
            "metrics": {
                "height": 50,
                "depth": 10,
                "ascent": 40
            }
        }"#;

        crate::shim::set_buffer(json_response);

        let m = MicroTex::new().expect("init ok");
        let r = m.render_to_svg_with_metrics("x", &RenderConfig::default());

        assert!(matches!(r, Err(RenderError::ParseJsonFailed(_))));
    }

    #[test]
    fn test_render_metrics_total_height() {
        let metrics = RenderMetrics::new(100, 50, 10, 40);
        assert_eq!(metrics.total_height(), 50.0);
    }

    #[test]
    fn test_render_metrics_aspect_ratio() {
        let metrics = RenderMetrics::new(200, 50, 10, 40);
        assert_eq!(metrics.aspect_ratio(), 4.0);
    }

    #[test]
    fn test_render_metrics_aspect_ratio_zero_height() {
        let metrics = RenderMetrics::new(100, 0, 0, 0);
        assert_eq!(metrics.aspect_ratio(), 1.0);
    }

    #[test]
    fn test_render_result_creation() {
        let metrics = RenderMetrics::new(100, 50, 10, 40);
        let result = RenderResult::new("<svg>test</svg>".to_string(), metrics);

        assert_eq!(result.svg, "<svg>test</svg>");
        assert_eq!(result.metrics.width, 100);
        assert_eq!(result.metrics.height, 50);
    }

    #[test]
    fn test_add_dpi_to_svg_simple() {
        let svg = r#"<svg width="100" height="50" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        let result = add_dpi_to_svg(svg, 720);
        assert!(result.contains(r#"data-dpi="720""#));
        assert!(result.contains(r#"width="100""#));
        assert!(result.contains(r#"height="50""#));
    }

    #[test]
    fn test_add_dpi_to_svg_with_namespace() {
        let svg =
            r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="120" height="60">"#;
        let result = add_dpi_to_svg(svg, 300);
        assert!(result.contains(r#"data-dpi="300""#));
        assert!(result.starts_with("<svg xmlns="));
    }

    #[test]
    fn test_add_dpi_to_svg_different_dpi_values() {
        let svg = r#"<svg viewBox="0 0 100 100">"#;
        let result_300 = add_dpi_to_svg(svg, 300);
        let result_720 = add_dpi_to_svg(svg, 720);

        assert!(result_300.contains(r#"data-dpi="300""#));
        assert!(result_720.contains(r#"data-dpi="720""#));
    }

    #[test]
    fn test_add_dpi_to_svg_no_svg_tag() {
        let svg = r#"<div>Not an SVG</div>"#;
        let result = add_dpi_to_svg(svg, 720);
        // Should return original string unchanged
        assert_eq!(result, svg);
    }

    #[test]
    fn test_add_dpi_to_svg_malformed() {
        let svg = r#"<svg no closing bracket here"#;
        let result = add_dpi_to_svg(svg, 720);
        // Should return original string unchanged
        assert_eq!(result, svg);
    }

    #[test]
    fn test_add_dpi_to_svg_preserves_content() {
        let svg = r#"<svg><circle cx="50" cy="50" r="40"/></svg>"#;
        let result = add_dpi_to_svg(svg, 720);
        assert!(result.contains(r#"<circle cx="50" cy="50" r="40"/></svg>"#));
        assert!(result.contains(r#"data-dpi="720""#));
    }

    #[test]
    fn test_extract_y_coordinates_simple() {
        let svg = r#"<svg><path d="M 10 20 L 30 40 Z"/></svg>"#;
        let y_coords = extract_y_coordinates(svg);
        assert!(y_coords.len() >= 2);
        assert!(y_coords.contains(&20.0));
        assert!(y_coords.contains(&40.0));
    }

    #[test]
    fn test_extract_y_coordinates_with_decimals() {
        let svg = r#"<svg><path d="M 10.5 20.25 L 30 39.121094 Z"/></svg>"#;
        let y_coords = extract_y_coordinates(svg);
        assert!(y_coords.contains(&20.25));
        // Check that max Y is approximately 39.121094
        let max_y = y_coords.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!((max_y - 39.121094).abs() < 0.001);
    }

    #[test]
    fn test_extract_y_coordinates_empty() {
        let svg = r#"<svg></svg>"#;
        let y_coords = extract_y_coordinates(svg);
        assert_eq!(y_coords.len(), 0);
    }

    #[test]
    fn test_extract_y_coordinates_multiple_paths() {
        let svg = r#"<svg>
            <path d="M 10 20 L 30 40 Z"/>
            <path d="M 5 15 L 25 35 Z"/>
        </svg>"#;
        let y_coords = extract_y_coordinates(svg);
        assert!(y_coords.len() >= 4);
        assert!(y_coords.contains(&20.0));
        assert!(y_coords.contains(&40.0));
        assert!(y_coords.contains(&15.0));
        assert!(y_coords.contains(&35.0));
    }

    #[test]
    fn test_adjust_svg_height_basic() {
        // Use single-line SVG to avoid text events
        let svg = r#"<svg width="100" height="50" viewBox="0 0 100 50"><path d="M 10 20 L 30 55.5 Z"/></svg>"#;
        let adjusted = adjust_svg_height_and_center(svg);
        println!("Original SVG:\n{}", svg);
        println!("Adjusted SVG:\n{}", adjusted);

        // Should contain updated height and viewBox
        assert!(adjusted.contains(r#"height="56""#), "Missing height=56");
        assert!(
            adjusted.contains(r#"viewBox="0 0 100 56""#),
            "Missing updated viewBox"
        );

        // Should contain <g> wrapper with translate
        assert!(
            adjusted.contains(r#"<g transform="translate(0, "#),
            "Missing <g> wrapper"
        );
        assert!(adjusted.contains("</g></svg>"), "Missing </g></svg>");
    }

    #[test]
    fn test_quick_xml_parsing() {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let svg = r#"<svg width="100"><path d="M 10 20"/></svg>"#;
        let mut reader = Reader::from_str(svg);
        let mut buf = Vec::new();
        let mut count = 0;

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => {
                    eprintln!("Event::Eof");
                    break;
                }
                Ok(Event::Start(e)) => {
                    let name_bytes = e.name();
                    let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("???");
                    eprintln!("Event::Start: {}", name);
                }
                Ok(Event::End(e)) => {
                    let name_bytes = e.name();
                    let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("???");
                    eprintln!("Event::End: {}", name);
                }
                Ok(_) => {
                    eprintln!("Other event");
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    break;
                }
            }
            count += 1;
            if count > 100 {
                eprintln!("Stopping after 100 iterations");
                break;
            }
        }
    }

    #[test]
    fn test_adjust_svg_height_within_tolerance() {
        // Test when max_y is truly within tolerance (< 0.02)
        let svg = r#"<svg width="100" height="50" viewBox="0 0 100 50">
<path d="M 10 20 L 30 0.01 Z"/>
</svg>"#;
        let adjusted = adjust_svg_height_and_center(svg);

        // Should not be modified (max_y = 20, ceil=20, no change needed since already in bounds)
        // Actually the test should check max_y < 0.02, which happens when all Y coords are near 0
        // Let's make a simpler test
        let svg2 = r#"<svg width="100" height="50" viewBox="0 0 100 50">
<path d="M 10 0 L 30 0.01 Z"/>
</svg>"#;
        let adjusted2 = adjust_svg_height_and_center(svg2);

        // max_y = 0.01, which is < 0.02, so no modification
        assert_eq!(adjusted2, svg2);
    }

    #[test]
    fn test_adjust_svg_height_complex() {
        let svg = r#"<svg width="188" height="39" viewBox="0 0 188 39">
<path d="M 10.480469 23.28125 L 6.621094 14.480469 L 2.71875 23.28125 Z"/>
<path d="M 61.191406 34.5 L 61.191406 27.640625 L 56.390625 34.5 Z M 64.8125 35.78125 L 62.75 35.78125 L 62.75 39.121094 L 61.191406 39.121094"/>
</svg>"#;
        let adjusted = adjust_svg_height_and_center(svg);

        // Should have updated height to 40 (ceil of 39.121094)
        assert!(adjusted.contains(r#"height="40""#));
        assert!(adjusted.contains(r#"viewBox="0 0 188 40""#));
        assert!(adjusted.contains(r#"<g transform="translate(0, "#));
    }

    #[test]
    fn test_extract_complexe_svg() {
        let svg = COMPLEXE_SVG;
        let y_coords = extract_y_coordinates(svg);
        assert!(y_coords.len() >= 20);
        // max cannot be > 40.0
        let max_y = y_coords.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        println!("Max Y coordinate: {}", max_y);
        assert!(max_y <= 40.0);
    }

    #[test]
    fn test_transformation_complete() {
        let svg = COMPLEXE_SVG;
        let transformed_svg = adjust_svg_height_and_center(svg);
        // new SVG height should be older height +1
        fn extract_height(s: &str) -> Option<f32> {
            if let Some(idx) = s.find(r#"height=""#) {
                let rest = &s[idx + 8..];
                if let Some(end) = rest.find('"') {
                    return rest[..end].parse::<f32>().ok();
                }
            }
            None
        }
        // Extract <g> translate parameters in transform attribute.
        fn extract_translate_y(s: &str) -> Option<f32> {
            // Search for translate(...)
            if let Some(idx) = s.find(r#"transform="translate("#) {
                let rest = &s[idx + r#"transform="translate("#.len()..];
                if let Some(end) = rest.find(')') {
                    let inside = &rest[..end];
                    // Split by comma or whitespace and keep non-empty parts
                    let parts: Vec<&str> = inside
                        .split(|c: char| c == ',' || c.is_whitespace())
                        .filter(|p| !p.is_empty())
                        .collect();

                    // translate(x, y) -> parts[0]=x, parts[1]=y
                    if parts.len() >= 2 {
                        return parts[1].trim().parse::<f32>().ok();
                    }
                }
            }
            None
        }
        let original_height = extract_height(svg).expect("original SVG should contain height");
        let transformed_height =
            extract_height(&transformed_svg).expect("transformed SVG should contain height");
        println!(
            "Original height: {}, Transformed height: {}",
            original_height, transformed_height
        );
        assert!((transformed_height - (original_height + 1.0)).abs() < f32::EPSILON);
        let translate_y = extract_translate_y(&transformed_svg).unwrap();
        println!("Translate Y: {}", translate_y);
        // Validate translate Y is small (less than 0.5)
        assert!(translate_y < 0.5);
    }
}
