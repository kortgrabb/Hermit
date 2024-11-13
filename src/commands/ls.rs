use colored::Colorize;
use std::{
    env,
    error::Error,
    fmt::Write,
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
pub struct ListDirectory {
    config: ListConfig,
}

impl ListDirectory {
    pub fn new(config: ListConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Clone)]
pub struct ListConfig {
    entries_per_row: usize,
    min_column_width: usize,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            entries_per_row: 4,
            min_column_width: 30,
        }
    }
}

#[derive(Debug)]
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
}

impl ListOptions {
    fn from_flags(flags: &Flags) -> Self {
        Self {
            show_hidden: flags.has_flag('a'),
            long_format: flags.has_flag('l'),
        }
    }
}

impl Default for ListDirectory {
    fn default() -> Self {
        Self {
            config: ListConfig {
                entries_per_row: 4,
                min_column_width: 30,
            },
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
            .unwrap_or_else(|| env::current_dir())?)
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

        let max_name_width = entries
            .iter()
            .map(|e| e.name.len())
            .max()
            .unwrap_or(0)
            .max(self.config.min_column_width);

        let term_width = utils::term_width();
        let num_columns = (term_width / max_name_width).max(1);
        let mut output = String::new();

        for (i, entry) in entries.iter().enumerate() {
            write!(
                output,
                "{:width$}",
                entry.colorize(),
                width = if (i + 1) % num_columns == 0 {
                    0
                } else {
                    max_name_width
                }
            )?;

            if (i + 1) % num_columns == 0 {
                writeln!(output)?;
            }
        }

        print!("{}", output);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> DirResult<TempDir> {
        let dir = TempDir::new()?;
        fs::write(dir.path().join("file1.txt"), "content")?;
        fs::write(dir.path().join("file2.txt"), "content")?;
        fs::create_dir(dir.path().join("dir1"))?;
        Ok(dir)
    }

    #[test]
    fn test_list_directory() -> DirResult<()> {
        let dir = setup_test_dir()?;
        let ls = ListDirectory::default();
        let flags = Flags::new(&[])?;
        let context = CommandContext::default();

        ls.execute(&[dir.path().to_str().unwrap()], &flags, &context)?;
        Ok(())
    }

    #[test]
    fn test_hidden_files() -> DirResult<()> {
        let dir = setup_test_dir()?;
        fs::write(dir.path().join(".hidden"), "content")?;

        let ls = ListDirectory::default();
        let mut flags = Flags::new(&[])?;
        flags.add_flag('a');

        let context = CommandContext::default();
        ls.execute(&[dir.path().to_str().unwrap()], &flags, &context)?;
        Ok(())
    }

    #[test]
    fn test_long_format() -> DirResult<()> {
        let dir = setup_test_dir()?;
        let ls = ListDirectory::default();
        let mut flags = Flags::new(&[])?;
        flags.add_flag('l');

        let context = CommandContext::default();
        ls.execute(&[dir.path().to_str().unwrap()], &flags, &context)?;
        Ok(())
    }
}
