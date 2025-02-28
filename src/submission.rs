use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::info;

use crate::{
    file::{CanvasFile, FileSubmission},
    Comment, Config, Grade,
};

#[derive(Debug, Deserialize)]
pub struct Submission {
    user_id: u64,
    assignment_id: u64,
    attempt: Option<u64>,
    /// None if submission has not been graded
    grader_id: Option<u64>,
    score: Option<f32>,
    workflow_state: WorkflowState,
    redo_request: bool,
    attachments: Option<Vec<CanvasFile>>,
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
            self.attempt()
        )
    }
}

impl Submission {
    pub fn graded(&self) -> bool {
        !self.redo_request && self.grader_id.is_some() && self.score.is_some()
    }

    pub fn submitted(&self) -> bool {
        matches!(self.workflow_state, WorkflowState::Submitted)
            || (!self.unsubmitted() && !self.graded())
    }

    pub fn unsubmitted(&self) -> bool {
        matches!(self.workflow_state, WorkflowState::Unsubmitted) || self.attachments.is_none()
    }

    pub fn assignment(&self) -> u64 {
        self.assignment_id
    }

    pub fn attempt(&self) -> u64 {
        self.attempt.unwrap_or(0)
    }

    pub fn user(&self) -> u64 {
        self.user_id
    }

    pub fn files(&self) -> Option<Vec<FileSubmission>> {
        Some(
            self.attachments
                .clone()?
                .into_iter()
                .map(|f| FileSubmission::new(self, f))
                .collect(),
        )
    }

    pub async fn assignment_submissions(assignment_id: u64, config: &Config) -> Result<Vec<Self>> {
        let url = format!(
            "{}/api/v1/courses/{}/assignments/{assignment_id}/submissions",
            config.base_url, config.course_id
        );

        let mut next_page_exists = true;
        let mut page = 1;
        let mut responses: Vec<Self> = Vec::new();
        while next_page_exists {
            let page_str = page.to_string();
            let form = HashMap::from([
                ("workflow_state", "submitted"),
                // ("per_page", "50"),
                ("page", &page_str),
            ]);

            info!("Requesting from \"{url}\", page {page}");
            let response = config.client.get(&url).query(&form).send().await?;
            let headers = response.headers().clone();

            info!("Getting body from response...");
            let body = response.text().await?;
            let untyped: serde_json::Value =
                serde_json::from_str(&body).context("Failed to parse invalid JSON body.")?;
            info!("Parsed into untyped JSON");

            info!("Attempting to parse JSON into structured data type...");

            let mut structured = serde_json::from_str(&body).with_context(|| {
                format!("Unable to parse response to data type: {:#?}", untyped)
            })?;
            responses.append(&mut structured);

            next_page_exists = headers
                .get("Link")
                .context("Failed to get link header.")?
                .to_str()
                .context("Failed to stringify link header")?
                .contains("next");
            page += 1;
        }

        Ok(responses.into_iter().filter(Self::submitted).collect())
    }

    pub async fn count_submissions(
        assignment_id: u64,
        predicate: &dyn Fn(&Self) -> bool,
        config: &Config,
    ) -> Result<usize> {
        let url = format!(
            "{}/api/v1/courses/{}/assignments/{assignment_id}/submissions",
            config.base_url, config.course_id
        );

        let mut next_page_exists = true;
        let mut page = 1;
        let mut responses: Vec<Self> = Vec::new();
        while next_page_exists {
            let page_str = page.to_string();
            let form = HashMap::from([
                ("workflow_state", "submitted"),
                // ("per_page", "50"),
                ("page", &page_str),
            ]);

            info!("Requesting from \"{url}\", page {page}");
            let response = config.client.get(&url).query(&form).send().await?;
            let headers = response.headers().clone();

            info!("Getting body from response...");
            let body = response.text().await?;
            let untyped: serde_json::Value =
                serde_json::from_str(&body).context("Failed to parse invalid JSON body.")?;
            info!("Parsed into untyped JSON");

            info!("Attempting to parse JSON into structured data type...");

            let mut structured = serde_json::from_str(&body).with_context(|| {
                format!("Unable to parse response to data type: {:#?}", untyped)
            })?;
            responses.append(&mut structured);

            next_page_exists = headers
                .get("Link")
                .context("Failed to get link header.")?
                .to_str()
                .context("Failed to stringify link header")?
                .contains("next");
            page += 1;
        }

        Ok(responses.into_iter().filter(predicate).count())
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

    pub async fn update_grades_with_comments(
        assignment_id: u64,
        grades: &[Grade],
        comments: &[Comment],
        config: &Config,
    ) -> Result<()> {
        let form_grades = grades.iter().map(|g| {
            (
                format!("grade_data[{}][posted_grade]", g.user_id),
                g.grade.to_string(),
            )
        });
        let form_comments = comments.iter().map(|c| {
            (
                format!("grade_data[{}][text_comment]", c.user_id),
                c.comment.to_owned(),
            )
        });

        let form = HashMap::<String, String>::from_iter(form_grades.chain(form_comments));

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
