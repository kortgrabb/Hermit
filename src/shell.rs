use colored::Colorize;
use git2::Repository;
use os_release::OsRelease;
use rustyline::{error::ReadlineError, history::FileHistory, Editor};
use std::{
    env,
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use crate::completer::CommandCompleter;
use crate::{builtin::CommandRegistry, external::ExternalCommand, git::GitInfo};

/// Shell represents an interactive command-line interface that handles both built-in
/// and external commands, with support for command history, git integration, and tab completion.
pub struct Shell {
    current_dir: PathBuf,
    editor: Editor<CommandCompleter, FileHistory>,
    git_info: Option<GitInfo>,
}

impl Shell {
    /// Creates a new Shell instance with initialized command completion, history, and git information.
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn Error>>` - A new Shell instance or an error if initialization fails.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut editor = Editor::new()?;

        // Create builtin command instance to get command list
        let builtin = CommandRegistry::setup(env::current_dir()?, editor.history());
        let commands = builtin.get_commands();
        let completer = CommandCompleter::new(commands);

        editor.set_helper(Some(completer));

        let history_path = Self::get_history_file_path();
        let _ = editor.load_history(&history_path);

        let repo = Repository::open(".").ok();
        let git_info = repo.map(GitInfo::new);

        Ok(Self {
            current_dir: env::current_dir()?,
            editor,
            git_info,
        })
    }

    /// Starts the main shell loop, processing user input until exit command is received.
    ///
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - Ok(()) on successful completion or an error if execution fails.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.display_prompt();
            let input = self.read_input();

            if input.is_empty() {
                continue;
            }

            for command in input {
                let parts = self.parse_args(&command);

                let parts: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
                let command = parts.first().unwrap();
                let expanded_args: Vec<String> = parts[1..]
                    .iter()
                    .map(|arg| self.expand_tilde(arg))
                    .collect();
                let args: Vec<&str> = expanded_args.iter().map(|s| s.as_str()).collect();

                match command.to_string().as_str() {
                    "exit" => {
                        let history_path = Self::get_history_file_path();
                        self.editor.save_history(&history_path).unwrap();
                        return Ok(());
                    }
                    _ => match self.execute(command, &args) {
                        Ok(_) => {}
                        Err(e) => eprintln!("{}", e),
                    },
                }

                // Update current directory
                self.current_dir = env::current_dir()?;

                // Update git info
                let repo = Repository::open(".").ok();
                self.git_info = repo.map(GitInfo::new);
            }
        }
    }

    /// Expands the tilde (~) character in paths to the user's home directory.
    ///
    /// # Arguments
    /// * `path` - The path string that may contain a tilde
    fn expand_tilde(&self, path: &str) -> String {
        if path.starts_with("~") {
            if let Ok(home) = env::var("HOME") {
                // Replace only the first occurrence of ~
                return path.replacen("~", &home, 1);
            }
        }

        path.to_string()
    }

    /// Returns the path to the shell history file.
    fn get_history_file_path() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".hermit_history")
    }

    /// Displays the shell prompt with username, distribution, current directory, and git information.
    fn display_prompt(&self) {
        let info = self.get_info();
        print!("{}", info);
        io::stdout().flush().unwrap();
    }

    /// Reads a line of input from the user, handling special cases like Ctrl-C and Ctrl-D.
    ///
    /// # Returns
    /// * `Vec<String>` - A vector of command strings split by semicolons
    fn read_input(&mut self) -> Vec<String> {
        match self.editor.readline(&self.get_info()) {
            Ok(line) => {
                self.editor.add_history_entry(&line).unwrap_or_default();
                // Save history after each command
                self.transform_input(line)
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                vec![]
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                std::process::exit(0);
            }
            Err(_) => vec![],
        }
    }

    /// Generates the shell prompt string with colored components.
    ///
    /// # Returns
    /// * `String` - The formatted prompt string
    fn get_info(&self) -> String {
        let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
        let distro = OsRelease::new()
            .map(|os| os.name)
            .unwrap_or_else(|_| "unknown".to_string());

        let home = env::var("HOME").unwrap_or_default();
        let current_dir = self.current_dir.display().to_string().replace(&home, "~");

        let git_info = match &self.git_info {
            Some(git_info) => format!(" {}", git_info.get_info()),
            None => String::new(),
        };

        format!(
            "{}@{} {}{} > ",
            username.bright_green(),
            distro.green(),
            current_dir.bright_blue(),
            git_info
        )
    }

    /// Transforms raw input by removing comments and splitting into multiple commands.
    ///
    /// # Arguments
    /// * `input` - The raw input string from the user
    fn transform_input(&self, input: String) -> Vec<String> {
        let transformed = match input.split('#').next() {
            Some(cmd) => cmd.trim(),
            None => "",
        };

        // split into multiple commands with ; first, then handle each separately
        let transformed = transformed.split(';');
        transformed
            .map(|cmd| cmd.trim().to_string())
            .filter(|cmd| !cmd.is_empty())
            .collect()
    }

    /// Executes a command with its arguments, handling pipelines, redirections, and built-in commands.
    ///
    /// # Arguments
    /// * `command` - The command to execute
    /// * `args` - The command arguments
    fn execute(&mut self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        if command.is_empty() {
            return Ok(());
        }

        // First check for pipeline
        if let Some(pipeline) = self.try_parse_pipeline(command, args) {
            let external = ExternalCommand::new(self.current_dir.clone());
            return Ok(external.execute_pipeline(&pipeline)?);
        }

        // Then check for redirects
        if let Some((cmd, args, output)) = self.try_parse_redirects(command, args) {
            let external = ExternalCommand::new(self.current_dir.clone());
            return Ok(external.execute_redirect(cmd, &args, &output)?);
        }

        // Finally try builtin or external
        self.execute_command(command, args)
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        match self.execute_builtin(command, args)? {
            true => Ok(()),
            false => self.execute_external(command, args),
        }
    }

    /// Parses a command line into a pipeline of commands if pipe operators are present.
    ///
    /// # Arguments
    /// * `command` - The main command
    /// * `args` - The command arguments
    ///
    /// # Returns
    /// * `Option<Vec<(&str, Vec<&str>)>>` - A vector of command and arguments tuples if pipeline exists
    fn try_parse_pipeline<'a>(
        &self,
        command: &'a str,
        args: &'a [&'a str],
    ) -> Option<Vec<(&'a str, Vec<&'a str>)>> {
        let mut commands = vec![command];
        commands.extend(args);

        if !commands.contains(&"|") {
            return None;
        }

        let mut pipeline = Vec::new();
        let mut current_cmd = Vec::new();

        for arg in commands {
            if arg == "|" {
                if !current_cmd.is_empty() {
                    pipeline.push((current_cmd[0], current_cmd[1..].to_vec()));
                    current_cmd.clear();
                }
            } else {
                current_cmd.push(arg);
            }
        }

        if !current_cmd.is_empty() {
            pipeline.push((current_cmd[0], current_cmd[1..].to_vec()));
        }

        Some(pipeline)
    }

    /// Parses command line for output redirection.
    ///
    /// # Arguments
    /// * `command` - The main command
    /// * `args` - The command arguments
    ///
    /// # Returns
    /// * `Option<(&str, Vec<&str>, String)>` - Tuple of command, args, and output file if redirection exists
    fn try_parse_redirects<'a>(
        &self,
        command: &'a str,
        args: &'a [&'a str],
    ) -> Option<(&'a str, Vec<&'a str>, String)> {
        let mut commands = vec![command];
        commands.extend(args);

        if let Some(pos) = commands.iter().position(|&x| x == ">") {
            if pos + 1 < commands.len() {
                let output = commands[pos + 1].to_string();
                let command = commands[0];
                let args = if pos > 1 {
                    commands[1..pos].to_vec()
                } else {
                    Vec::new()
                };
                return Some((command, args, output));
            }
        }

        None
    }

    fn execute_builtin(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        let mut builtin = CommandRegistry::setup(self.current_dir.clone(), self.editor.history());
        builtin.execute(command, args)
    }

    fn execute_external(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        let external = ExternalCommand::new(self.current_dir.clone());
        match external.execute(command, args) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(format!("command not found: {}", command).into())
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Parses input string into command arguments, handling quoted strings.
    ///
    /// # Arguments
    /// * `input` - The input string to parse
    ///
    /// # Returns
    /// * `Vec<String>` - The parsed command arguments
    pub fn parse_args(&self, input: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;

        for c in input.chars().peekable() {
            match c {
                '"' => in_quotes = !in_quotes,
                ' ' if !in_quotes => {
                    if !current_part.is_empty() {
                        parts.push(current_part.clone());
                        current_part.clear();
                    }
                }
                _ => current_part.push(c),
            }
        }

        if !current_part.is_empty() {
            parts.push(current_part);
        }

        parts
    }
}

#[cfg(test)]
mod tests {
    use rustyline::history::History;

    use super::*;

    #[test]
    fn test_shell_new() {
        let shell = Shell::new().unwrap();
        assert_eq!(shell.current_dir, env::current_dir().unwrap());

        // Check if history file is created in the right location
        assert!(Shell::get_history_file_path().exists());
    }

    // Stoopid test but whatever
    #[test]
    fn test_shell_history() {
        let mut shell = Shell::new().unwrap();
        let _ = shell.editor.add_history_entry("echo hello");
        assert_eq!(shell.editor.history().len(), 1);

        let _ = shell.editor.clear_history();

        assert_eq!(shell.editor.history().len(), 0);
    }

    #[test]
    fn test_shell_current_dir() {
        let shell = Shell::new().unwrap();
        assert!(shell.current_dir.exists());
        assert!(shell.current_dir.is_absolute());
    }

    #[test]
    fn test_execute_builtin_empty() {
        let mut shell = Shell::new().unwrap();
        let result = shell.execute_builtin("", &[]).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_execute_external_empty() {
        let shell = Shell::new().unwrap();
        let result = shell.execute_external("", &[]);
        assert!(result.is_err());
    }
}
