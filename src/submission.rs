use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::info;

use crate::{file::FileSubmission, Config, Grade};

#[derive(Debug, Deserialize)]
pub struct Submission {
    user_id: u64,
    assignment_id: u64,
    canvadoc_document_id: u64,
    attempt: u64,
    /// None if submission has not been graded
    grader_id: Option<()>,
    redo_request: bool,
}

impl std::fmt::Display for Submission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.user_id, self.attempt)
    }
}

impl Submission {
    pub fn graded(&self) -> bool {
        self.grader_id.is_some() && !self.redo_request
    }

    pub fn assignment(&self) -> u64 {
        self.assignment_id
    }

    pub fn attempt(&self) -> u64 {
        self.attempt
    }

    pub fn user(&self) -> u64 {
        self.user_id
    }

    pub fn file_id(&self) -> u64 {
        self.canvadoc_document_id
    }

    pub async fn get_all(config: &Config) -> Result<Vec<Self>> {
        let response = reqwest::get(format!(
            "{}/api/v1/courses/{}/students/submissions",
            config.base_url, config.course_id
        ))
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

    pub async fn get_file(&self, config: &Config) -> Result<FileSubmission> {
        FileSubmission::get(self, config).await
    }

    pub async fn update_grades(
        assignment_id: u64,
        grades: &[Grade],
        config: &Config,
    ) -> Result<()> {
        let form = HashMap::<String, f32>::from_iter(
            grades
                .iter()
                .map(|g| (format!("grade_data[{}][posted_grade]", g.user_id), g.grade)),
        );

        crate::create_client(config.access_token.clone())?
            .post(format!(
                "{}/api/v1/courses/{}/assignments/{assignment_id}/submissions/update_grades",
                config.base_url, config.course_id
            ))
            .form(&form)
            .send()
            .await?;

        Ok(())
    }
}
