use os_pipe::pipe;
use std::{
    error::Error,
    path::PathBuf,
    process::{Command, Stdio},
};

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

    pub fn execute_pipeline(&self, pipeline: &[(&str, Vec<&str>)]) -> Result<(), Box<dyn Error>> {
        let mut processes = Vec::new();
        let mut previous_pipe = None;

        // Go through each command in the pipeline
        for (i, (cmd, args)) in pipeline.iter().enumerate() {
            let is_last = i == pipeline.len() - 1;
            let mut command = Command::new(cmd);
            command.args(args);
            command.current_dir(&self.current_dir);

            // Set up stdin from previous pipe if it exists
            if let Some(prev_pipe) = previous_pipe.take() {
                command.stdin(prev_pipe);
            }

            /*
            If we are not last, we want to print to stdout
            instead of the terminal.
             */
            if !is_last {
                let (reader, writer) = pipe()?;
                command.stdout(writer);
                previous_pipe = Some(reader);
            }

            // Execute the command
            let child = command.spawn()?;
            processes.push(child);
        }

        // Wait for all processes to complete
        for mut process in processes {
            let status = process.wait()?;
            if !status.success() {
                return Err(format!("Pipeline command exited with status: {}", status).into());
            }
        }

        Ok(())
    }
}
