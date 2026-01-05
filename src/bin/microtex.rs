/// Command-line interface for MicroTeX LaTeX to SVG conversion.
///
/// This simple CLI allows converting LaTeX formulas to SVG files.
use clap::{Parser, ValueEnum};
use microtex_rs::{MicroTex, RenderConfig};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "microtex")]
#[command(about = "Convert LaTeX formulas to SVG", long_about = None)]
struct Args {
    /// LaTeX formula to render
    #[arg(value_name = "LATEX")]
    formula: String,

    /// Output SVG file path
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// DPI (dots per inch) for rendering
    #[arg(short, long, default_value = "720")]
    dpi: i32,

    /// Line width in pixels
    #[arg(long, default_value = "20.0")]
    line_width: f32,

    /// Line height in pixels
    #[arg(long, default_value = "6.666667")]
    line_height: f32,

    /// Text color (ARGB hex, e.g., 0xff000000 for black)
    #[arg(long, default_value = "0xff000000")]
    color: String,

    /// Enable path-based glyph rendering
    #[arg(long, default_value = "true")]
    use_path: bool,

    /// Print SVG to stdout instead of file
    #[arg(short, long)]
    stdout: bool,
}

fn parse_color(s: &str) -> Result<u32, String> {
    let s = s.trim_start_matches("0x");
    u32::from_str_radix(s, 16).map_err(|e| format!("Invalid color: {}", e))
}

/// Run the CLI logic given parsed `Args`. Returns the rendered SVG string on success.
fn run_with_args(args: &Args) -> Result<String, Box<dyn std::error::Error>> {
    // Parse color
    let text_color = parse_color(&args.color)?;

    // Create renderer
    let renderer = MicroTex::new()?;

    // Create config
    let config = RenderConfig {
        dpi: args.dpi,
        line_width: args.line_width,
        line_height: args.line_height,
        text_color,
        render_glyph_use_path: args.use_path,
        ..Default::default()
    };

    // Render
    let svg = renderer.render(&args.formula, &config)?;

    // Output
    if args.stdout {
        // When stdout is requested, just return the svg string
        Ok(svg)
    } else {
        let output_path = args
            .output
            .clone()
            .unwrap_or_else(|| PathBuf::from("output.svg"));

        fs::write(&output_path, &svg)?;
        Ok(svg)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .try_init()
        .ok();

    let args = Args::parse();

    eprintln!("Initializing MicroTeX renderer...");

    let svg = run_with_args(&args)?;

    eprintln!("✓ Rendering successful! ({} bytes)", svg.len());

    if args.stdout {
        println!("{}", svg);
    } else if let Some(output) = args.output {
        eprintln!("✓ Saved to: {}", output.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use microtex_rs::test_control as tc;

    #[test]
    fn test_parse_color_ok() {
        assert_eq!(parse_color("0xff000000").unwrap(), 0xff000000);
        assert_eq!(parse_color("ff000000").unwrap(), 0xff000000);
    }

    #[test]
    fn test_parse_color_err() {
        assert!(parse_color("zzzz").is_err());
    }

    #[test]
    fn test_run_with_args_stdout() {
        let _g = tc::lock_test();
        tc::set_init_succeed(true);
        tc::set_parse_succeed(true);
        tc::set_return_empty(false);
        tc::set_buffer(b"<svg>cli</svg>");

        let args = Args {
            formula: "x".to_string(),
            output: None,
            dpi: 720,
            line_width: 20.0,
            line_height: 20.0 / 3.0,
            color: "0xff000000".to_string(),
            use_path: true,
            stdout: true,
        };

        let svg = run_with_args(&args).expect("run should succeed");
        assert!(svg.contains("<svg"));
    }
}