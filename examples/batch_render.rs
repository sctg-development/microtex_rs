/// Example demonstrating batch rendering with different configurations.
use microtex_rs::{MicroTex, RenderConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init()
        .ok();

    println!("Initializing MicroTeX renderer...");
    let renderer = MicroTex::new()?;

    // Create output directory
    fs::create_dir_all("examples/batch_output")?;

    // Different configurations for different purposes
    let configs = [
        (
            "screen",
            RenderConfig {
                dpi: 96,
                line_width: 12.0,
                line_height: 12.0 / 3.0,
                text_color: 0xff000000,
                render_glyph_use_path: false,
                ..Default::default()
            },
        ),
        (
            "print_quality",
            RenderConfig {
                dpi: 300,
                line_width: 20.0,
                line_height: 20.0 / 3.0,
                text_color: 0xff000000,
                render_glyph_use_path: true,
                ..Default::default()
            },
        ),
        (
            "high_dpi",
            RenderConfig {
                dpi: 1440,
                line_width: 40.0,
                line_height: 40.0 / 3.0,
                text_color: 0xff000000,
                render_glyph_use_path: true,
                ..Default::default()
            },
        ),
        (
            "inverted_colors",
            RenderConfig {
                dpi: 720,
                line_width: 20.0,
                line_height: 20.0 / 3.0,
                text_color: 0xffffffff, // white text
                render_glyph_use_path: true,
                ..Default::default()
            },
        ),
    ];

    let latex = r#"
        \[
        \sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}
        \]
    "#;

    println!("\nRendering formula with different configurations:\n");

    for (config_name, config) in &configs {
        let filename = format!("examples/batch_output/basel_problem_{}.svg", config_name);

        print!("Rendering with {} config ... ", config_name);

        match renderer.render(latex, config) {
            Ok(svg) => {
                let size = svg.len();
                fs::write(&filename, svg)?;
                println!("✓ ({} bytes)", size);
            }
            Err(e) => {
                println!("✗ {}", e);
            }
        }
    }

    // Also demonstrate rendering a list of formulas
    println!("\n✓ Batch rendering complete!");
    println!("Output files: examples/batch_output/");

    Ok(())
}
