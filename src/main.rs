mod config;
mod datastore;
mod args;
mod model;

use std::io;
use std::io::{Read, stdout, Write};
use colored::Colorize;
use log::{debug, error, info, SetLoggerError, trace};
use rustyline::{Config, DefaultEditor, Editor};
use rustyline::error::ReadlineError;
use serde::{Deserialize, Serialize};
use crate::args::Args;
use crate::config::ApplicationConfig;
use clap::Parser;
use crossterm::{cursor, execute};
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType};
use rusqlite::Connection;
use crate::datastore::DataStore;
use crate::model::{Question, QuestionAnswers, QuestionId};

const YES: &str = "y";
const NO: &str = "n";

fn setup_logging(args: &Args) -> Result<(), SetLoggerError> {
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
}

fn read_yes_no(prompt: &str) -> anyhow::Result<bool> {
    loop {
        let mut rl = DefaultEditor::new()?;
        let response = rl.readline(prompt)?.to_lowercase();

        if response == YES {
            return Ok(true);
        }
        if response == NO {
            return Ok(false);
        }
    }
}

fn ask(mut connection: &mut Connection, question_answers: &QuestionAnswers) -> anyhow::Result<()> {
    let _ = execute!(stdout(), Clear(ClearType::All ), cursor::MoveTo(0, 0));
    let question = &question_answers.question;
    let header = format!(
        "# Nummer: {} ({})\t{} mal korrekt von {}",
        Stylize::blue(question.id.to_string()),
        question.topic,
        question_answers.count_correct().to_string().green(),
        Stylize::yellow(question_answers.answers.len().to_string())
    );
    println!("{}", header.bold());
    println!("{}\n", question.question);
    io::stdout().flush()?;

    let mut rl = DefaultEditor::new()?;
    let response = rl.readline(">> ")?;

    println!("\n{} {}\n", Stylize::green("Musterlösung:"), question.answer);
    let prompt = format!("War die Antwort korrekt? ({}/{}): ", YES, NO);
    let correct = read_yes_no(&prompt)?;

    connection.insert_answer(&question.get_id(), &response, correct)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    setup_logging(&args).expect("Failed to setup logging!");
    let config = ApplicationConfig::default();

    debug!("Database: {:?}", config.project_dirs.database_path);

    let mut con = Connection::connect_database(&config).unwrap();


    loop {
        let qid = &con.view_candidates(20, 1)?[0];
        let question = con.view_question_answers(qid)?;
        match ask(&mut con, &question) {
            Ok(_) => {}
            Err(_) => {
                return Ok(());
            }
        }
    }
    Ok(())
}
