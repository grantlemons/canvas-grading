use std::{fs::File, str::FromStr, sync::Mutex};

use anyhow::{anyhow, Result};
use canvas_grading::{Command, Comment, Config, Grade, Submission, CLI};
use clap::{CommandFactory, Parser};
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup autocomplete
    let matches = CLI::command().get_matches();
    if let Some(generator) = matches
        .get_one::<clap_complete::aot::Shell>("generate")
        .copied()
    {
        let mut cmd = CLI::command();
        eprintln!("Generating completion file for {generator}...");
        let name = cmd.get_name().to_string();
        clap_complete::generate(generator, &mut cmd, name, &mut io::stdout());

        return Ok(());
    }

    let cli = CLI::try_parse()?;

    setup_logging();
    let config = Config::get(&cli)?;

    match cli.command {
        Command::Submissions => {
            let submissions =
                Submission::assignment_submissions(cli.assignment_id, &config).await?;
            let files: Vec<_> = submissions
                .iter()
                .flat_map(Submission::files)
                .flatten()
                .collect();

            let runtime_directiory = dirs::runtime_dir()
                .expect("Unable to get runtime directiory for system!")
                .join("grading");

            for file in files {
                file.download(&runtime_directiory).await?;
                println!(
                    "{}",
                    runtime_directiory
                        .join(file.to_string())
                        .to_str()
                        .ok_or(anyhow!("Unable to convert path to string"))?
                );
            }
        }
        Command::Grade => {
            let (grades, comments) = read_grades_and_comments();

            // Reduce multiple comments to a single one
            let mut reduced_comments = Vec::new();
            let mut acc = String::new();
            let mut user_id: Option<u64> = None;
            for comment in comments {
                if user_id.map_or(true, |id| id == comment.user_id) {
                    acc.push_str(&comment.comment);
                    acc.push('\n');
                } else if let Some(uid) = user_id {
                    reduced_comments.push(Comment {
                        user_id: uid,
                        comment: acc.to_owned(),
                    });
                    acc = comment.comment;
                }

                user_id = Some(comment.user_id)
            }
            if let Some(user_id) = user_id {
                reduced_comments.push(Comment {
                    user_id,
                    comment: acc.to_owned(),
                });
            }

            Submission::update_grades_with_comments(
                cli.assignment_id,
                &grades,
                &reduced_comments,
                &config,
            )
            .await?;
        }
    }

    Ok(())
}

fn read_grades_and_comments() -> (Vec<Grade>, Vec<Comment>) {
    let stdin = io::stdin();

    let (grades, comments): (Vec<_>, Vec<_>) = stdin
        .lines()
        .map_while(Result::ok)
        .map(|line| line.trim().to_owned())
        .map(|line| (line.to_owned(), Grade::from_str(&line).ok()))
        .map(|(line, grade)| {
            grade.map_or_else(
                || (None, Comment::from_str(&line).ok()),
                |g| (Some(g), None),
            )
        })
        .unzip();
    let grades: Vec<Grade> = grades.into_iter().flatten().collect();
    let comments: Vec<Comment> = comments.into_iter().flatten().collect();

    (grades, comments)
}

#[allow(unused)]
fn setup_logging() {
    let log_directory = dirs::data_dir()
        .expect("Unable to get data directory for system!")
        .join("grading");
    std::fs::create_dir_all(&log_directory).expect("Unable to create log directory!");
    let log_file = File::create(log_directory.join("grading.log")).unwrap();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .with_writer(Mutex::new(log_file))
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
