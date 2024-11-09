use std::error::Error;

mod builtin;
mod command;
mod commands;
mod external;
mod flags;
mod shell;
mod utils;

pub use builtin::BuiltinCommand;
use shell::Shell;

fn main() -> Result<(), Box<dyn Error>> {
    let mut shell = Shell::new()?;
    match shell.run() {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}
