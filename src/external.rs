use std::{error::Error, path::PathBuf, process::Command};

pub struct ExternalCommand {
    pub current_dir: PathBuf,
}

impl ExternalCommand {
    pub fn new(current_dir: PathBuf) -> Self {
        Self { current_dir }
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        let mut child = Command::new(command)
            .args(args)
            .current_dir(&self.current_dir)
            .spawn()?;

        let status = child.wait()?;

        if !status.success() {
            return Err(format!("Command exited with status: {}", status).into());
        }

        Ok(())
    }
}
