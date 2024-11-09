use crate::command::Command;
use std::{env, error::Error};

pub struct ChangeDirectory;

impl Command for ChangeDirectory {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn description(&self) -> &'static str {
        "Change the current working directory"
    }

    fn extended_description(&self) -> &'static str {
        "Change the current working directory. If no directory is specified, change to the user's home directory."
    }

    fn execute(&self, args: &[&str]) -> Result<(), Box<dyn Error>> {
        let path = if args.is_empty() {
            dirs::home_dir().ok_or("Could not determine home directory")?
        } else {
            std::path::PathBuf::from(args[0])
        };

        env::set_current_dir(path)?;

        Ok(())
    }
}
