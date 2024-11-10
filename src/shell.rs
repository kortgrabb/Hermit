use rustyline::{error::ReadlineError, history::FileHistory};
use std::{
    env,
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use crate::{builtin::BuiltinCommand, external::ExternalCommand};
use colored::Colorize;
use os_release::OsRelease;
use rustyline::Editor;

#[derive(Debug)]
pub struct Shell {
    current_dir: PathBuf,
    editor: Editor<(), FileHistory>,
}

impl Shell {
    fn get_history_file_path() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".hermit_history")
    }

    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut editor = Editor::new()?;
        let history_path = Self::get_history_file_path();
        let _ = editor.load_history(&history_path);

        Ok(Shell {
            current_dir: env::current_dir()?,
            editor,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            self.display_prompt();
            let input = self.read_input();

            if input.is_empty() {
                continue;
            }

            for command in input {
                let parts: Vec<&str> = command.split_whitespace().collect();
                let command = parts.first().unwrap();
                let args = &parts[1..];

                match command.to_string().as_str() {
                    "exit" => {
                        let history_path = Self::get_history_file_path();
                        self.editor.save_history(&history_path).unwrap();
                        return Ok(());
                    }
                    _ => match self.execute(command, args) {
                        Ok(_) => {}
                        Err(e) => eprintln!("{}", e),
                    },
                }

                // Update current directory
                self.current_dir = env::current_dir()?;
            }
        }
    }

    // TODO: colored, git integration
    fn display_prompt(&self) {
        let prompt = self.get_prompt();
        print!("{}", prompt);
        io::stdout().flush().unwrap_or_default();
    }

    fn read_input(&mut self) -> Vec<String> {
        match self.editor.readline(&self.get_prompt()) {
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

    // Helper method to generate prompt string
    fn get_prompt(&self) -> String {
        let username = env::var("USER").unwrap_or_else(|_| String::from("user"));
        let distro = OsRelease::new()
            .map(|os| os.name)
            .unwrap_or_else(|_| String::from("unknown"));

        let current_dir = self.current_dir.display().to_string();
        let current_dir = current_dir.replace(env::var("HOME").unwrap_or_default().as_str(), "~");

        format!(
            "{}@{} {} > ",
            username.bright_green(),
            distro.green(),
            current_dir.bright_blue()
        )
    }

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

    fn execute(&mut self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        // Check if command contains pipes
        let pipeline: Vec<(&str, Vec<&str>)> = self.parse_pipeline(command, args);

        // If we have a piped command
        if pipeline.len() > 1 {
            let external = ExternalCommand::new(self.current_dir.clone());
            external.execute_pipeline(&pipeline)?;
            return Ok(());
        }

        // Check if command contains redirects
        let (command, args, output) = self.parse_redirects(command, args);

        // If we have an output redirect
        if let Some(output) = output {
            let external = ExternalCommand::new(self.current_dir.clone());
            external.execute_redirect(command, &args, output)?;
            return Ok(());
        }

        // Execute without pipes
        match self.execute_builtin(command, &args) {
            Ok(true) => Ok(()),
            Ok(false) => match self.execute_external(command, &args) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }

    fn parse_redirects<'a>(
        &self,
        command: &'a str,
        args: &'a [&'a str],
    ) -> (&'a str, Vec<&'a str>, Option<&'a str>) {
        let mut args = args;
        let mut output = None;

        if args.contains(&">") {
            let index = args.iter().position(|&x| x == ">").unwrap();
            output = Some(args[index + 1]);
            args = &args[..index];
        }

        (command, args.to_vec(), output)
    }

    fn parse_pipeline<'a>(
        &self,
        command: &'a str,
        args: &'a [&'a str],
    ) -> Vec<(&'a str, Vec<&'a str>)> {
        let mut pipeline = Vec::new();
        let mut current_command = Vec::new();
        current_command.push(command);
        current_command.extend(args);

        let mut result = Vec::new();

        for arg in current_command {
            if arg == "|" {
                if !result.is_empty() {
                    let cmd = result[0];
                    let args = result[1..].to_vec();
                    pipeline.push((cmd, args));
                    result.clear();
                }
            } else {
                result.push(arg);
            }
        }

        if !result.is_empty() {
            let cmd = result[0];
            let args = result[1..].to_vec();
            pipeline.push((cmd, args));
        }

        pipeline
    }

    fn execute_builtin(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        let mut builtin = BuiltinCommand::new(self.current_dir.clone(), self.editor.history());
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
