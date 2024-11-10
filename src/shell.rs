use std::{
    env,
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use crate::{builtin::BuiltinCommand, external::ExternalCommand};
use colored::Colorize;
use os_release::OsRelease;

#[derive(Debug)]
pub struct Shell {
    current_dir: PathBuf,
    // TODO: save to file
    history: Vec<String>,
}

impl Shell {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Shell {
            current_dir: env::current_dir()?,
            history: Vec::new(),
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

                self.history.push(format!("{} {}", command, args.join(" ")));

                match command.to_lowercase().as_str() {
                    "help" => {
                        if args.is_empty() {
                            self.print_help();
                        } else {
                            self.print_command_help(args[0]);
                        }
                    }
                    "exit" => {
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
        let username = env::var("USER").unwrap_or_else(|_| String::from("user"));
        let distro = OsRelease::new()
            .map(|os| os.name)
            .unwrap_or_else(|_| String::from("unknown"));

        let current_dir = self.current_dir.display().to_string();
        let current_dir = current_dir.replace(env::var("HOME").unwrap_or_default().as_str(), "~");

        print!(
            "{}@{} {}$ ",
            username.bright_green(),
            distro.green(),
            current_dir.bright_blue()
        );
        io::stdout().flush().unwrap_or_default();
    }

    fn read_input(&self) -> Vec<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        self.transform_input(input)
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

        // Execute without pipes
        match self.execute_builtin(command, args) {
            Ok(true) => Ok(()),
            Ok(false) => match self.execute_external(command, args) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
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
        let mut builtin = BuiltinCommand::new(self.current_dir.clone(), self.history.clone());
        builtin.execute(command, args)
    }

    fn print_help(&self) {
        let builtin = BuiltinCommand::new(PathBuf::new(), Vec::new());
        builtin.print_all_help();
    }

    fn print_command_help(&self, command: &str) {
        let builtin = BuiltinCommand::new(self.current_dir.clone(), self.history.clone());
        builtin.print_command_help(command);
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
    use super::*;

    #[test]
    fn test_shell_new() {
        let shell = Shell::new().unwrap();
        assert_eq!(shell.current_dir, env::current_dir().unwrap());
        assert!(shell.history.is_empty());
    }

    // Stoopid test but whatever
    #[test]
    fn test_shell_history() {
        let mut shell = Shell::new().unwrap();
        assert!(shell.history.is_empty());

        shell.history.push("test".to_string());
        assert_eq!(shell.history, vec!["test"]);
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
