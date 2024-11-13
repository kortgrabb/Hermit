use colored::Colorize;
use std::{env, error::Error, fs, path::PathBuf};

use crate::core::command::CommandContext;
use crate::core::{command::Command, flags::Flags};
use crate::utils;

pub struct ListDirectory;

const ENTRIES_PER_ROW: usize = 4;
const MIN_COLUMN_WIDTH: usize = 30;

impl Command for ListDirectory {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn execute(
        &self,
        args: &[&str],
        flags: &Flags,
        _context: &CommandContext,
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

                print!("{:<MIN_COLUMN_WIDTH$}", colored_name);
                if idx % ENTRIES_PER_ROW == ENTRIES_PER_ROW - 1 {
                    println!();
                }
            }

            idx += 1;
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
