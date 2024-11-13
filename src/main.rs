use std::error::Error;

mod commands;
mod config;
mod core;
mod git;
mod shell;
mod utils;

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
