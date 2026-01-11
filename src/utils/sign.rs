//! Signature utilities
//! 
//! Note: In the new architecture, signatures are generated within the browser context
//! during the login process. This module provides helper structs for API compatibility.

use anyhow::Result;

/// Signature result containing all required headers
#[derive(Debug, Clone)]
pub struct SignatureResult {
    pub x_s: String,
    pub x_t: i64,
    pub x_s_common: String,
}

/// Request details for signature generation
#[derive(Debug)]
pub struct RequestDetails<'a> {
    pub url: &'a str,
    pub method: &'a str,
    pub body: Option<&'a str>,
}

/// Legacy function - no longer used in new architecture
/// Kept for API compatibility but will return an error.
pub async fn generate_signature(_details: &RequestDetails<'_>) -> Result<(String, i64, String, String)> {
    Err(anyhow::anyhow!(
        "Legacy signature generation is deprecated. Use stored credentials from MongoDB."
    ))
}
