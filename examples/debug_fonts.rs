fn main() {
    println!("Available embedded CLM files:");
    
    // Try to access the embedded CLM data
    // We'll call the function generated in embedded_clms.rs
    let available = microtex_rs::available_embedded_clms();
    for font in available {
        match microtex_rs::get_embedded_clm(font) {
            Some(data) => println!("  {} ({} bytes)", font, data.len()),
            None => println!("  {} (FAILED TO LOAD)", font),
        }
    }
}
