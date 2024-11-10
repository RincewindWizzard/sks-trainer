BEGIN;

CREATE TABLE IF NOT EXISTS questions (
    id          INTEGER,
    topic       TEXT NOT NULL,
    question    TEXT,
    answer      TEXT,
    PRIMARY KEY (id, topic)
);

CREATE TABLE IF NOT EXISTS answers (
    question_id INTEGER,
    topic       TEXT NOT NULL,
    timestamp   TEXT NOT NULL,
    correct     BOOLEAN NOT NULL,
    content     TEXT,
    PRIMARY KEY (question_id, topic, timestamp)
);

DROP VIEW IF EXISTS answer_sheet;
CREATE VIEW answer_sheet AS
SELECT
    questions.id as question_id,
    questions.topic as topic,
    COALESCE(answers.timestamp, '1970-01-01') as timestamp,
    COALESCE(answers.correct, 0) as correct
FROM questions
LEFT JOIN answers
ON questions.id = answers.question_id AND questions.topic = answers.topic
ORDER BY timestamp ASC;

DROP VIEW IF EXISTS statistics;
CREATE VIEW statistics AS
SELECT
    question_id,
    topic,
    COUNT(*) AS response_count,
    COUNT(CASE WHEN correct = TRUE THEN 1 END) AS correct_count,
    MAX(timestamp) as last_update,
    COALESCE(
        (SELECT correct
        FROM answers
        WHERE answers.question_id = answer_sheet.question_id AND answers.topic = answer_sheet.topic
        ORDER BY timestamp DESC
        LIMIT 1),
        0
    ) as last_correct
FROM
    answer_sheet
GROUP BY
    question_id,
    topic;

DROP VIEW IF EXISTS candidates;
CREATE VIEW candidates AS
SELECT *
FROM statistics
WHERE last_correct = 0
ORDER BY last_update DESC;

COMMIT;