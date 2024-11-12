use std::error::Error;

mod builtin;
mod command;
mod commands;
mod completer;
mod external;
mod flags;
mod git;
mod shell;
mod utils;

pub use builtin::CommandRegistry;
use shell::Shell;

fn main() -> Result<(), Box<dyn Error>> {
    let mut shell = Shell::new().map_err(|e| format!("Failed to initialize shell: {}", e))?;

    if let Err(e) = shell.run() {
        eprintln!("Shell error: {}", e);
        std::process::exit(1);
    }

    println!("Goodbye!");
    Ok(())
}
