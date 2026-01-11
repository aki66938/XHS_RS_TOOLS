use std::sync::Arc;
use reqwest::{Client, cookie::Jar};
use anyhow::Result;

#[derive(Clone)]
pub struct XhsClient {
    http_client: Client,
    cookie_store: Arc<Jar>,
}

impl XhsClient {
    pub fn new() -> Result<Self> {
        let cookie_store = Arc::new(Jar::default());
        // Configure the client with a standard browser User-Agent
        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_store.clone())
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()?;

        Ok(Self {
            http_client: client,
            cookie_store,
        })
    }

    pub fn get_client(&self) -> &Client {
        &self.http_client
    }

    pub fn get_cookie_store(&self) -> Arc<Jar> {
        self.cookie_store.clone()
    }
}
