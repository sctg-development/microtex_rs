/// Simple example rendering a basic LaTeX formula to SVG.
use microtex_rs::{MicroTex, RenderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init()
        .ok();

    println!("Initializing MicroTeX renderer...");
    let renderer = MicroTex::new()?;

    println!("Creating render configuration...");
    let config = RenderConfig::default();

    let formulas = [
        ("Einstein's mass-energy equivalence", r#"\[E = mc^2\]"#),
        ("Pythagorean theorem", r#"\[a^2 + b^2 = c^2\]"#),
        (
            "Quadratic formula",
            r#"\[x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}\]"#,
        ),
        ("Sum notation", r#"\[\sum_{i=1}^{n} i = \frac{n(n+1)}{2}\]"#),
        (
            "Integration",
            r#"\[\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}\]"#,
        ),
    ];

    for (description, latex) in &formulas {
        println!("\nRendering: {}", description);
        println!("LaTeX: {}", latex);

        match renderer.render(latex, &config) {
            Ok(svg) => {
                let svg_len = svg.len();
                println!("✓ Success! Generated SVG ({} bytes)", svg_len);
                println!("  First 100 chars: {}", &svg[..100.min(svg.len())]);
            }
            Err(e) => {
                eprintln!("✗ Failed: {}", e);
            }
        }
    }
    println!("\n✓ All examples completed!");
    Ok(())
}
