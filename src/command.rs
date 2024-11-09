use std::error::Error;
use std::path::PathBuf;

pub trait Command {
    fn execute(&self, args: &[&str]) -> Result<(), Box<dyn Error>>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn extended_description(&self) -> &'static str;
}
