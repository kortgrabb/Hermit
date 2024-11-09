use std::io::{self, Write};
use std::process::Command;

fn main() -> io::Result<()> {
    loop {
        // Print prompt
        print!("$ ");
        io::stdout().flush()?;

        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Trim whitespace and check if empty
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Handle exit command
        if input == "exit" {
            println!("Goodbye!");
            break;
        }
    }

    Ok(())
}
