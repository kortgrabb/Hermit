use crate::{command::Command, flags::Flags};
use std::error::Error;

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

    fn execute(&self, args: &[&str], _flags: &Flags) -> Result<(), Box<dyn Error>> {
        println!("{}", args.join(" "));
        Ok(())
    }
}
