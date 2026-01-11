// Quick test to verify data-dpi attribute
fn main() {
    use microtex_rs::{MicroTex, RenderConfig};
    
    let renderer = MicroTex::new().expect("init");
    let config = RenderConfig::default();
    
    let svg = renderer.render(r#"\[E = mc^2\]"#, &config).expect("render");
    
    // Show the opening tag
    if let Some(svg_start) = svg.find("<svg") {
        if let Some(close) = svg[svg_start..].find('>') {
            let tag = &svg[svg_start..svg_start + close + 1];
            println!("SVG opening tag:");
            println!("{}", tag);
            
            // Verify data-dpi is present
            if tag.contains(r#"data-dpi="720""#) {
                println!("\n✓ SUCCESS: data-dpi attribute is present!");
            } else {
                println!("\n✗ FAILED: data-dpi attribute is missing!");
            }
        }
    }
}
