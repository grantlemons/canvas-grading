use std::{fs::File, io::Read, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// A struct representing an access token for Canvas. Can be serialized and deserialized but does
/// not implement Debug.
#[derive(Serialize, Deserialize, Clone)]
pub struct AccessToken(String);

#[derive(Deserialize)]
pub struct ConfigFile {
    pub access_token: Option<AccessToken>,
    pub course_id: Option<u64>,
}

pub struct Config {
    pub access_token: AccessToken,
    pub course_id: u64,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
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

impl Config {
    pub fn get(command_line_options: &Cli) -> Result<Self> {
        let config_file_path = dirs::config_dir()
            .context("Unable to get config dir for system")?
            .join("config.toml");

        let config_contents = ConfigFile::read_from_file(&config_file_path)?;

        Ok(Self {
            access_token: command_line_options
                .access_token
                .clone()
                .map(AccessToken)
                .or(config_contents.access_token)
                .ok_or(anyhow!("Access token not configured!"))?,
            course_id: command_line_options
                .course_id
                .or(config_contents.course_id)
                .ok_or(anyhow!("Course id not configured!"))?,
        })
    }
}

impl ConfigFile {
    pub fn read_from_file(path: &PathBuf) -> Result<Self> {
        let config_file = File::open(path);

        let mut file_contents = String::new();
        if let Ok(mut file) = config_file {
            file.read_to_string(&mut file_contents)
                .context("Unable to read file contents.")?;
        }

        toml::from_str(&file_contents).context("Unable to parse config as TOML")
    }
}
