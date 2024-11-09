use std::{collections::HashMap, error::Error, path::PathBuf};

use crate::{
    command::Command,
    commands::{ChangeDirectory, Echo, History},
};

pub struct BuiltinCommand {
    commands: HashMap<&'static str, Box<dyn Command>>,
    pub current_dir: PathBuf,
    pub history: Vec<String>,
}

impl BuiltinCommand {
    pub fn new(current_dir: PathBuf, history: Vec<String>) -> Self {
        let mut commands = HashMap::new();
        commands.insert("echo", Box::new(Echo) as Box<dyn Command>);
        commands.insert("cd", Box::new(ChangeDirectory) as Box<dyn Command>);
        commands.insert(
            "history",
            Box::new(History::new(&history)) as Box<dyn Command>,
        );

        Self {
            commands,
            current_dir,
            history,
        }
    }

    pub fn execute(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        if let Some(cmd) = self.commands.get(command) {
            cmd.execute(args)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
