use std::{env, error::Error, path::PathBuf};

pub struct BuiltinCommand {
    pub current_dir: PathBuf,
    pub history: Vec<String>,
}

impl BuiltinCommand {
    pub fn new(current_dir: PathBuf, history: Vec<String>) -> Self {
        Self {
            current_dir,
            history,
        }
    }

    pub fn execute(&mut self, command: &str, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        match command {
            "cd" => self.cd(args),
            "history" => self.history(args),
            "echo" => self.echo(args),
            _ => Ok(false),
        }
    }

    fn cd(&mut self, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        if args.is_empty() {
            return Err("cd: missing argument".into());
        }

        let new_dir = match args[0] {
            "~" => env::var("HOME").map(PathBuf::from)?,
            "-" => env::var("OLDPWD")
                .map(PathBuf::from)
                .unwrap_or_else(|_| self.current_dir.clone()),
            _ => {
                let path = PathBuf::from(args[0]);
                if path.is_absolute() {
                    path
                } else {
                    self.current_dir.join(path)
                }
            }
        };

        if new_dir.is_dir() {
            env::set_var("OLDPWD", &self.current_dir);
            self.current_dir = new_dir.canonicalize()?;
            env::set_current_dir(&self.current_dir)?;
        } else {
            return Err(format!("cd: no such file or directory: {}", new_dir.display()).into());
        }

        Ok(true)
    }

    fn history(&self, _args: &[&str]) -> Result<bool, Box<dyn Error>> {
        for (i, cmd) in self.history.iter().enumerate() {
            println!("{}: {}", i, cmd);
        }
        Ok(true)
    }

    fn echo(&self, args: &[&str]) -> Result<bool, Box<dyn Error>> {
        println!("{}", args.join(" "));
        Ok(true)
    }
}
