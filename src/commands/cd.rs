use crate::{command::Command, flags::Flags};
use std::{env, error::Error};

#[derive(Clone)]
pub struct ChangeDirectory;

impl Command for ChangeDirectory {
    fn execute(
        &self,
        args: &[&str],
        _flags: &Flags,
        _context: &crate::command::CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        let new_dir = args.first().map_or_else(
            || Ok::<String, Box<dyn Error>>(env::var("HOME")?),
            |path| Ok::<String, Box<dyn Error>>(path.to_string()),
        )?;
        env::set_current_dir(new_dir)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "cd"
    }

    fn description(&self) -> &'static str {
        "Change the current working directory"
    }

    fn extended_description(&self) -> &'static str {
        "Change the current working directory. If no directory is specified, change to the user's home directory."
    }
}
