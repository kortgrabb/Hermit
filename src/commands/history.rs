use crate::command::Command;
use std::error::Error;

pub struct History {
    pub history: Vec<String>,
}

impl History {
    pub fn new(history: &[String]) -> Self {
        Self {
            history: history.to_vec(),
        }
    }
}

impl Command for History {
    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Prints the history of commands"
    }

    fn extended_description(&self) -> &'static str {
        "Prints the history of commands"
    }

    fn execute(&self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        for (i, command) in self.history.iter().enumerate() {
            println!("{}: {}", i + 1, command);
        }
        Ok(())
    }
}
