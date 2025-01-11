use std::{fs::File, str::FromStr, sync::Mutex};

use anyhow::{Context, Result};
use canvas_grading::{Command, Config, Grade, CLI};
use clap::Parser;
use std::io;

fn main() -> Result<()> {
    let cli = CLI::try_parse()?;
    setup_logging();

    // let config = Config::get(&cli)?;

    match cli.command {
        Command::Submissions => todo!(),
        Command::Grade => {
            let grades = read_grades()?;

            println!("Grades: {:#?}", grades);
        }
    }

    Ok(())
}

fn read_grades() -> Result<Vec<Grade>> {
    let stdin = io::stdin();

    Ok(stdin
        .lines()
        .map_while(Result::ok)
        .map(|line| Grade::from_str(line.trim()))
        .map_while(Result::ok)
        .collect())
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
