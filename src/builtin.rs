use crate::{
    command::{Command, CommandContext},
    commands::{ChangeDirectory, Echo, History, ListDirectory, PrintWorkingDirectory},
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
            Box::new(ListDirectory::new()),
            Box::new(PrintWorkingDirectory::new()),
            Box::new(History),
        ];

        let mut command_map = HashMap::new();
        for cmd in commands {
            command_map.insert(cmd.name(), cmd);
        }

        Self {
            commands: command_map,
            current_dir,
            context: CommandContext { history },
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
