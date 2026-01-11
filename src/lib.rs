pub mod api;
pub mod auth;  // New authentication module
pub mod client;
pub mod models;
pub mod utils;
pub mod server;

pub use client::XhsClient;
pub use auth::{UserCredentials, CredentialStorage, AuthService};

