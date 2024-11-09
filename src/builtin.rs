use std::{collections::HashMap, error::Error, path::PathBuf};

use crate::{
    command::Command,
    commands::{ChangeDirectory, Echo, History, ListDirectory},
    flags::Flags,
};

pub struct BuiltinCommand {
    commands: HashMap<&'static str, Box<dyn Command>>,
    pub current_dir: PathBuf,
    pub history: Vec<String>,
}

impl BuiltinCommand {
    pub fn new(current_dir: PathBuf, history: Vec<String>) -> Self {
        let commands: Vec<Box<dyn Command>> = vec![
            Box::new(Echo),
            Box::new(ChangeDirectory),
            Box::new(History::new(&history)),
            Box::new(ListDirectory::new()),
        ];

        let mut command_map = HashMap::new();
        for cmd in commands {
            command_map.insert(cmd.name(), cmd);
        }

        Self {
            commands: command_map,
            current_dir,
            history,
        }
    }

    pub fn execute(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        if let Some(cmd) = self.commands.get(command) {
            let flags = Flags::new(args);
            cmd.execute(args, &flags)?;
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
}
