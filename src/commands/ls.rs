use std::{env, error::Error, fs, path::PathBuf};

use colored::Colorize;

use crate::{
    command::Command,
    flags::{self, Flags},
    utils,
};

pub struct ListDirectory;

impl ListDirectory {
    pub fn new() -> Self {
        ListDirectory
    }
}

impl Command for ListDirectory {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn execute(
        &self,
        args: &[&str],
        flags: &Flags,
        _context: &crate::command::CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        let path = args
            .iter()
            .find(|arg| !arg.starts_with('-'))
            .map_or_else(env::current_dir, |arg| Ok(PathBuf::from(arg)))?;

        let entries = fs::read_dir(path)?;

        let show_hidden = flags.has_flag('a');
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if !show_hidden && file_name.starts_with('.') {
                continue;
            }

            if entry.file_type()?.is_dir() {
                println!("{}", file_name.on_bright_black().white());
            } else {
                println!("{}", file_name);
            }
        }

        Ok(())
    }

    fn description(&self) -> &'static str {
        "List directory contents"
    }

    fn extended_description(&self) -> &'static str {
        "List directory contents. If no path is provided, the current directory is used."
    }
}
