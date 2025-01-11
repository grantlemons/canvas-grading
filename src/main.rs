use std::{fs::File, str::FromStr, sync::Mutex};

use anyhow::Result;
use canvas_grading::{Command, Config, FileSubmission, Grade, Submission, CLI};
use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::try_parse()?;
    setup_logging();

    let config = Config::get(&cli)?;

    match cli.command {
        Command::Submissions => {
            let submissions =
                Submission::assignment_submissions(cli.assignment_id, &config).await?;
            let ungraded_submissions = submissions.iter().filter(|s| !s.graded());

            println!("{:#?}", ungraded_submissions);

            let mut files: Vec<FileSubmission> = Vec::new();
            for i in ungraded_submissions {
                files.push(i.get_file(&config).await?);
            }
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
