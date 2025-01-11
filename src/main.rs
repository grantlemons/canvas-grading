use std::{fs::File, sync::Mutex};

use anyhow::Result;
use canvas_grading::{Cli, Config};
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::try_parse()?;
    setup_logging();

    let config = Config::get(&cli)?;

    Ok(())
}

#[allow(unused)]
fn setup_logging() {
    let log_file = File::create("most-recent.log").unwrap();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .with_writer(Mutex::new(log_file))
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
