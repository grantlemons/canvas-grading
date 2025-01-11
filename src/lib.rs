use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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
    /// Override the Canvas access token from config
    /// Either this or the option in config MUST BE SET
    #[arg(long)]
    pub access_token: Option<String>,

    /// Override the course id from config
    /// Either this or the option in config MUST BE SET
    #[arg(long, short)]
    pub course_id: Option<u64>,

    /// Override the base URL for Canvas from config
    /// Either this or the option in config MUST BE SET
    #[arg(long, short)]
    pub base_url: Option<String>,

    #[command(subcommand)]
    pub command: Command,

    /// Assignment ID
    pub assignment_id: u64,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    Submissions,
    Grade,
}

#[derive(Debug)]
pub struct Grade {
    user_id: u64,
    grade: f32,
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

    Ok(reqwest::ClientBuilder::new()
        .default_headers(headers)
        .build()?)
}
