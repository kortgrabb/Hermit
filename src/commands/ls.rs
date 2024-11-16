use colored::Colorize;
use std::{
    env,
    error::Error,
    fs::{self, DirEntry, FileType, Metadata},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::{
    core::{command::Command, command::CommandContext, flags::Flags},
    utils,
};

type DirResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub struct ListDirectory;

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    metadata: Metadata,
    file_type: FileType,
}

impl FileEntry {
    fn new(entry: DirEntry) -> DirResult<Self> {
        Ok(Self {
            name: entry.file_name().to_string_lossy().into_owned(),
            metadata: entry.metadata()?,
            file_type: entry.file_type()?,
        })
    }

    fn format_permissions(&self) -> String {
        let mode = self.metadata.mode();
        let mut perms = String::with_capacity(10);

        // File type
        perms.push(match self.file_type {
            t if t.is_dir() => 'd',
            t if t.is_symlink() => 'l',
            _ => '-',
        });

        // Owner permissions
        perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

        // Group permissions
        perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

        // Others permissions
        perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
        perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
        perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });

        perms
    }

    fn format_long(&self) -> DirResult<String> {
        let perms = self.format_permissions();
        let size = utils::format_size(self.metadata.len());
        let mtime = self
            .metadata
            .modified()?
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let time_str = utils::format_time(mtime);

        Ok(format!(
            "{} {:>4} {:>8} {}",
            perms,
            self.metadata.nlink(),
            size,
            time_str
        ))
    }

    fn colorize(&self) -> String {
        if self.file_type.is_dir() {
            self.name.bright_blue().to_string()
        } else if self.metadata.mode() & 0o111 != 0 {
            self.name.green().to_string()
        } else {
            self.name.clone()
        }
    }
}

#[derive(Debug, Default)]
struct ListOptions {
    show_hidden: bool,
    long_format: bool,
    help: bool,
}

impl ListOptions {
    fn from_flags(flags: &Flags) -> Self {
        Self {
            show_hidden: flags.has_flag('a'),
            long_format: flags.has_flag('l'),
            help: flags.has_flag('?'),
        }
    }
}

impl Command for ListDirectory {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn execute(&self, args: &[&str], flags: &Flags, _context: &CommandContext) -> DirResult<()> {
        let path = self.get_target_path(args)?;
        let options = ListOptions::from_flags(flags);

        if options.help {
            println!("{}", self.extended_description());
            return Ok(());
        }

        let entries = self.read_directory_entries(&path, &options)?;
        self.display_entries(&entries, &options)?;

        if !options.long_format && !entries.is_empty() {
            println!();
        }

        Ok(())
    }

    fn description(&self) -> &'static str {
        "List directory contents"
    }

    fn extended_description(&self) -> &'static str {
        "List directory contents with optional formatting.\n\n\
         Flags:\n\
         -a: Show hidden files\n\
         -l: Use long listing format\n\n\
         If no path is provided, the current directory is used."
    }
}

impl ListDirectory {
    fn get_target_path(&self, args: &[&str]) -> DirResult<PathBuf> {
        Ok(args
            .iter()
            .find(|arg| !arg.starts_with('-'))
            .map(|&arg| Ok(PathBuf::from(arg)))
            .unwrap_or_else(env::current_dir)?)
    }

    fn read_directory_entries(
        &self,
        path: &Path,
        options: &ListOptions,
    ) -> DirResult<Vec<FileEntry>> {
        let mut entries = fs::read_dir(path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let file_name = entry.file_name().to_string_lossy().into_owned();

                if !options.show_hidden && file_name.starts_with('.') {
                    return None;
                }

                FileEntry::new(entry).ok()
            })
            .collect::<Vec<_>>();

        entries.sort_by_cached_key(|entry| entry.name.to_lowercase());
        Ok(entries)
    }

    fn display_entries(&self, entries: &[FileEntry], options: &ListOptions) -> DirResult<()> {
        if options.long_format {
            self.display_long_format(entries)
        } else {
            self.display_grid_format(entries)
        }
    }

    fn display_long_format(&self, entries: &[FileEntry]) -> DirResult<()> {
        for entry in entries {
            let formatted = entry.format_long()?;
            println!("{} {}", formatted, entry.colorize());
        }
        Ok(())
    }

    fn display_grid_format(&self, entries: &[FileEntry]) -> DirResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        // Get terminal width (fallback to 80 if can't determine)
        let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);

        let max_len = entries
            .iter()
            .map(|e| e.name.chars().count()) // chars() allow us to count all unicode characters
            .max()
            .unwrap_or(0);

        let col_width = max_len + 2;
        let num_cols = std::cmp::max(1, term_width / col_width);
        let num_rows = (entries.len() + num_cols - 1) / num_cols;

        for row in 0..num_rows {
            let mut line = String::new();

            for col in 0..num_cols {
                let idx = col * num_rows + row;
                if idx >= entries.len() {
                    break;
                }

                let entry = &entries[idx];
                let colored_name = entry.colorize();

                let display_width = entry.name.chars().count();
                let padding = " ".repeat(col_width.saturating_sub(display_width));

                line.push_str(&colored_name);
                line.push_str(&padding);
            }

            // Trim trailing spaces and print
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                print!("{}", trimmed);
                if row < num_rows - 1 {
                    println!();
                }
            }
        }

        Ok(())
    }
}
