use std::error::Error;

use crate::flags::Flags;

pub trait Command {
    fn execute(&self, args: &[&str], flags: &Flags) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn extended_description(&self) -> &'static str {
        self.description()
    }
}
