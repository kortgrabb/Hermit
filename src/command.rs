use crate::flags::Flags;
use std::error::Error;

#[derive(Clone)]
pub struct CommandContext {
    pub history: Vec<String>,
    // Add other shared data here as needed
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
    fn extended_description(&self) -> &'static str {
        self.description()
    }
}
