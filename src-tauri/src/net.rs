//! Shared HTTP client. One place to set the user agent and TLS backend.

use anyhow::Result;
use std::time::Duration;

pub fn client() -> Result<reqwest::Client> {
    let c = reqwest::Client::builder()
        .user_agent("FrontPressLocal/0.1 (+https://frontpress.studio)")
        .connect_timeout(Duration::from_secs(20))
        .build()?;
    Ok(c)
}
