use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

mod config;
mod file;
mod submission;

pub use config::Config;
pub use file::FileSubmission;
pub use submission::Submission;

/// A struct representing an access token for Canvas. Hides its value from Debug.
#[derive(Serialize, Deserialize, Clone)]
pub struct AccessToken(String);

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccessToken")
    }
}

impl AccessToken {
    pub fn secret(&self) -> &str {
        &self.0
    }
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct CLI {
    /// Override the Canvas access token from config.
    /// Either this or the option in config MUST BE SET
    #[arg(long)]
    pub access_token: Option<String>,

    /// Override the course id from config.
    /// Either this or the option in config MUST BE SET
    #[arg(long, short)]
    pub course_id: Option<u64>,

    /// Override the base URL for Canvas from config.
    /// Either this or the option in config MUST BE SET
    #[arg(long, short)]
    pub base_url: Option<String>,

    /// Generate shell completion
    #[arg(long)]
    generate: Option<Shell>,

    #[command(subcommand)]
    pub command: Command,

    /// Assignment ID in Canvas
    pub assignment_id: u64,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Download ungraded submissions and print the paths to standard output
    Submissions,
    /// Upload grades and comments from file
    Grade,
    /// Count the number of submissions meeting a requirement
    #[command(subcommand)]
    Count(CountOptions),
}

#[derive(Subcommand, Clone, Debug)]
pub enum CountOptions {
    Unsubmitted,
    Submitted,
    Graded,
}

#[derive(Debug)]
pub struct Grade {
    pub user_id: u64,
    pub grade: f32,
}

impl FromStr for Grade {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(": ");
        let user_id = parts
            .next()
            .context("Unable to parse user id from stdin.")?;
        let grade = parts.next().context("Unable to parse grade from stdin.")?;

        Ok(Self {
            user_id: user_id.parse().context("Unable to parse user id to u64")?,
            grade: grade.parse().context("Unable to parse grade to f32")?,
        })
    }
}

#[derive(Debug)]
pub struct Comment {
    pub user_id: u64,
    pub comment: String,
}

impl FromStr for Comment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(": ");
        let user_id = parts
            .next()
            .context("Unable to parse user id from stdin.")?;
        let comment = parts
            .next()
            .context("Unable to parse comment from stdin.")?;

        Ok(Self {
            user_id: user_id.parse().context("Unable to parse user id to u64")?,
            comment: comment.to_string(),
        })
    }
}

pub fn create_client(auth_token: AccessToken) -> Result<Client> {
    info!("Building application reqwest client...");
    info!("Setting auth header...");
    let mut auth_bearer: reqwest::header::HeaderValue = ("Bearer ".to_owned()
        + auth_token.secret())
    .try_into()
    .unwrap();
    auth_bearer.set_sensitive(true);
    info!("Auth header set!");

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, auth_bearer);
    headers.insert("per_page", 100.into());

    Ok(reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?)
}
