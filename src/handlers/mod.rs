//! HTTP Handlers Module
//! 
//! This module contains all HTTP request handlers organized by domain.

pub mod search;
pub mod auth;
pub mod notification;
pub mod user;
pub mod feed;
pub mod media;

// Re-export all handlers for convenient access
pub use search::*;
pub use auth::*;
pub use notification::*;
pub use user::*;
pub use feed::*;
pub use media::*;
