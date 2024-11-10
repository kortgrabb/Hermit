use std::{env, error::Error, fs, path::PathBuf};

use colored::Colorize;

use crate::{
    command::Command,
    flags::{self, Flags},
    utils,
};

#[derive(Clone)]
pub struct ListDirectory;

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
        let show_long = flags.has_flag('l');
        let mut idx = 0;
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if !show_hidden && file_name.starts_with('.') {
                continue;
            }

            if show_long {
                let metadata = entry.metadata()?;
                let file_type = metadata.file_type();
                let file_type = if file_type.is_dir() {
                    "d".blue()
                } else {
                    "f".normal()
                };

                let file_name = utils::colorize_file_name(&file_name, &metadata);

                let size = metadata.len();

                println!("{:6} {} {}", size, file_type, file_name);
            } else {
                let metadata = entry.metadata()?;
                let colored_name = utils::colorize_file_name(&file_name, &metadata);

                println!("{} ", colored_name);
            }

            idx += 1;
        }

        if !show_long {
            println!();
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
