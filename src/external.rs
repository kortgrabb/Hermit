use os_pipe::pipe;
use std::{
    io,
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

    pub fn execute(&self, command: &str, args: &[&str]) -> io::Result<()> {
        let mut child = Command::new(command)
            .args(args)
            .current_dir(&self.current_dir)
            .spawn()?;

        let status = child.wait()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Command exited with status: {}", status),
            ));
        }

        Ok(())
    }

    pub fn execute_pipeline(&self, pipeline: &[(&str, Vec<&str>)]) -> io::Result<()> {
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
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Pipeline command exited with status: {}", status),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_execute() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let current_dir = tmp_dir.path().to_path_buf();

        let command = ExternalCommand::new(current_dir.clone());
        let result = command.execute("touch", &["test.txt"]);

        assert!(result.is_ok());
        assert!(current_dir.join("test.txt").exists());
    }

    #[test]
    fn test_execute_pipeline() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let current_dir = tmp_dir.path().to_path_buf();

        let command = ExternalCommand::new(current_dir);
        let pipeline = vec![("echo", vec!["hello"]), ("grep", vec!["hello"])];

        let result = command.execute_pipeline(&pipeline);

        assert!(result.is_ok());
    }
}
