use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn collect_clm_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read res dir") {
        let e = entry.expect("entry");
        let p = e.path();
        if p.is_dir() {
            collect_clm_files(&p, out);
        } else if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
            if ext.starts_with("clm") || ext == "clm" {
                out.push(p);
            }
        }
    }
}

fn run_cmd(cmd: &mut std::process::Command) {
    eprintln!("running: {:?}", cmd);
    let output = cmd.output().expect("failed to spawn command");
    if !output.status.success() {
        eprintln!("=== COMMAND STDOUT ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("=== COMMAND STDERR ===");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        panic!("command failed: {:?}", output.status);
    }
}

/// Download a tarball by url to `dst` using curl or wget.
fn download_to(url: &str, dst: &Path) {
    if dst.exists() {
        // Verify file is valid (at least a minimal tarball, not an error page)
        let metadata = match std::fs::metadata(dst) {
            Ok(m) => m,
            Err(_) => {
                std::fs::remove_file(dst).ok();
                return;
            }
        };
        // If file is suspiciously small (< 1KB), it's probably an error page; re-download
        if metadata.len() < 1024 {
            eprintln!("Warning: {} is suspiciously small ({}B), re-downloading", dst.display(), metadata.len());
            std::fs::remove_file(dst).ok();
        } else {
            return;
        }
    }
    eprintln!("Downloading {} to {}", url, dst.display());
    let downloaded = if std::process::Command::new("curl")
        .arg("-L")
        .arg("--fail")  // Fail on HTTP errors
        .arg("-o")
        .arg(dst)
        .arg(url)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        true
    } else {
        std::process::Command::new("wget")
            .arg(url)
            .arg("-O")
            .arg(dst)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    if !downloaded {
        std::fs::remove_file(dst).ok();
        panic!("failed to download {}. Please install curl or wget and check internet connection.", url);
    }
}

/// Extract a tarball to dest, handles .tar.gz and .tar.xz
fn extract_tarball(tarball: &Path, dest: &Path) {
    if dest.exists() {
        return;
    }
    let _ = std::fs::create_dir_all(dest);
    let file = tarball.to_string_lossy();

    if file.ends_with(".tar.xz") {
        // Use xz piped to tar for better compatibility (especially on macOS)
        eprintln!("Extracting {} using xz pipe", tarball.display());
        let xz_child = std::process::Command::new("xz")
            .arg("-dc")
            .arg(tarball)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("failed to spawn xz");

        let tar_status = std::process::Command::new("tar")
            .arg("-xf")
            .arg("-")
            .arg("-C")
            .arg(dest)
            .arg("--strip-components=1")
            .stdin(xz_child.stdout.expect("failed to get xz stdout"))
            .status()
            .expect("failed to run tar");

        if !tar_status.success() {
            panic!("tar extraction failed for {}", tarball.display());
        }
    } else {
        run_cmd(
            std::process::Command::new("tar")
                .arg("-xzf")
                .arg(tarball)
                .arg("-C")
                .arg(dest)
                .arg("--strip-components=1"),
        );
    }
}

/// Build a meson-based project located in `src_dir` and install into `install_dir`.
fn meson_build_and_install(src_dir: &Path, install_dir: &Path, meson_args: &[&str]) {
    let build_dir = src_dir.join("build");
    let _ = std::fs::create_dir_all(&build_dir);

    // Ensure meson/ninja are available, try to auto-bootstrap if needed
    match ensure_meson_and_ninja() {
        Ok(_) => {}
        Err(e) => panic!("Tool bootstrap failed: {}", e),
    }

    // Detect target and prepare architecture-specific flags
    let target = env::var("TARGET").unwrap_or_default();
    let mut c_args = String::from("-mmacosx-version-min=11.0");
    let mut cpp_args = String::from("-mmacosx-version-min=11.0");
    let mut ld_args = String::from("-mmacosx-version-min=11.0");
    
    if target.contains("apple") {
        // If cross-compiling on macOS (e.g., arm64 runner to x86_64 target),
        // specify the architecture explicitly
        if target.contains("x86_64") {
            c_args.push_str(" -arch x86_64");
            cpp_args.push_str(" -arch x86_64");
            ld_args.push_str(" -arch x86_64");
            eprintln!("Cross-compiling for x86_64-apple-darwin; added -arch x86_64 flags");
        } else if target.contains("aarch64") {
            c_args.push_str(" -arch arm64");
            cpp_args.push_str(" -arch arm64");
            ld_args.push_str(" -arch arm64");
            eprintln!("Compiling for aarch64-apple-darwin; added -arch arm64 flags");
        }
    } else if target.contains("musl") {
        // musl compilation: ensure compatibility
        c_args = String::from("-fPIC");
        cpp_args = String::from("-fPIC");
        ld_args = String::from("-static-libgcc");
    }

    // Try running meson setup, but be resilient to unknown -D options
    // Some Cairo releases expose different meson options; if meson reports
    // "Unknown option: \"foo\"" we remove the offending -Dfoo option and retry.
    let mut args: Vec<String> = meson_args.iter().map(|s| s.to_string()).collect();
    
    // Add architecture/platform flags as Meson options
    if !c_args.is_empty() {
        args.push(format!("-Dc_args={}", c_args));
    }
    if !cpp_args.is_empty() {
        args.push(format!("-Dcpp_args={}", cpp_args));
    }
    if !ld_args.is_empty() {
        args.push(format!("-Dcpp_link_args={}", ld_args));
        args.push(format!("-Dc_link_args={}", ld_args));
    }

    for attempt in 0..4 {
        let mut cmd_try = std::process::Command::new("meson");
        cmd_try.arg("setup").arg(&build_dir).arg(src_dir).arg(format!("--prefix={}", install_dir.display()));
        for a in args.iter() {
            cmd_try.arg(a);
        }

        eprintln!("Attempt {}: running meson with args: {:?}", attempt + 1, &args);
        let output = cmd_try.output().expect("failed to spawn meson");
        if output.status.success() {
            break;
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("meson failed: {}", stderr);
        // Look for Unknown option messages and remove offending -D flags
        let mut removed_any = false;
        for line in stderr.lines() {
            if line.contains("ERROR: Unknown option:") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let opt = &line[start + 1..start + 1 + end];
                        // Remove any -D<opt> or -D<opt>=... entries from args
                        let before = args.len();
                        args.retain(|a| {
                            if a == &format!("-D{}", opt) {
                                false
                            } else if a.starts_with(&format!("-D{}=", opt)) {
                                false
                            } else {
                                true
                            }
                        });
                        if args.len() < before {
                            eprintln!("Removed unsupported meson option: {}", opt);
                            removed_any = true;
                        }
                    }
                }
            }
        }

        // If Meson didn't emit the unknown-option error on stderr, check meson-log.txt
        if !removed_any {
            let meson_log = build_dir.join("meson-logs").join("meson-log.txt");
            if meson_log.exists() {
                if let Ok(contents) = std::fs::read_to_string(&meson_log) {
                    for line in contents.lines() {
                        if line.contains("ERROR: Unknown option:") {
                            if let Some(start) = line.find('"') {
                                if let Some(end) = line[start + 1..].find('"') {
                                    let opt = &line[start + 1..start + 1 + end];
                                    let before = args.len();
                                    args.retain(|a| {
                                        if a == &format!("-D{}", opt) {
                                            false
                                        } else if a.starts_with(&format!("-D{}=", opt)) {
                                            false
                                        } else {
                                            true
                                        }
                                    });
                                    if args.len() < before {
                                        eprintln!("Removed unsupported meson option from meson-log: {}", opt);
                                        removed_any = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !removed_any {
            // No unknown option detected, collect extra debug info and abort
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("Meson stdout:\n{}", stdout);
            eprintln!("Meson stderr:\n{}", stderr);

            // Print environment and tool versions that can affect Meson
            if let Ok(path) = env::var("PATH") {
                eprintln!("PATH={}", path);
            }
            if let Ok(pkg) = env::var("PKG_CONFIG_PATH") {
                eprintln!("PKG_CONFIG_PATH={}", pkg);
            }
            let _ = std::process::Command::new("which")
                .arg("meson")
                .status()
                .map(|s| eprintln!("which meson exit: {:?}", s));
            let _ = std::process::Command::new("meson")
                .arg("--version")
                .status()
                .map(|s| eprintln!("meson --version exit: {:?}", s));
            let _ = std::process::Command::new("ninja")
                .arg("--version")
                .status()
                .map(|s| eprintln!("ninja --version exit: {:?}", s));
            let _ = std::process::Command::new("python3")
                .arg("--version")
                .status()
                .map(|s| eprintln!("python3 --version exit: {:?}", s));
            let _ = std::process::Command::new("pipx")
                .arg("--version")
                .status()
                .map(|s| eprintln!("pipx --version exit: {:?}", s));

            // If Meson created a meson-log, print it for more detail
            let meson_log = build_dir.join("meson-logs").join("meson-log.txt");
            if meson_log.exists() {
                if let Ok(contents) = std::fs::read_to_string(&meson_log) {
                    eprintln!("meson log ({}):\n{}", meson_log.display(), contents);
                } else {
                    eprintln!("meson log exists but could not be read: {}", meson_log.display());
                }
            } else {
                eprintln!("meson log not found at expected path: {}", meson_log.display());
            }

            panic!("Meson setup failed and no unknown options were found. See CI log for meson stdout/stderr and meson log file.");
        }
        // otherwise retry with pruned args
    }

    run_cmd(
        std::process::Command::new("ninja")
            .arg("-C")
            .arg(&build_dir),
    );
    run_cmd(
        std::process::Command::new("ninja")
            .arg("-C")
            .arg(&build_dir)
            .arg("install"),
    );

    // Verify installation succeeded by checking for library files
    let lib_dir = install_dir.join("lib");
    if !lib_dir.exists() {
        panic!(
            "Installation failed: lib directory not found at {}. Meson/Ninja did not complete successfully.",
            lib_dir.display()
        );
    }
}

/// Vendoring removed on `main`. To experiment with vendoring, see the `vendored` branch.
/// This stub returns `false` so callers fall back to system libraries or dependency bundles.
fn vendor_core_deps(_out_dir: &Path) -> bool {
    println!("cargo:warning=Vendoring support removed from main branch. Use dependency bundles (scripts/create_bundle_macos.sh) or the 'vendored' branch for full vendored builds.");
    false
}

/// Vendor and build Pango + GLib dependencies (libffi, fribidi, glib, pango)
use std::process::Command;

fn ensure_meson_and_ninja() -> Result<(), String> {
    // quick check
    let meson_ok = Command::new("meson")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let ninja_ok = Command::new("ninja")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if meson_ok && ninja_ok {
        return Ok(());
    }

    // attempt to add user's site-packages bin to PATH
    if let Ok(output) = Command::new("python3")
        .arg("-m")
        .arg("site")
        .arg("--user-base")
        .output()
    {
        if output.status.success() {
            let base = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let bin = format!("{}/bin", base);
            let prev = env::var("PATH").unwrap_or_default();
            if !prev.split(':').any(|p| p == bin) {
                let new = format!("{}:{}", bin, prev);
                env::set_var("PATH", new);
            }
        }
    }

    // helper to try pip install --user pkg
    let try_pip = |pkg: &str| -> bool {
        eprintln!("Trying python3 -m pip install --user {}", pkg);
        Command::new("python3")
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("--user")
            .arg(pkg)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    // helper to try pipx install
    let try_pipx = |pkg: &str| -> bool {
        if Command::new("pipx").arg("--version").status().is_err() {
            return false;
        }
        eprintln!("Trying pipx install {}", pkg);
        Command::new("pipx")
            .arg("install")
            .arg(pkg)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    // helper to try brew install (macOS)
    let try_brew = |pkg: &str| -> bool {
        if Command::new("brew").arg("--version").status().is_err() {
            return false;
        }
        eprintln!("Trying brew install {}", pkg);
        Command::new("brew")
            .arg("install")
            .arg(pkg)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    // For meson
    if !meson_ok {
        if try_pip("meson") || try_pipx("meson") || try_brew("meson") {
            let recheck = Command::new("meson")
                .arg("--version")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if recheck {
                eprintln!("meson bootstrapped via pip/pipx/brew");
            } else {
                eprintln!("meson not found after install attempts");
            }
        }
    }

    // For ninja
    if !ninja_ok {
        if try_pip("ninja") || try_pipx("ninja") || try_brew("ninja") {
            let recheck = Command::new("ninja")
                .arg("--version")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if recheck {
                eprintln!("ninja bootstrapped via pip/pipx/brew");
            } else {
                eprintln!("ninja not found after install attempts");
            }
        }
    }

    // Final check
    let meson_ok = Command::new("meson")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let ninja_ok = Command::new("ninja")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if meson_ok && ninja_ok {
        Ok(())
    } else {
        Err("meson or ninja missing and bootstrap failed".to_string())
    }
}

fn vendor_pango_deps(_out_dir: &Path) -> bool {
    println!("cargo:warning=Vendoring of Pango/GLib removed from main branch. Use dependency bundles or system packages via Homebrew.");
    false
}

fn main() {
    // make sure builds rerun when user changes bundle env var
    println!("cargo:rerun-if-env-changed=MICROTEX_BUNDLE_DIR");

    // Optionally build a vendored Cairo and add its pkgconfig path so the CMake
    // step finds it. Enable with feature `vendored-cairo` or env var
    // `MICROTEX_VENDORED_CAIRO=1`. Use `MICROTEX_USE_SYSTEM_CAIRO=1` to prefer
    // system libraries and skip vendoring.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let prefer_system = env::var("MICROTEX_USE_SYSTEM_CAIRO").is_ok();
    let target = env::var("TARGET").unwrap_or_default();

    // Vendoring of Cairo removed from main branch. Use system libraries or dependency bundles
    // (see `scripts/create_bundle_macos.sh` and BUILDING.md for instructions).

    // Detect dependency bundle and configure PKG_CONFIG_PATH / link search if present
    let bundle_dir = env::var("MICROTEX_BUNDLE_DIR").map(|s| PathBuf::from(s)).unwrap_or_else(|_| {
        let mut p = PathBuf::from("dependencies_bundle");
        if target.contains("apple") && target.contains("x86_64") {
            p = p.join("macos").join("intel");
        } else if target.contains("apple") && target.contains("aarch64") {
            p = p.join("macos").join("arm64");
        } else if target.contains("linux") && target.contains("x86_64") {
            p = p.join("ubuntu").join("amd64");
        } else if target.contains("linux") && target.contains("aarch64") {
            p = p.join("ubuntu").join("arm64");
        }
        p
    });

    let mut using_bundle = false;
    if bundle_dir.exists() {
        let pkgconfig_path = bundle_dir.join("lib").join("pkgconfig");
        if pkgconfig_path.exists() {
            let prev = env::var("PKG_CONFIG_PATH").unwrap_or_default();
            let new = if prev.is_empty() {
                format!("{}", pkgconfig_path.display())
            } else {
                format!("{}:{}", pkgconfig_path.display(), prev)
            };
            // Ensure pkg-config searches the bundle first
            env::set_var("PKG_CONFIG_PATH", new.clone());
            // Also set PKG_CONFIG_LIBDIR to prefer the bundle .pc files while preserving any
            // existing PKG_CONFIG_LIBDIR so missing system .pc files remain discoverable.
            // Only set PKG_CONFIG_LIBDIR if it already exists so we don't hide system pkg-config
            // search paths (when unset, pkg-config falls back to system dirs). This avoids making
            // bundled .pc files the sole source of pkg-config info which can accidentally hide
            // system-provided .pc files like zlib or expat.
            if let Ok(prev_libdir) = env::var("PKG_CONFIG_LIBDIR") {
                let new_libdir = format!("{}:{}", pkgconfig_path.display(), prev_libdir);
                env::set_var("PKG_CONFIG_LIBDIR", new_libdir);
            }
            println!("cargo:warning=Using dependency bundle at {}", bundle_dir.display());
        }
    }

    // Ensure pkg-config is present (CMake uses it to find system libraries)
    if std::process::Command::new("pkg-config")
        .arg("--version")
        .status()
        .is_err()
    {
        panic!("pkg-config not found on PATH. Install it (e.g. `brew install pkg-config`) or ensure it is available. If you want a fully vendored build, we can extend to build Pango / Fontconfig too — open an issue if you want that.");
    }

    // If not using vendored cairo, verify required pkg-config libraries are present.
    let mut using_vendored = env::var("CARGO_FEATURE_VENDORED_CAIRO").is_ok()
        || env::var("MICROTEX_VENDORED_CAIRO").is_ok();

    // Check if the user explicitly asked to vendor pango/glib
    let vendored_pango_feature = env::var("CARGO_FEATURE_VENDORED_PANGO").is_ok();
    let vendored_pango_env = env::var("MICROTEX_VENDORED_PANGO").is_ok();

    // Verify required packages (either provided by system pkg-config or by dependency bundle).
    let required = ["cairo", "pango", "pangocairo", "fontconfig"];
    let mut missing = Vec::new();
    for pkg in required.iter() {
        let ok = std::process::Command::new("pkg-config")
            .arg("--exists")
            .arg(pkg)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            missing.push(*pkg);
        }
    }

    if !missing.is_empty() {
        if using_bundle {
            println!("cargo:warning=Dependency bundle present but missing packages: {:?}. Ensure bundle has .pc files under lib/pkgconfig.", missing);
        } else if prefer_system {
            panic!("Missing system packages: {:?}. Install with Homebrew: brew install cairo pango fontconfig pkg-config lzo", missing);
        } else {
            panic!("Missing packages: {:?}. Install via Homebrew or create a dependency bundle and set MICROTEX_BUNDLE_DIR.", missing);
        }
    }

    // For packages found via pkg-config, add their library search paths so rustc's linker finds the system libraries.
    for pkg in ["cairo", "pango", "pangocairo", "fontconfig"].iter() {
        // run `pkg-config --libs-only-L pkg`
        let out = std::process::Command::new("pkg-config")
            .arg("--libs-only-L")
            .arg(pkg)
            .output();
        if let Ok(o) = out {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout);
                for token in s.split_whitespace() {
                    if token.starts_with("-L") {
                        let dir = &token[2..];
                        println!("cargo:rustc-link-search=native={}", dir);
                    }
                }
            }
        }
    }

    // Build MicroTeX C++ library using cmake with HAVE_CWRAPPER enabled
    let mut cmake_config = cmake::Config::new("./c++");
    
    // Ensure CMake inherits PKG_CONFIG_PATH and PKG_CONFIG_LIBDIR for dependency bundle discovery
    if let Ok(pkg_config_path) = env::var("PKG_CONFIG_PATH") {
        eprintln!("Passing PKG_CONFIG_PATH to CMake: {}", pkg_config_path);
        cmake_config.env("PKG_CONFIG_PATH", &pkg_config_path);
    }
    if let Ok(pkg_config_libdir) = env::var("PKG_CONFIG_LIBDIR") {
        eprintln!("Passing PKG_CONFIG_LIBDIR to CMake: {}", pkg_config_libdir);
        cmake_config.env("PKG_CONFIG_LIBDIR", &pkg_config_libdir);
    }
    
    cmake_config
        .define("HAVE_CWRAPPER", "ON")
        .define("BUILD_STATIC", "ON")
        .define("CAIRO", "ON")
        .profile("Release")
        .build_target("microtex");

    // Set macOS deployment target to match the Rust target
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("apple") {
        // Use a reasonable macOS minimum deployment target (11.0 for M1+ support)
        cmake_config.define("CMAKE_OSX_DEPLOYMENT_TARGET", "11.0");
        
        // If cross-compiling on macOS (e.g., arm64 runner to x86_64 target),
        // specify the architecture explicitly
        if target.contains("x86_64") {
            cmake_config.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
            eprintln!("Cross-compiling for x86_64-apple-darwin; set CMAKE_OSX_ARCHITECTURES");
        } else if target.contains("aarch64") {
            cmake_config.define("CMAKE_OSX_ARCHITECTURES", "arm64");
            eprintln!("Compiling for aarch64-apple-darwin; set CMAKE_OSX_ARCHITECTURES");
        }
    }

    let dst = cmake_config.build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    // If CMake placed the static library deeper (e.g. in build/lib), find it and add that dir too
    fn find_lib_dir(start: &Path) -> Option<PathBuf> {
        let mut stack = vec![start.to_path_buf()];
        while let Some(p) = stack.pop() {
            if let Ok(iter) = fs::read_dir(&p) {
                for e in iter {
                    if let Ok(e) = e {
                        let path = e.path();
                        if path.is_dir() {
                            stack.push(path);
                        } else if path
                            .file_name()
                            .map(|s| s == "libmicrotex.a")
                            .unwrap_or(false)
                        {
                            return Some(p);
                        }
                    }
                }
            }
        }
        None
    }

    if let Some(libdir) = find_lib_dir(&dst) {
        println!("cargo:rustc-link-search=native={}", libdir.display());
    }

    println!("cargo:rustc-link-lib=static=microtex");
    println!("cargo:rerun-if-changed=./c++/lib/wrapper/cwrapper.h");
    println!("cargo:rerun-if-changed=./c++/lib/wrapper/callback.h");
    // Watch C++ source files for changes (important for dimension calculations, etc)
    println!("cargo:rerun-if-changed=./c++/lib/render/render.cpp");
    println!("cargo:rerun-if-changed=./c++/lib/render/render.h");
    println!("cargo:rerun-if-changed=./c++/res");

    // Link the C++ standard library depending on target
    let target = env::var("TARGET").unwrap_or_default();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
        // On macOS, explicitly link required frameworks for Quartz/CoreText support in Cairo
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=CoreText");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Link system graphics libraries required when CAIRO is enabled.
    // If we built vendored static cairo above (including auto-built), prefer to link the static copy.
    if using_vendored {
        // prefer static cairo provided by vendored build
        println!("cargo:rustc-link-lib=static=cairo");
    } else {
        println!("cargo:rustc-link-lib=cairo");
    }
    println!("cargo:rustc-link-lib=pango-1.0");
    println!("cargo:rustc-link-lib=pangocairo-1.0");
    println!("cargo:rustc-link-lib=fontconfig");

    // Also query pkg-config for any additional link flags required by our graphics
    // toolchain packages (static case) and emit appropriate cargo:rustc-link-lib directives.
    // This ensures dependencies (glib, gobject, freetype, pixman, png, z, etc.) are linked
    // when using vendored static Cairo/Pango builds.
    if std::process::Command::new("pkg-config")
        .arg("--version")
        .status()
        .is_ok()
    {
        for pkg in ["cairo", "pango", "pangocairo", "fontconfig"] {
            if let Ok(out) = std::process::Command::new("pkg-config")
                .arg("--libs")
                .arg("--static")
                .arg(pkg)
                .output()
            {
                if !out.status.success() {
                    continue;
                }
                let s = String::from_utf8_lossy(&out.stdout);
                // collect L dirs so we can check for lib<name>.a presence
                let mut search_dirs: Vec<String> = Vec::new();
                for token in s.split_whitespace() {
                    if token.starts_with("-L") {
                        let dir = &token[2..];
                        println!("cargo:rustc-link-search=native={}", dir);
                        search_dirs.push(dir.to_string());
                    }
                }
                let tokens: Vec<&str> = s.split_whitespace().collect();
                let mut i = 0;
                while i < tokens.len() {
                    let token = tokens[i];
                    // Handle macOS Frameworks emitted by pkg-config ("-framework CoreFoundation")
                    if token == "-framework" {
                        if i + 1 < tokens.len() {
                            let framework = tokens[i + 1];
                            println!("cargo:rustc-link-lib=framework={}", framework);
                            i += 2;
                            continue;
                        }
                    }

                    if token.starts_with("-l") {
                        let lib = &token[2..];
                        // On macOS there is no libdl: skip it when targeting apple platforms.
                        let target = env::var("TARGET").unwrap_or_default();
                        if lib == "dl" && target.contains("apple") {
                            eprintln!("Skipping lib 'dl' on apple target");
                            i += 1;
                            continue;
                        }
                        // check if a static lib exists in any of the search dirs
                        let mut has_static = false;
                        for d in &search_dirs {
                            if std::path::Path::new(d)
                                .join(format!("lib{}.a", lib))
                                .exists()
                            {
                                has_static = true;
                                break;
                            }
                        }
                        if using_bundle && has_static {
                            println!("cargo:rustc-link-lib=static={}", lib);
                        } else {
                            println!("cargo:rustc-link-lib={}", lib);
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    // Generate bindings for the C wrapper
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings = bindgen::Builder::default()
        .header("./c++/lib/wrapper/cwrapper.h")
        .header("./c++/lib/wrapper/callback.h")
        .clang_arg("-I./c++/lib")
        .clang_arg("-DHAVE_CWRAPPER")
        // parse headers as C++
        .clang_arg("-xc++")
        .clang_arg("-std=c++17")
        .allowlist_function("microtex_.*")
        .allowlist_type("(TextLayoutBounds|FontDesc|.*Ptr|DrawingData)")
        .generate_comments(false)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Collect CLM files in the c++/res directory and generate a small helper file
    let mut clms = Vec::new();
    let res_dir = Path::new("./c++/res");
    collect_clm_files(res_dir, &mut clms);

    let _manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .canonicalize()
        .expect("canonicalize manifest dir");

    let mut gen = String::new();
    gen.push_str("// Auto-generated by build.rs - do not edit\n");
    gen.push_str("/// Macro to access embedded CLM data by filename (runtime check).\n");
    gen.push_str("#[macro_export]\n");
    gen.push_str("macro_rules! embedded_clm {\n    ($name:expr) => {\n        match $name {\n");

    let mut avail = Vec::new();
    for p in &clms {
        // Keep the path as discovered (typically like "./c++/res/..."), and prefix with a slash
        // so concat!(env!("CARGO_MANIFEST_DIR"), "/./c++/res/...") works correctly.
        let include_path = format!("/{}", p.to_string_lossy());
        let filename = p.file_name().unwrap().to_string_lossy();
        avail.push(filename.to_string());
        gen.push_str(&format!(
            "            \"{}\" => include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"{}\")),\n",
            filename, include_path
        ));
    }

    gen.push_str("            _ => panic!(\"embedded clm not found: {}\", $name),\n");
    gen.push_str("        }\n    };\n}\n\n");

    // helper: list available
    gen.push_str("/// List available embedded CLM filenames.\n");
    gen.push_str("pub fn available_embedded_clms() -> &'static [&'static str] {\n    &[");
    for f in &avail {
        gen.push_str(&format!("\"{}\",", f));
    }
    gen.push_str("]\n}\n\n");

    // helper: get clm data
    gen.push_str("/// Get embedded CLM data by name.\n");
    gen.push_str(
        "pub fn get_embedded_clm(name: &str) -> Option<&'static [u8]> {\n    match name {\n",
    );
    for f in &avail {
        gen.push_str(&format!(
            "        \"{0}\" => Some(embedded_clm!(\"{0}\")),\n",
            f
        ));
    }
    gen.push_str("        _ => None,\n    }\n}\n");

    let mut fh = fs::File::create(out_path.join("embedded_clms.rs")).expect("create gen file");
    fh.write_all(gen.as_bytes()).expect("write gen file");
}
