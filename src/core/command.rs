use std::error::Error;

use super::flags::Flags;

pub struct CommandContext {
    pub history: Vec<String>,
    pub builtins: Vec<&'static str>,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            builtins: Vec::new(),
        }
    }
}

pub trait Command {
    fn execute(
        &self,
        args: &[&str],
        flags: &Flags,
        context: &CommandContext,
    ) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    // TODO
    fn extended_description(&self) -> &'static str {
        self.description()
    }
}
