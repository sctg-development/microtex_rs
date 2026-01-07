/// Example demonstrating how to render a set of LaTeX formulas to SVG files.
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
    let output_dir = "svg_output/formula_examples";
    fs::create_dir_all(output_dir)?;
    println!("Output directory: {}", output_dir);

    // The formulas provided by the user
    let examples = [
        (
            "H_s",
            r#"\[H(s) = \prod_{i=1}^{n/2} \frac{1}{s^2 + \frac{\omega_0}{Q_i}s + \omega_0^2}\]"#,
        ),
        ("delta_f_symbolic", r#"\[\Delta f = \frac{f_s}{N}\]"#),
        (
            "delta_f_numeric",
            r#"\[\Delta f = \frac{48000}{4096} \approx 11.7 \text{ Hz}\]"#,
        ),
        (
            "f_peak",
            r#"\[f_{peak} = f_k + \frac{\delta f}{2} \cdot \frac{m_{k-1} - m_{k+1}}{m_{k-1} - 2m_k + m_{k+1}}\]"#,
        ),
        (
            "polynomial_C",
            r#"\[C = a_0 + a_1 \cdot S + a_2 \cdot S^2 + a_3 \cdot S^3 + a_4 \cdot S^4\]"#,
        ),
    ];

    // Rendering configuration (adjust as needed)
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
    println!(
        "Results: {} successful, {} errors",
        success_count, error_count
    );
    println!("Output files are in: {}/", output_dir);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}
