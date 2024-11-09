use std::error::Error;

mod builtin;
mod command;
mod commands;
mod external;
mod shell;

pub use builtin::BuiltinCommand;
use shell::Shell;

fn main() -> Result<(), Box<dyn Error>> {
    let mut shell = Shell::new()?;
    shell.run()?;

    Ok(())
}
