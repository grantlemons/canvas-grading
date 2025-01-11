use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::info;

use crate::Config;

#[derive(Debug, Deserialize)]
pub struct CanvasFile {
    id: u64,
    url: String,
    filename: String,
}

impl CanvasFile {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub async fn get(id: u64, config: &Config) -> Result<CanvasFile> {
        let response = reqwest::get(format!("{}/api/v1/files/{}", config.base_url, id)).await?;

        info!("Getting body from response...");
        let body = response.text().await?;
        let untyped: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse invalid JSON body.")?;
        info!("Parsed into untyped JSON");

        info!("Attempting to parse JSON into structured data type...");
        serde_json::from_str(&body)
            .with_context(|| format!("Unable to parse response to data type: {:#?}", untyped))
    }
}
