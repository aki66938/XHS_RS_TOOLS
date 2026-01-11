use anyhow::{anyhow, Result};
use qrcode::QrCode;
use qrcode::render::unicode;

/// QR code result containing both ASCII and URL representation
#[derive(Debug, Clone)]
pub struct QrCodeResult {
    /// ASCII art representation for terminal display
    pub ascii: String,
    /// Original URL for UI-based QR code rendering
    pub url: String,
}

/// Generate QR code as ASCII art for terminal display
pub fn generate_qr_ascii(url: &str) -> Result<QrCodeResult> {
    let code = QrCode::new(url.as_bytes())
        .map_err(|e| anyhow!("Failed to generate QR code: {}", e))?;
    
    // Render as unicode blocks for terminal
    let ascii = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)  // Invert for dark terminals
        .light_color(unicode::Dense1x2::Dark)
        .build();
    
    Ok(QrCodeResult {
        ascii,
        url: url.to_string(),
    })
}

/// Print QR code to terminal with a header
pub fn print_qr_to_terminal(url: &str, title: &str) -> Result<()> {
    let qr = generate_qr_ascii(url)?;
    
    println!("\n{}", title);
    println!("{}", "=".repeat(title.len()));
    println!("{}", qr.ascii);
    println!("URL: {}\n", qr.url);
    
    Ok(())
}
