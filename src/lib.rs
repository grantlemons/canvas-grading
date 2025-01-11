use std::str::FromStr;

use anyhow::Context;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

mod config;

pub use config::Config;

/// A struct representing an access token for Canvas. Hides its value from Debug.
#[derive(Serialize, Deserialize, Clone)]
pub struct AccessToken(String);

impl std::fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AccessToken")
    }
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct CLI {
    /// Override the Canvas access token from config
    #[arg(long)]
    pub access_token: Option<String>,

    /// Override the course id from config
    /// Either this or the option in config MUST BE SET
    #[arg(long, short)]
    pub course_id: Option<u64>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    Submissions,
    Grade,
}

#[derive(Debug)]
pub struct Grade {
    user_id: String,
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
