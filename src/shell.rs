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

type ShellResult<T> = Result<T, Box<dyn Error>>;

/// Shell represents an interactive command-line interface that handles both built-in
/// and external commands, with support for command history, git integration, and tab completion.
pub struct Shell {
    current_dir: PathBuf,
    editor: Editor<CommandCompleter, FileHistory>,
    git_info: Option<GitInfo>,
    history_path: PathBuf,
}

impl Shell {
    /// Creates a new Shell instance with initialized command completion, history, and git information.
    pub fn new() -> ShellResult<Self> {
        let mut editor = Editor::new()?;
        let current_dir = env::current_dir()?;
        let history_path = Self::get_history_file_path();

        Self::setup_editor(&mut editor, &current_dir, &history_path)?;

        let repo = Repository::discover(&current_dir).ok();
        let git_info = repo.map(GitInfo::new);

        Ok(Self {
            current_dir,
            editor,
            git_info,
            history_path,
        })
    }

    fn setup_editor(
        editor: &mut Editor<CommandCompleter, FileHistory>,
        current_dir: &PathBuf,
        history_path: &PathBuf,
    ) -> ShellResult<()> {
        let builtin = CommandRegistry::setup(current_dir.clone(), editor.history());
        let commands = builtin.get_commands();
        let completer = CommandCompleter::new(commands);

        editor.set_helper(Some(completer));
        editor.load_history(history_path)?;

        Ok(())
    }

    /// Starts the main shell loop, processing user input until exit command is received.
    pub fn run(&mut self) -> ShellResult<()> {
        while let Some(input) = self.read_input() {
            if input.is_empty() {
                continue;
            }

            self.process_commands(&input)?;
            self.update_state()?;
        }

        self.editor.save_history(&self.history_path)?;
        Ok(())
    }

    fn process_commands(&mut self, commands: &[String]) -> ShellResult<()> {
        for command in commands {
            let parts = self.parse_args(command);
            if parts.is_empty() {
                continue;
            }

            let (cmd, args) = parts.split_first().unwrap();
            let expanded_args: Vec<String> =
                args.iter().map(|arg| self.expand_tilde(arg)).collect();

            if *cmd == "exit" {
                return self.handle_exit();
            }

            if let Err(e) = self.execute(cmd, &expanded_args) {
                eprintln!("Error: {}", e);
            }
        }
        Ok(())
    }

    fn handle_exit(&mut self) -> ShellResult<()> {
        self.editor.save_history(&self.history_path)?;
        std::process::exit(0);
    }

    fn update_state(&mut self) -> ShellResult<()> {
        self.current_dir = env::current_dir()?;
        self.git_info = Repository::discover(&self.current_dir)
            .ok()
            .map(GitInfo::new);
        Ok(())
    }

    /// Expands the tilde (~) character in paths to the user's home directory.
    fn expand_tilde(&self, path: &str) -> String {
        if path.starts_with('~') {
            if let Ok(home) = env::var("HOME") {
                return path.replacen('~', &home, 1);
            }
        }
        path.to_string()
    }

    /// Returns the path to the shell history file.
    fn get_history_file_path() -> PathBuf {
        env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".hermit_history")
    }

    /// Displays the shell prompt with username, distribution, current directory, and git information.
    fn display_prompt(&self) {
        print!("{}", self.get_prompt_info());
        io::stdout().flush().unwrap();
    }

    /// Reads a line of input from the user, handling special cases like Ctrl-C and Ctrl-D.
    fn read_input(&mut self) -> Option<Vec<String>> {
        self.display_prompt();

        match self.editor.readline(&self.get_prompt_info()) {
            Ok(line) => {
                self.editor.add_history_entry(&line).ok();
                Some(self.transform_input(line))
            }
            Err(ReadlineError::Interrupted) => Some(vec![]),
            Err(ReadlineError::Eof) => None,
            Err(_) => Some(vec![]),
        }
    }

    /// Generates the shell prompt string with colored components.
    fn get_prompt_info(&self) -> String {
        let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
        let distro = OsRelease::new()
            .map(|os| os.name)
            .unwrap_or_else(|_| "unknown".to_string());

        let current_dir = self.format_current_dir();
        let git_info = self
            .git_info
            .as_ref()
            .map(|git| format!(" {}", git.get_info()))
            .unwrap_or_default();

        format!(
            "{}@{} {}{} > ",
            username.bright_green(),
            distro.green(),
            current_dir.bright_blue(),
            git_info
        )
    }

    fn format_current_dir(&self) -> String {
        if let Ok(home) = env::var("HOME") {
            self.current_dir.display().to_string().replace(&home, "~")
        } else {
            self.current_dir.display().to_string()
        }
    }

    /// Transforms raw input by removing comments and splitting into multiple commands.
    fn transform_input(&self, input: String) -> Vec<String> {
        input
            .split('#')
            .next()
            .unwrap_or("")
            .split(';')
            .map(str::trim)
            .filter(|cmd| !cmd.is_empty())
            .map(String::from)
            .collect()
    }

    /// Executes a command with its arguments, handling pipelines, redirections, and built-in commands.
    fn execute(&mut self, command: &str, args: &[String]) -> ShellResult<()> {
        if command.is_empty() {
            return Ok(());
        }

        let args: Vec<&str> = args.iter().map(String::as_str).collect();

        if let Some(pipeline) = self.try_parse_pipeline(command, &args) {
            return self.execute_pipeline(&pipeline);
        }

        if let Some((cmd, args, output)) = self.try_parse_redirects(command, &args) {
            return self.execute_redirect(cmd, &args, &output);
        }

        self.execute_command(command, &args)
    }

    fn execute_pipeline(&self, pipeline: &[(&str, Vec<&str>)]) -> ShellResult<()> {
        let external = ExternalCommand::new(self.current_dir.clone());
        Ok(external.execute_pipeline(pipeline)?)
    }

    fn execute_redirect(&self, cmd: &str, args: &[&str], output: &str) -> ShellResult<()> {
        let external = ExternalCommand::new(self.current_dir.clone());
        Ok(external.execute_redirect(cmd, args, output)?)
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> ShellResult<()> {
        if self.execute_builtin(command, args)? {
            Ok(())
        } else {
            self.execute_external(command, args)
        }
    }

    /// Parses command line for output redirection.
    fn try_parse_redirects<'a>(
        &self,
        command: &'a str,
        args: &'a [&'a str],
    ) -> Option<(&'a str, Vec<&'a str>, String)> {
        let mut commands = std::iter::once(command)
            .chain(args.iter().copied())
            .collect::<Vec<_>>();

        if let Some(pos) = commands.iter().position(|&x| x == ">") {
            if pos + 1 < commands.len() {
                let output = commands[pos + 1].to_string();
                let command = commands[0];
                let args = commands[1..pos].to_vec();
                return Some((command, args, output));
            }
        }
        None
    }

    /// Parses a command line into a pipeline of commands if pipe operators are present.
    fn try_parse_pipeline<'a>(
        &self,
        command: &'a str,
        args: &'a [&'a str],
    ) -> Option<Vec<(&'a str, Vec<&'a str>)>> {
        let commands = std::iter::once(command)
            .chain(args.iter().copied())
            .collect::<Vec<_>>();

        if !commands.contains(&"|") {
            return None;
        }

        let mut pipeline = Vec::new();
        let mut current_cmd = Vec::new();

        for &arg in &commands {
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

    fn execute_builtin(&mut self, command: &str, args: &[&str]) -> ShellResult<bool> {
        let mut builtin = CommandRegistry::setup(self.current_dir.clone(), self.editor.history());
        builtin.execute(command, args)
    }

    fn execute_external(&self, command: &str, args: &[&str]) -> ShellResult<()> {
        let external = ExternalCommand::new(self.current_dir.clone());
        external.execute(command, args).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                format!("command not found: {}", command).into()
            } else {
                e.into()
            }
        })
    }

    /// Parses input string into command arguments, handling quoted strings.
    pub fn parse_args(&self, input: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut in_quotes = false;

        for c in input.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ' ' if !in_quotes => {
                    if !current_part.is_empty() {
                        parts.push(std::mem::take(&mut current_part));
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
    use super::*;

    #[test]
    fn test_shell_initialization() -> ShellResult<()> {
        let shell = Shell::new()?;
        assert!(shell.current_dir.is_absolute());
        assert!(shell.history_path.ends_with(".hermit_history"));
        Ok(())
    }

    #[test]
    fn test_parse_args() {
        let shell = Shell::new().unwrap();

        assert_eq!(
            shell.parse_args(r#"command "quoted arg" unquoted"#),
            vec!["command", "quoted arg", "unquoted"]
        );

        assert_eq!(
            shell.parse_args("command with multiple    spaces"),
            vec!["command", "with", "multiple", "spaces"]
        );
    }

    #[test]
    fn test_expand_tilde() {
        let shell = Shell::new().unwrap();
        let home = env::var("HOME").unwrap();

        assert_eq!(shell.expand_tilde("~/test"), format!("{}/test", home));
        assert_eq!(shell.expand_tilde("/absolute/path"), "/absolute/path");
    }

    #[test]
    fn test_transform_input() {
        let shell = Shell::new().unwrap();

        assert_eq!(
            shell.transform_input("cmd1; cmd2 # comment".to_string()),
            vec!["cmd1", "cmd2"]
        );

        assert_eq!(
            shell.transform_input("cmd1;; cmd2".to_string()),
            vec!["cmd1", "cmd2"]
        );
    }
}
