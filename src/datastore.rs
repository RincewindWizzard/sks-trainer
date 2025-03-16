use std::cmp::max;
use anyhow::Context;
use chrono::{DateTime, Utc};
use log::debug;
use thiserror::Error;
use rusqlite::{Connection, Params, params, Row};
use stderrlog::Timestamp;
use crate::config::ApplicationConfig;
use crate::datastore::DataStoreError::FileSystem;
use crate::model::{Answer, QuestionId, QuestionAnswers, Progress};
use crate::model::Question;


#[derive(Error, Debug)]
enum DataStoreError {
    #[error("Cannot access database file: {0}")]
    FileSystem(String)
}


pub trait DataStore {
    fn connect_database(config: &ApplicationConfig) -> anyhow::Result<Connection>;
    fn run_migrations(&mut self) -> anyhow::Result<()>;


    /// Executes a select statement and converts all Rows into T using the function from_row
    fn view_query<T, F, P>(&mut self, sql: &str, params: P, fro_row: F) -> anyhow::Result<Vec<T>>
        where
            F: FnMut(&Row<'_>) -> rusqlite::Result<T, rusqlite::Error>,
            P: Params;

    /// Inserts a Vector of Documents into a table using the sql statement and the function to_row
    fn insert_query<'a, T, F, P>(&mut self, sql: &str, docs: &'a [T], to_row: F) -> anyhow::Result<()>
        where
            F: Fn(&'a T) -> P,
            P: Params;

    fn insert_answer(&mut self, question: &QuestionId, content: &str, correct: bool) -> anyhow::Result<()>;

    fn insert_questions(&mut self, questions: &[Question]) -> anyhow::Result<()>;

    fn view_answers(&mut self, question_id: &QuestionId) -> anyhow::Result<Vec<Answer>>;
    fn view_question(&mut self, question_id: &QuestionId) -> anyhow::Result<Question>;

    fn view_candidates(&mut self, skip: usize, count: usize) -> anyhow::Result<Vec<QuestionId>>;

    fn view_question_answers(&mut self, question_id: &QuestionId) -> anyhow::Result<QuestionAnswers>;
    fn view_progress(&mut self) -> anyhow::Result<Vec<Progress>>;
}

impl DataStore for Connection {
    fn connect_database(config: &ApplicationConfig) -> anyhow::Result<Connection> {
        let database_path = &config.project_dirs.database_path;

        let dir_path = database_path
            .parent()
            .ok_or(FileSystem(format!("Could not create database path: {}", database_path.display())))?;

        std::fs::create_dir_all(dir_path)?;
        let new_database = !database_path.exists();

        let mut connection = Connection::open(database_path.clone())?;
        connection.run_migrations()?;

        if new_database {
            let questions: Vec<Question> = serde_json::from_str(include_str!("questions.json"))?;
            connection.insert_questions(&questions)?;
        }

        debug!("Succesfully setup database at {}", database_path.display());

        Ok(connection)
    }


    fn run_migrations(&mut self) -> anyhow::Result<()> {
        // running migrations
        let sql = include_str!("sql/01_schema.sql");
        debug!("Executing sql: {sql}");

        Ok(self.execute_batch(sql)?)
    }

    fn insert_answer(&mut self, question_id: &QuestionId, content: &str, correct: bool) -> anyhow::Result<()> {
        let answer = Answer::new(question_id, Utc::now(), content, correct);

        self.execute(
            "INSERT INTO answers (question_id, topic, timestamp, content, correct) VALUES (?, ?, ?, ?, ?);",
            params![
                &answer.question_id.id,
                &answer.question_id.topic,
                &answer.timestamp.to_rfc3339(),
                &answer.content,
                &answer.correct
            ],
        )?;

        Ok(())
    }


    fn insert_questions(&mut self, questions: &[Question]) -> anyhow::Result<()> {
        self.insert_query(
            "INSERT INTO questions (id, topic, question, answer) VALUES (?, ?, ?, ?);",
            questions,
            |question| (
                &question.id,
                &question.topic,
                &question.question,
                &question.answer
            ),
        )?;
        Ok(())
    }


    fn view_query<T, F, P>(&mut self, sql: &str, params: P, from_row: F) -> anyhow::Result<Vec<T>>
        where
            F: FnMut(&Row<'_>) -> rusqlite::Result<T, rusqlite::Error>,
            P: Params
    {
        let tx = self.transaction()?;

        let result = tx
            .prepare(sql)?
            .query_map(params, from_row)?
            .filter_map(|x| x.ok())
            .collect();

        tx.commit()?;
        Ok(result)
    }

    fn insert_query<'a, T, F, P>(&mut self, sql: &str, docs: &'a [T], to_row: F) -> anyhow::Result<()>
        where
            F: Fn(&'a T) -> P,
            P: Params
    {
        let tx = self.transaction()?;
        {
            let mut stmt = tx.prepare(sql)?;

            let _errors: Vec<rusqlite::Error> = docs
                .iter()
                .map(to_row)
                .map(|params| stmt.execute(params))
                .filter_map(|result| result.err())
                .collect();
        }
        tx.commit()?;
        Ok(())
    }

    fn view_question_answers(&mut self, question_id: &QuestionId) -> anyhow::Result<QuestionAnswers> {
        Ok(QuestionAnswers {
            question: self.view_question(question_id)?,
            answers: self.view_answers(question_id)?,
        })
    }

    fn view_answers(&mut self, question_id: &QuestionId) -> anyhow::Result<Vec<Answer>> {
        let answers = self.view_query(
            "SELECT timestamp, content, correct FROM answers WHERE question_id = ? AND topic = ?;",
            params![question_id.id, question_id.topic],
            |row: &Row| {
                // Parse timestamp as String and convert it to DateTime<Utc>
                let timestamp_str: String = row.get("timestamp")?;
                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .unwrap()
                    .with_timezone(&Utc);

                // Get content and correct values with expected types
                let content: String = row.get("content")?;
                let correct: bool = row.get("correct")?;
                let answer = Answer::new(
                    question_id,
                    timestamp,
                    &content,
                    correct,
                );
                Ok(answer)
            },
        )?;
        Ok(answers)
    }

    fn view_question(&mut self, question_id: &QuestionId) -> anyhow::Result<Question> {
        let questions = self.view_query(
            "SELECT question, answer from questions WHERE id = ? AND topic = ? LIMIT 1;",
            params![question_id.id, question_id.topic],
            |row| Ok(
                Question {
                    id: question_id.id,
                    question: row.get("question")?,
                    answer: row.get("answer")?,
                    topic: question_id.topic.clone(),
                }
            ),
        )?;
        let question = questions[0].clone();
        Ok(question)
    }

    fn view_candidates(&mut self, skip: usize, count: usize) -> anyhow::Result<Vec<QuestionId>> {
        let questions = self.view_query(
            "SELECT question_id, topic from candidates LIMIT ?, ?;",
            params![skip, count],
            |row| {
                let topic: String = row.get("topic")?;

                Ok(
                    QuestionId::new(
                        row.get("question_id")?,
                        &topic,
                    )
                )
            },
        )?;
        Ok(questions)
    }

    fn view_progress(&mut self) -> anyhow::Result<Vec<Progress>> {
        let questions = self.view_query(
            "SELECT topic, SUM(CASE WHEN correct_count > 0 THEN 1 ELSE 0 END) as correct, COUNT(question_id) as alle FROM statistics GROUP BY topic;",
            params![],
            |row| {
                let topic: String = row.get("topic")?;
                let correct: u64 = row.get("correct")?;
                let alle: u64 = row.get("alle").unwrap();



                Ok(
                    Progress::new(
                        topic,
                        correct,
                        alle,
                    )
                )
            },
        )?;
        Ok(questions)
    }
}