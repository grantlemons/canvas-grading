use std::{io::Cursor, path::Path};

use anyhow::{anyhow, Result};
use serde::Deserialize;
use tracing::info;


#[derive(Debug, Clone)]
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
    pub fn new(user_id: u64, assignment_id: u64, file: CanvasFile) -> Self {
        Self {
            user_id,
            assignment_id,
            file,
        }
    }

    pub async fn download(&self, directiory: &Path) -> Result<()> {
        let path = directiory.join(self.to_string());

        info!(
            "Downloading \"{}\" to {}",
            self.file.url(),
            path.to_str().unwrap()
        );
        self.file.download(&path).await
    }
}

#[derive(Debug, Clone, Deserialize)]
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

    pub async fn download(&self, path: &Path) -> Result<()> {
        let response = reqwest::get(&self.url).await?;

        // Create parent directories
        std::fs::create_dir_all(
            path.parent()
                .ok_or(anyhow!("Path does not have a parent directory!"))?,
        )?;

        let mut file = std::fs::File::create(path)?;
        let mut contents = Cursor::new(response.bytes().await?);
        std::io::copy(&mut contents, &mut file)?;

        Ok(())
    }
}
