//! Browser-based login using Python Playwright
//! 
//! This module calls an external Python script to handle browser automation
//! for user login via QR code scanning.

use anyhow::{anyhow, Result};
use std::process::Command;
use tracing::{info, warn};

/// Trigger browser-based login using Python Playwright
/// 
/// This function spawns a Python process that:
/// 1. Opens a browser window
/// 2. Navigates to XHS
/// 3. Waits for user to scan QR code
/// 4. Captures cookies after login
/// 5. Saves credentials to MongoDB
pub async fn trigger_python_login() -> Result<()> {
    info!("Triggering Python Playwright login script...");
    
    // Find Python executable
    let python = find_python()?;
    
    let script_path = std::path::Path::new("scripts/login.py");
    if !script_path.exists() {
        return Err(anyhow!("Login script not found at {:?}", script_path));
    }
    
    info!("Running: {} {:?}", python, script_path);
    
    // Run Python script
    let output = Command::new(&python)
        .arg(script_path)
        .output()
        .map_err(|e| anyhow!("Failed to run Python script: {}", e))?;
    
    // Print stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("JSON_RESULT:") {
            // Parse the JSON result
            let json_str = line.trim_start_matches("JSON_RESULT:");
            if let Ok(result) = serde_json::from_str::<serde_json::Value>(json_str) {
                if result["success"].as_bool() == Some(true) {
                    info!("Login successful via Python script");
                    return Ok(());
                } else {
                    let error = result["error"].as_str().unwrap_or("Unknown error");
                    return Err(anyhow!("Login failed: {}", error));
                }
            }
        } else {
            println!("{}", line);
        }
    }
    
    // Print stderr if any
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        warn!("Python script stderr: {}", stderr);
    }
    
    if !output.status.success() {
        return Err(anyhow!("Python login script failed with exit code: {:?}", output.status.code()));
    }
    
    Ok(())
}

/// Find Python executable
fn find_python() -> Result<String> {
    // Try common Python executable names
    let candidates = ["python", "python3", "py"];
    
    for candidate in candidates {
        if Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok()
        {
            return Ok(candidate.to_string());
        }
    }
    
    Err(anyhow!("Python not found. Please install Python and ensure it's in PATH."))
}
