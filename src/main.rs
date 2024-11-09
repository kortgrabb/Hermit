use std::error::Error;

mod builtin;
mod external;
mod shell;

use shell::Shell;

fn main() -> Result<(), Box<dyn Error>> {
    let mut shell = Shell::new()?;
    shell.run()?;

    Ok(())
}
