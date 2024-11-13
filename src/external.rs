use os_pipe::pipe;
use std::{
    fs::OpenOptions,
    io::{self, Error, ErrorKind},
    path::PathBuf,
    process::{Child, Command, ExitStatus},
};

type CommandResult<T> = io::Result<T>;

/// Represents an external command executor that can run system commands
#[derive(Debug, Clone)]
pub struct ExternalCommand {
    current_dir: PathBuf,
}

impl ExternalCommand {
    /// Creates a new ExternalCommand instance with the specified working directory
    pub fn new(current_dir: PathBuf) -> Self {
        Self { current_dir }
    }

    /// Executes a single command with arguments
    pub fn execute(&self, command: &str, args: &[&str]) -> CommandResult<()> {
        let status = self.spawn_command(command, args)?.wait()?;
        self.check_status(status, "Command")
    }

    /// Executes a pipeline of commands where each command's output feeds into the next command's input
    pub fn execute_pipeline(&self, pipeline: &[(&str, Vec<&str>)]) -> CommandResult<()> {
        if pipeline.is_empty() {
            return Ok(());
        }

        let mut processes = Vec::new();
        let mut previous_pipe = None;

        // Set up and spawn all processes in the pipeline
        for (i, (cmd, args)) in pipeline.iter().enumerate() {
            let mut command = self.create_base_command(cmd, args);

            // Connect pipes between processes
            if let Some(prev_pipe) = previous_pipe.take() {
                command.stdin(prev_pipe);
            }

            // Create pipe for next process if not last in pipeline
            if i < pipeline.len() - 1 {
                let (reader, writer) = pipe()?;
                command.stdout(writer);
                previous_pipe = Some(reader);
            }

            processes.push(command.spawn()?);
        }

        // Wait for all processes and check their status
        self.wait_for_processes(processes)
    }

    /// Executes a command and redirects its output to a file
    pub fn execute_redirect(
        &self,
        command: &str,
        args: &[&str],
        redirect: &str,
    ) -> CommandResult<()> {
        let file = self.open_redirect_file(redirect)?;

        let status = self
            .spawn_command_with_output(command, args, file)?
            .wait()?;
        self.check_status(status, "Redirect command")
    }

    // Helper methods

    fn spawn_command(&self, command: &str, args: &[&str]) -> CommandResult<Child> {
        self.create_base_command(command, args).spawn()
    }

    fn spawn_command_with_output(
        &self,
        command: &str,
        args: &[&str],
        output: impl Into<std::process::Stdio>,
    ) -> CommandResult<Child> {
        self.create_base_command(command, args)
            .stdout(output)
            .spawn()
    }

    fn create_base_command(&self, command: &str, args: &[&str]) -> Command {
        let mut cmd = Command::new(command);
        cmd.args(args).current_dir(&self.current_dir);
        cmd
    }

    fn check_status(&self, status: ExitStatus, context: &str) -> CommandResult<()> {
        if !status.success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("{} exited with status: {}", context, status),
            ));
        }
        Ok(())
    }

    fn wait_for_processes(&self, processes: Vec<Child>) -> CommandResult<()> {
        for (i, mut process) in processes.into_iter().enumerate() {
            let status = process.wait()?;
            if !status.success() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Pipeline command {} exited with status: {}", i + 1, status),
                ));
            }
        }
        Ok(())
    }

    fn open_redirect_file(&self, path: &str) -> CommandResult<std::fs::File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.trim())
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("Failed to open redirect file: {}", e),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> (ExternalCommand, TempDir) {
        let tmp_dir = TempDir::new().expect("Failed to create temp dir");
        let command = ExternalCommand::new(tmp_dir.path().to_path_buf());
        (command, tmp_dir)
    }

    #[test]
    fn test_execute_basic_command() {
        let (command, tmp_dir) = setup();
        let test_file = tmp_dir.path().join("test.txt");

        command
            .execute("touch", &[test_file.to_str().unwrap()])
            .unwrap();
        assert!(test_file.exists());
    }

    #[test]
    fn test_execute_failing_command() {
        let (command, _tmp_dir) = setup();
        let result = command.execute("nonexistent", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_pipeline() {
        let (command, tmp_dir) = setup();
        let output_file = tmp_dir.path().join("output.txt");

        // Create a pipeline that writes to a file
        let pipeline = vec![("echo", vec!["hello world"]), ("grep", vec!["world"])];

        command.execute_pipeline(&pipeline).unwrap();
    }

    #[test]
    fn test_execute_redirect() {
        let (command, tmp_dir) = setup();
        let output_file = tmp_dir.path().join("redirect.txt");
        let output_path = output_file.to_str().unwrap();

        command
            .execute_redirect("echo", &["hello"], output_path)
            .unwrap();

        let content = fs::read_to_string(output_file).unwrap();
        assert_eq!(content.trim(), "hello");
    }

    #[test]
    fn test_empty_pipeline() {
        let (command, _tmp_dir) = setup();
        let result = command.execute_pipeline(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_redirect_path() {
        let (command, _tmp_dir) = setup();
        let result = command.execute_redirect("echo", &["test"], "/nonexistent/path/file.txt");
        assert!(result.is_err());
    }
}
