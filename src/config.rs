use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::{fs::File, io::Read, path::PathBuf};

use crate::{AccessToken, CLI};

#[derive(Debug, Clone)]
pub struct Config {
    pub access_token: AccessToken,
    pub course_id: u64,
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigFile {
    pub access_token: Option<AccessToken>,
    pub course_id: Option<u64>,
    pub base_url: Option<String>,
}

impl Config {
    pub fn get(command_line_options: &CLI) -> Result<Self> {
        let config_file_path = dirs::config_dir()
            .context("Unable to get config dir for system")?
            .join("grading")
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
            base_url: command_line_options
                .base_url
                .clone()
                .or(config_contents.base_url)
                .ok_or(anyhow!("Base URL not configured!"))?,
        })
    }
}

impl ConfigFile {
    pub fn read_from_file(path: &PathBuf) -> Result<Self> {
        let config_file = File::open(path);

        let mut buffer = String::new();
        if let Ok(mut file) = config_file {
            file.read_to_string(&mut buffer)
                .context("Unable to read file contents.")?;
        }

        toml::from_str(&buffer).context("Unable to parse config as TOML")
    }
}
