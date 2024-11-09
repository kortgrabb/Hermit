use std::{
    env,
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use crate::{builtin::BuiltinCommand, external::ExternalCommand};
use std::path::Path;

#[derive(Debug)]
pub struct Shell {
    current_dir: PathBuf,
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
            print!("{}> ", self.current_dir.display());
            io::stdout().flush()?;

            let mut input = String::new();
            if io::stdin().read_line(&mut input)? == 0 {
                println!("\nGoodbye!");
                break;
            }

            let input = match input.split('#').next() {
                Some(cmd) => cmd.trim(),
                None => continue,
            };

            if input.is_empty() {
                continue;
            }

            self.history.push(input.to_string());

            let mut parts = input.split_whitespace();
            let command = parts.next().unwrap_or("");
            let args: Vec<&str> = parts.filter(|&x| !x.is_empty()).collect();

            if !self.execute_builtin(command, &args)? {
                self.execute_external(command, &args)?;
            }
        }

        Ok(())
    }

    fn execute_builtin(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        let mut builtin = BuiltinCommand::new(self.current_dir.clone(), self.history.clone());
        builtin.execute(command, args)
    }

    fn execute_external(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        let external = ExternalCommand::new(self.current_dir.clone());
        external.execute(command, args)
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
