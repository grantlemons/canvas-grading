use std::{io::Cursor, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::info;

use crate::{Config, Submission};

pub struct FileSubmission {
    user_id: u64,
    assignment_id: u64,
    file: CanvasFile,
}

impl std::fmt::Display for FileSubmission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}_{}",
            self.user_id,
            self.assignment_id,
            self.file.filename()
        )
    }
}

impl FileSubmission {
    pub async fn get(submission: &Submission, config: &Config) -> Result<Self> {
        Ok(Self {
            user_id: submission.user(),
            assignment_id: submission.assignment(),
            file: CanvasFile::get(submission.file_id(), config).await?,
        })
    }

    pub async fn download(&self, directiory: &Path) -> Result<()> {
        let path = directiory.join(self.to_string());

        self.file.download(&path).await
    }
}

#[derive(Debug, Deserialize)]
pub struct CanvasFile {
    url: String,
    filename: String,
}

impl CanvasFile {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub async fn get(id: u64, config: &Config) -> Result<CanvasFile> {
        let response = config
            .client
            .get(format!("{}/api/v1/files/{}", config.base_url, id))
            .send()
            .await?;

        info!("Getting body from response...");
        let body = response.text().await?;
        let untyped: serde_json::Value =
            serde_json::from_str(&body).context("Failed to parse invalid JSON body.")?;
        info!("Parsed into untyped JSON");

        info!("Attempting to parse JSON into structured data type...");
        serde_json::from_str(&body)
            .with_context(|| format!("Unable to parse response to data type: {:#?}", untyped))
    }

    pub async fn download(&self, path: &Path) -> Result<()> {
        let response = reqwest::get(&self.url).await?;

        let mut file = std::fs::File::create(path)?;
        let mut contents = Cursor::new(response.bytes().await?);
        std::io::copy(&mut contents, &mut file)?;

        Ok(())
    }
}
