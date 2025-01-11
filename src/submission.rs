use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::info;

use crate::{file::FileSubmission, Config, Grade};

#[derive(Debug, Deserialize)]
pub struct Submission {
    user_id: u64,
    assignment_id: u64,
    // canvadoc_document_id: Option<u64>,
    attempt: Option<u64>,
    /// None if submission has not been graded
    grader_id: Option<u64>,
    workflow_state: WorkflowState,
    redo_request: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowState {
    Unsubmitted,
    Submitted,
    Graded,
    PendingReview,
}

impl std::fmt::Display for Submission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}_{}",
            self.user_id,
            self.assignment_id,
            self.attempt.unwrap_or(0)
        )
    }
}

impl Submission {
    pub fn graded(&self) -> bool {
        !self.redo_request && self.grader_id.is_some()
    }

    pub fn sumitted(&self) -> bool {
        matches!(self.workflow_state, WorkflowState::Submitted)
    }

    pub fn assignment(&self) -> u64 {
        self.assignment_id
    }

    pub fn attempt(&self) -> Option<u64> {
        self.attempt
    }

    pub fn user(&self) -> u64 {
        self.user_id
    }

    pub fn file_id(&self) -> u64 {
        0
    }

    pub async fn assignment_submissions(assignment_id: u64, config: &Config) -> Result<Vec<Self>> {
        let url = format!(
            "{}/api/v1/courses/{}/students/submissions",
            config.base_url, config.course_id
        );
        info!("Requesting from \"{url}\"");

        let tmp = format!("[{}]", assignment_id);
        let form = HashMap::from([("workflow_state", "submitted"), ("assignment_ids", &tmp)]);
        let response = config.client.get(url).query(&form).send().await?;

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

        config
            .client
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
