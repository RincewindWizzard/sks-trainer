use std::io;
use std::io::Write;
use rustyline::Editor;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Question {
    id: u32,
    question: String,
    answer: String,
}

impl Question {
    fn ask(&self) {
        println!("# Nummer: {}", self.id);
        println!("{}", self.question);
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim_end();

        println!("-------");
        println!("{}\n", self.answer);

        let mut rl = Editor::<()>::new().expect("Failed to create readline editor");


        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str()); // Speichere die Eingabe in den Verlauf
                println!("You entered: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C received. Exiting...");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D received. Exiting...");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let questions: Vec<Question> = serde_json::from_str(include_str!("questions.json"))?;
    questions[0].ask();
    Ok(())
}
