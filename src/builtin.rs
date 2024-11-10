use rustyline::history::FileHistory;

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
    pub fn new(current_dir: PathBuf, history: &FileHistory) -> Self {
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
            history: history.iter().map(|s| s.to_string()).collect(),
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
