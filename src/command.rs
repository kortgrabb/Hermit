use crate::flags::Flags;
use std::error::Error;

pub struct CommandContext {
    pub history: Vec<String>,
    pub builtins: Vec<&'static str>,
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
