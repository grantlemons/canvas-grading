use std::{fs::File, str::FromStr, sync::Mutex};

use anyhow::Result;
use canvas_grading::{CanvasFile, Command, Config, Grade, Submission, CLI};
use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::try_parse()?;
    setup_logging();

    let config = Config::get(&cli)?;

    match cli.command {
        Command::Submissions => {
            let submissions = Submission::get_all(&config).await?;
            let ungraded_submissions = submissions.iter().filter(|s| !s.graded());

            let mut files: Vec<CanvasFile> = Vec::new();
            for i in ungraded_submissions.to_owned() {
                files.push(i.get_file(&config).await?);
            }

            println!("{:#?}", ungraded_submissions);
        }
        Command::Grade => {
            let grades = read_grades();

            Submission::update_grades(cli.assignment_id, &grades, &config).await?;

            println!("Grades: {:#?}", grades);
        }
    }

    Ok(())
}

fn read_grades() -> Vec<Grade> {
    let stdin = io::stdin();

    stdin
        .lines()
        .map_while(Result::ok)
        .map(|line| Grade::from_str(line.trim()))
        .map_while(Result::ok)
        .collect()
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
