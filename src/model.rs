use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use stderrlog::Timestamp;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuestionId {
    pub id: u64,
    pub topic: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Progress {
    pub topic: String,
    pub nominator: u64,
    pub denominator: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Question {
    pub id: u64,
    pub question: String,
    pub answer: String,
    pub topic: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Answer {
    pub question_id: QuestionId,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub correct: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuestionAnswers {
    pub question: Question,
    pub answers: Vec<Answer>,
}


impl QuestionId {
    pub(crate) fn new(id: u64, topic: &str) -> QuestionId {
        QuestionId {
            id,
            topic: topic.to_string(),
        }
    }
}

impl Progress {
    pub(crate) fn new(topic: String, nominator: u64, denominator: u64) -> Progress {
        Progress {
            topic,
            nominator,
            denominator,
        }
    }

    pub fn percentage(&self) -> f64 {
        (100.0 * (self.nominator) as f64) / (self.denominator) as f64
    }
}

impl Question {
    pub(crate) fn get_id(&self) -> QuestionId {
        QuestionId::new(self.id, &self.topic)
    }
}

impl Answer {
    pub fn new(question_id: &QuestionId, timestamp: DateTime<Utc>, content: &str, correct: bool) -> Answer {
        Answer {
            question_id: question_id.clone(),
            timestamp,
            content: content.to_string(),
            correct,
        }
    }
}

impl QuestionAnswers {
    pub(crate) fn new(question: &Question, answers: &[Answer]) -> QuestionAnswers {
        QuestionAnswers {
            question: question.clone(),
            answers: Vec::from(answers.clone()),
        }
    }

    pub fn count_correct(&self) -> usize {
        self.answers.iter().filter(|rp| rp.correct).count()
    }
}