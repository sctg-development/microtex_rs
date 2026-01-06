//! Modular build script for microtex_rs
//!
//! This script:
//! 1. Compiles the C++ MicroTeX library (static) using CMake
//! 2. Generates FFI bindings using bindgen
//! 3. Embeds CLM font data as Rust code
//! 4. Links the compiled library to the Rust crate

use std::path::Path;

mod build_config {
    use std::path::PathBuf;

    /// Get the workspace root directory
    pub fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// Get the C++ directory path
    pub fn cpp_dir() -> PathBuf {
        workspace_root().join("c++")
    }

    /// Get the build directory for CMake
    pub fn build_dir() -> PathBuf {
        cpp_dir().join("build")
    }

    /// Get the output directory for build artifacts
    pub fn out_dir() -> PathBuf {
        PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"))
    }

    /// Get the resources directory containing font files
    pub fn res_dir() -> PathBuf {
        cpp_dir().join("res")
    }
}

mod cmake_builder {
    use std::path::Path;
    use std::process::Command;

    /// Run CMake to configure and build the C++ MicroTeX library
    pub fn build(cpp_dir: &Path, build_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create build directory if it doesn't exist
        std::fs::create_dir_all(build_dir)?;

        println!("cargo:warning=Building MicroTeX C++ library...");
        println!("cargo:warning=CMake directory: {}", cpp_dir.display());
        println!("cargo:warning=Build directory: {}", build_dir.display());

        // Configure with CMake
        let status = Command::new("cmake")
            .arg("-DCAIRO=ON")
            .arg("-DHAVE_CAIRO=ON")
            .arg("-DBUILD_STATIC=ON")
            .arg("-DHAVE_CWRAPPER=ON")
            .current_dir(build_dir)
            .arg("..")
            .status()
            .map_err(|e| format!("Failed to run cmake configure: {}", e))?;

        if !status.success() {
            return Err("CMake configuration failed".into());
        }

        // Build with make
        let num_jobs = num_cpus::get();
        let status = Command::new("make")
            .arg(format!("-j{}", num_jobs))
            .current_dir(build_dir)
            .status()
            .map_err(|e| format!("Failed to run make: {}", e))?;

        if !status.success() {
            return Err("CMake build failed".into());
        }

        println!("cargo:warning=MicroTeX C++ library built successfully!");
        Ok(())
    }

    /// Find all static libraries (.a files) in the build output
    pub fn find_static_libraries(build_dir: &Path) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
        let mut libs = Vec::new();
        
        // Walk the build directory looking for .a files
        for entry in walkdir::WalkDir::new(build_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "a") {
                println!("cargo:warning=Found static library: {}", path.display());
                libs.push(path.to_path_buf());
            }
        }

        Ok(libs)
    }
}

mod bindgen_builder {
    use std::path::{Path, PathBuf};
    use std::fs::File;
    use std::io::Write;

    /// Generate FFI bindings using bindgen
    pub fn generate_bindings(
        _cpp_include: &Path,
        out_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        println!("cargo:warning=Generating FFI bindings...");

        // Use a simplified C-only wrapper to avoid C++ includes
        let wrapper_content = r#"
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void* FontMetaPtr;
typedef void* RenderPtr;
typedef void* DrawingData;

// Core functions
const char* microtex_version(void);
void microtex_release(void);
bool microtex_isInited(void);

// Font functions
FontMetaPtr microtex_init(unsigned long len, const unsigned char* data);
FontMetaPtr microtex_addFont(unsigned long len, const unsigned char* data);
void microtex_releaseFontMeta(FontMetaPtr ptr);
const char* microtex_getFontFamily(FontMetaPtr ptr);
const char* microtex_getFontName(FontMetaPtr ptr);
bool microtex_isMathFont(FontMetaPtr ptr);

// Configuration functions
void microtex_setDefaultMathFont(const char* name);
void microtex_setDefaultMainFont(const char* name);
bool microtex_hasGlyphPathRender(void);
void microtex_setRenderGlyphUsePath(bool use);
bool microtex_isRenderGlyphUsePath(void);

// Rendering functions
RenderPtr microtex_parseRender(
    const char* tex,
    int width,
    float textSize,
    float lineSpace,
    unsigned int color,
    bool fillWidth,
    bool enableOverrideTeXStyle,
    unsigned int texStyle
);
void microtex_deleteRender(RenderPtr render);
DrawingData microtex_getDrawingData(RenderPtr render);
void microtex_freeDrawingData(DrawingData data);
unsigned char* microtex_render_to_svg(RenderPtr render, unsigned long* len);
void microtex_free_buffer(void* ptr);

#ifdef __cplusplus
}
#endif
"#;

        let bindings = bindgen::Builder::default()
            .header_contents("microtex_wrapper.h", wrapper_content)
            .use_core()
            .ctypes_prefix("std::os::raw")
            .generate()
            .map_err(|_| "bindgen failed to generate bindings")?;

        let out_path = out_dir.join("bindings.rs");
        bindings.write_to_file(&out_path)?;

        println!("cargo:warning=Bindings generated: {}", out_path.display());
        Ok(out_path)
    }
}

mod fonts_embedder {
    use std::path::{Path, PathBuf};
    use std::fs::File;
    use std::io::Write;

    /// Embed CLM font files as Rust code
    pub fn embed_fonts(res_dir: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("cargo:warning=Embedding CLM fonts...");

        let mut rust_code = String::new();
        rust_code.push_str("// Auto-generated font embedding code\n\n");

        let mut fonts_found = 0;
        let mut fonts_list = Vec::new();

        // Collect all font files from different font family directories
        for font_family in &["firamath", "lm-math", "tex-gyre", "xits"] {
            let font_dir = res_dir.join(font_family);

            if !font_dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&font_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "clm2") {
                        let file_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        if !file_name.is_empty() {
                            fonts_list.push((file_name, path));
                            fonts_found += 1;
                        }
                    }
                }
            }
        }

        // Generate the available_embedded_clms() function
        rust_code.push_str("/// List of available embedded CLM fonts\n");
        rust_code.push_str("pub fn available_embedded_clms() -> Vec<&'static str> {\n");
        rust_code.push_str("    vec![\n");

        for (file_name, _) in &fonts_list {
            rust_code.push_str(&format!("        \"{}\",\n", file_name));
        }

        rust_code.push_str("    ]\n");
        rust_code.push_str("}\n\n");

        // First, generate all the static font data constants BEFORE the match function
        for (file_name, path) in &fonts_list {
            let const_name = file_name
                .to_uppercase()
                .replace(".", "_")
                .replace("-", "_");

            let font_data = std::fs::read(path)?;
            rust_code.push_str(&format!(
                "// Font: {} ({} bytes)\n",
                file_name,
                font_data.len()
            ));
            rust_code.push_str(&format!("const {}: &[u8] = &[\n", const_name));

            // Write font data in hex format, 16 bytes per line
            for chunk in font_data.chunks(16) {
                rust_code.push_str("    ");
                for byte in chunk {
                    rust_code.push_str(&format!("{:#04x}, ", byte));
                }
                rust_code.push_str("\n");
            }

            rust_code.push_str("];\n\n");
        }

        // Generate the get_embedded_clm() function
        rust_code.push_str("/// Retrieve embedded CLM font data by filename\n");
        rust_code.push_str("pub fn get_embedded_clm(name: &str) -> Option<&'static [u8]> {\n");
        rust_code.push_str("    match name {\n");

        for (file_name, _) in &fonts_list {
            let const_name = file_name
                .to_uppercase()
                .replace(".", "_")
                .replace("-", "_");

            rust_code.push_str(&format!("        \"{}\" => Some(&{}),\n", file_name, const_name));
        }

        rust_code.push_str("        _ => None,\n");
        rust_code.push_str("    }\n");
        rust_code.push_str("}\n");

        if fonts_found == 0 {
            eprintln!("Warning: No CLM fonts found in {}", res_dir.display());
        }

        let out_path = out_dir.join("embedded_clms.rs");
        let mut file = File::create(&out_path)?;
        file.write_all(rust_code.as_bytes())?;

        println!("cargo:warning=Embedded {} fonts", fonts_found);
        println!("cargo:warning=Fonts module generated: {}", out_path.display());
        Ok(())
    }
}

mod linker_config {
    use std::path::Path;
    use std::process::Command;

    /// Configure the linker to link against the compiled C++ library
    pub fn configure(build_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("cargo:warning=Configuring linker...");

        // Add the build/lib directory to the linker search path
        let lib_dir = build_dir.join("lib");
        if lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
        }

        // Add the build/platform/cairo directory to the linker search path
        let cairo_lib_dir = build_dir.join("platform/cairo");
        if cairo_lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", cairo_lib_dir.display());
        }

        // Link against the static microtex library
        println!("cargo:rustc-link-lib=static=microtex");

        // Link against the microtex-cairo library (contains Cairo rendering code)
        println!("cargo:rustc-link-lib=static=microtex-cairo");

        // Link C++ standard library
        #[cfg(target_os = "macos")]
        {
            println!("cargo:rustc-link-lib=dylib=c++");
        }

        #[cfg(target_os = "linux")]
        {
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }

        #[cfg(target_os = "windows")]
        {
            println!("cargo:rustc-link-lib=dylib=msvcrt");
        }

        // Use pkg-config to find and link all Cairo and Pango dependencies
        let mut cairo_libs = Vec::new();
        let mut search_paths = Vec::new();

        // Get Cairo flags from pkg-config
        if let Ok(output) = Command::new("pkg-config")
            .arg("--exists")
            .arg("cairo")
            .status()
        {
            if output.success() {
                println!("cargo:warning=Found cairo via pkg-config");
                
                if let Ok(libs_output) = Command::new("pkg-config")
                    .arg("--libs")
                    .arg("--static")
                    .arg("cairo")
                    .arg("pango")
                    .arg("pangocairo")
                    .output()
                {
                    let libs_str = String::from_utf8_lossy(&libs_output.stdout);
                    println!("cargo:warning=Cairo/Pango linker flags: {}", libs_str.trim());
                    
                    for flag in libs_str.split_whitespace() {
                        if flag.starts_with("-l") {
                            let lib_name = &flag[2..];
                            if !cairo_libs.contains(&lib_name.to_string()) {
                                cairo_libs.push(lib_name.to_string());
                                println!("cargo:rustc-link-lib={}", lib_name);
                            }
                        } else if flag.starts_with("-L") {
                            let lib_path = &flag[2..];
                            if !search_paths.contains(&lib_path.to_string()) {
                                search_paths.push(lib_path.to_string());
                                println!("cargo:rustc-link-search=native={}", lib_path);
                            }
                        }
                    }
                } else {
                    // Fallback: try to get libs without --static flag
                    if let Ok(libs_output) = Command::new("pkg-config")
                        .arg("--libs")
                        .arg("cairo")
                        .output()
                    {
                        let libs_str = String::from_utf8_lossy(&libs_output.stdout);
                        println!("cargo:warning=Cairo linker flags: {}", libs_str.trim());
                        
                        for flag in libs_str.split_whitespace() {
                            if flag.starts_with("-l") {
                                let lib_name = &flag[2..];
                                if !cairo_libs.contains(&lib_name.to_string()) {
                                    println!("cargo:rustc-link-lib={}", lib_name);
                                }
                            } else if flag.starts_with("-L") {
                                let lib_path = &flag[2..];
                                if !search_paths.contains(&lib_path.to_string()) {
                                    println!("cargo:rustc-link-search=native={}", lib_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("cargo:warning=Linker configured successfully!");
        Ok(())
    }
}

fn main() {
    println!("cargo:rerun-if-changed=c++/");
    println!("cargo:rerun-if-changed=build.rs");

    // Step 1: Build the C++ library with CMake
    let cpp_dir = build_config::cpp_dir();
    let build_dir = build_config::build_dir();
    let out_dir = build_config::out_dir();

    if let Err(e) = cmake_builder::build(&cpp_dir, &build_dir) {
        eprintln!("Error building C++ library: {}", e);
        std::process::exit(1);
    }

    // Step 2: Find the compiled static libraries
    match cmake_builder::find_static_libraries(&build_dir) {
        Ok(libs) => {
            if libs.is_empty() {
                eprintln!("Warning: No static libraries found in build directory");
            }
        }
        Err(e) => {
            eprintln!("Error finding static libraries: {}", e);
            std::process::exit(1);
        }
    }

    // Step 3: Generate FFI bindings
    let cpp_include = cpp_dir.join("lib");
    if let Err(e) = bindgen_builder::generate_bindings(&cpp_include, &out_dir) {
        eprintln!("Error generating bindings: {}", e);
        std::process::exit(1);
    }

    // Step 4: Embed CLM fonts
    let res_dir = build_config::res_dir();
    if let Err(e) = fonts_embedder::embed_fonts(&res_dir, &out_dir) {
        eprintln!("Error embedding fonts: {}", e);
        std::process::exit(1);
    }

    // Step 5: Configure the linker
    if let Err(e) = linker_config::configure(&build_dir) {
        eprintln!("Error configuring linker: {}", e);
        std::process::exit(1);
    }

    println!("cargo:warning=Build script completed successfully!");
}

