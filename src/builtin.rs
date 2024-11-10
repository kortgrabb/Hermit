use crate::{
    command::{Command, CommandContext},
    commands::{ChangeDirectory, Echo, History, ListDirectory, PrintWorkingDirectory, TypeCommand},
    flags::Flags,
};
use std::{collections::HashMap, error::Error, path::PathBuf};

pub struct BuiltinCommand {
    commands: HashMap<&'static str, Box<dyn Command>>,
    pub current_dir: PathBuf,
    context: CommandContext,
}

impl BuiltinCommand {
    pub fn new(current_dir: PathBuf, history: Vec<String>) -> Self {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(Echo),
            Box::new(ChangeDirectory),
            Box::new(ListDirectory),
            Box::new(PrintWorkingDirectory),
            Box::new(History),
            Box::new(TypeCommand),
        ];

        let command_names: Vec<&'static str> = commands.iter().map(|cmd| cmd.name()).collect();
        let mut command_map = HashMap::new();
        for cmd in commands {
            command_map.insert(cmd.name(), cmd);
        }

        let context = CommandContext {
            history,
            builtins: command_names,
        };

        BuiltinCommand {
            commands: command_map,
            current_dir,
            context,
        }
    }

    pub fn execute(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        if let Some(cmd) = self.commands.get(command) {
            let flags = Flags::new(args);
            cmd.execute(args, &flags, &self.context)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn print_command_help(&self, command: &str) {
        if let Some(cmd) = self.commands.get(command) {
            println!("{} - {}", cmd.name(), cmd.description());
            println!("{}", cmd.extended_description());
        }
    }

    pub fn print_all_help(&self) {
        for cmd in self.commands.values() {
            println!("{} - {}", cmd.name(), cmd.description());
        }
    }

    pub fn get_commands(&self) -> Vec<&'static str> {
        self.commands.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // test that all commands are loaded
    fn test_builtin_command() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let builtin = BuiltinCommand::new(current_dir, history);

        let commands = builtin.get_commands();
        assert_eq!(commands.len(), 6);
        assert!(commands.contains(&"echo"));
        assert!(commands.contains(&"cd"));
        assert!(commands.contains(&"ls"));
        assert!(commands.contains(&"pwd"));
        assert!(commands.contains(&"history"));
        assert!(commands.contains(&"type"));
    }

    #[test]
    fn test_execute_echo() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let mut builtin = BuiltinCommand::new(current_dir, history);

        let result = builtin.execute("echo", &["hello"]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_execute_cd() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let mut builtin = BuiltinCommand::new(current_dir, history);

        let result = builtin.execute("cd", &["/"]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_execute_ls() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let mut builtin = BuiltinCommand::new(current_dir, history);

        let result = builtin.execute("ls", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_execute_pwd() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let mut builtin = BuiltinCommand::new(current_dir, history);

        let result = builtin.execute("pwd", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_execute_history() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let mut builtin = BuiltinCommand::new(current_dir, history);

        let result = builtin.execute("history", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_execute_type() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let mut builtin = BuiltinCommand::new(current_dir, history);

        let result = builtin.execute("type", &["echo"]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_execute_unknown_command() {
        let current_dir = PathBuf::from("/home/user");
        let history = vec!["echo hello".to_string(), "cd /".to_string()];
        let mut builtin = BuiltinCommand::new(current_dir, history);

        let result = builtin.execute("unknown", &[]);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
