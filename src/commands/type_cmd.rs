use crate::core::{command::Command, command::CommandContext, flags::Flags};

#[derive(Clone)]
pub struct TypeCommand;

impl Command for TypeCommand {
    fn name(&self) -> &'static str {
        "type"
    }

    fn description(&self) -> &'static str {
        "Display information about command type"
    }

    fn extended_description(&self) -> &'static str {
        "Display information about command type"
    }

    fn execute(
        &self,
        args: &[&str],
        _flags: &Flags,
        context: &CommandContext,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if args.is_empty() {
            return Err("No command provided".into());
        }

        let cmd = args[0];

        if context.builtins.contains(&cmd) {
            println!("{} is a shell builtin", cmd);
        } else {
            let path = std::env::var("PATH")?;
            let paths = path.split(':');
            let mut found = false;

            for p in paths {
                let full_path = format!("{}/{}", p, cmd);
                if std::path::Path::new(&full_path).exists() {
                    println!("{} is {}", cmd, full_path);
                    found = true;
                    break;
                }
            }

            if !found {
                println!("{} not found", cmd);
            }
        }

        Ok(())
    }
}
