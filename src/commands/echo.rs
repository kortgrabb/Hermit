use crate::core::{
    command::{Command, CommandContext},
    flags::Flags,
};
use std::error::Error;

#[derive(Clone)]
pub struct Echo;

impl Command for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Prints the given arguments"
    }

    fn extended_description(&self) -> &'static str {
        "Prints the given arguments"
    }

    fn execute(
        &self,
        args: &[&str],
        _flags: &Flags,
        _context: &CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        println!("{}", args.join(" "));
        Ok(())
    }
}
