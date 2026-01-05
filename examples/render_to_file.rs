/// Example demonstrating how to render LaTeX formulas to SVG files.
use microtex_rs::{MicroTex, RenderConfig};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init()
        .ok();

    println!("Initializing MicroTeX renderer...");
    let renderer = MicroTex::new()?;

    // Create output directory
    let output_dir = "svg_output";
    fs::create_dir_all(output_dir)?;
    println!("Output directory: {}", output_dir);

    // Example formulas with descriptive filenames
    let examples = [
        ("einstein_mass_energy", r#"\[E = mc^2\]"#),
        ("pythagoras_theorem", r#"\[a^2 + b^2 = c^2\]"#),
        ("quadratic_formula", r#"\[x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}\]"#),
        ("golden_ratio", r#"\[\phi = \frac{1 + \sqrt{5}}{2}\]"#),
        ("integration_example", r#"\[\int_0^{\infty} e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2}\]"#),
        ("limit_definition", r#"\[\lim_{x \to 0} \frac{\sin x}{x} = 1\]"#),
        ("matrix_example", r#"\[\begin{pmatrix} 1 & 2 \\ 3 & 4 \end{pmatrix}\]"#),
        ("derivative", r#"\[\frac{d}{dx} \left[ x^n \right] = n x^{n-1}\]"#),
    ];

    // Standard rendering configuration
    let config = RenderConfig {
        dpi: 720,
        line_width: 20.0,
        line_height: 20.0 / 3.0,
        text_color: 0xff000000,
        render_glyph_use_path: true,
        ..Default::default()
    };

    let mut success_count = 0;
    let mut error_count = 0;

    for (name, latex) in &examples {
        let filename = format!("{}.svg", name);
        let filepath = Path::new(output_dir).join(&filename);

        print!("Rendering {} ... ", name);

        match renderer.render(latex, &config) {
            Ok(svg) => {
                fs::write(&filepath, svg)?;
                println!("✓ Saved to {}", filepath.display());
                success_count += 1;
            }
            Err(e) => {
                println!("✗ Error: {}", e);
                error_count += 1;
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Results: {} successful, {} errors", success_count, error_count);
    println!("Output files are in: {}/", output_dir);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
