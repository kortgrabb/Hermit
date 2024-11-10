use crate::{
    command::{Command, CommandContext},
    flags::Flags,
};
use std::error::Error;

#[derive(Clone)]
pub struct History;

impl Command for History {
    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Display command history"
    }

    fn extended_description(&self) -> &'static str {
        "Display the command history with line numbers"
    }

    fn execute(
        &self,
        _args: &[&str],
        _flags: &Flags,
        context: &CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        for (i, cmd) in context.history.iter().enumerate() {
            println!("{:5} {}", i + 1, cmd);
        }
        Ok(())
    }
}
