use rustyline::history::FileHistory;

use crate::commands::{
    ChangeDirectory, Echo, History, ListDirectory, PrintWorkingDirectory, TypeCommand,
};
use std::{collections::HashMap, error::Error, path::PathBuf};

use super::{
    command::{Command, CommandContext},
    flags::Flags,
};

pub struct CommandRegistry {
    commands: HashMap<&'static str, Box<dyn Command>>,
    context: CommandContext,
}

impl CommandRegistry {
    pub fn setup(history: &FileHistory) -> Self {
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

        CommandRegistry {
            commands: command_map,
            context,
        }
    }

    pub fn execute(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        if let Some(cmd) = self.commands.get(command) {
            let flags = Flags::new(args);
            cmd.execute(args, &flags?, &self.context)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_commands(&self) -> Vec<&'static str> {
        self.commands.keys().copied().collect()
    }
}
