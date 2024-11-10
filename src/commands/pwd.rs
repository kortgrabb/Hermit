use std::{env, error::Error};

use crate::{command::Command, flags::Flags};

pub struct PrintWorkingDirectory;

impl Command for PrintWorkingDirectory {
    fn execute(
        &self,
        _args: &[&str],
        _flags: &Flags,
        _context: &crate::command::CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        println!("{}", env::current_dir()?.display());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "pwd"
    }

    fn description(&self) -> &'static str {
        "Print the current working directory"
    }

    fn extended_description(&self) -> &'static str {
        "Print the current working directory"
    }
}

impl PrintWorkingDirectory {
    pub fn new() -> Self {
        PrintWorkingDirectory
    }
}
