use rustyline::{
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    validate::{self, MatchingBracketValidator, Validator},
    Context, Helper,
};

pub struct CommandCompleter {
    commands: Vec<String>,
}

impl CommandCompleter {
    pub fn new(commands: Vec<&'static str>) -> Self {
        Self {
            commands: commands.into_iter().map(String::from).collect(),
        }
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let word = &line[start..pos].to_lowercase();

        let mut matches = Vec::new();

        // Only match commands if we're at the start of the line
        if start == 0 {
            matches.extend(
                self.commands
                    .iter()
                    .filter(|cmd| {
                        let cmd_lower = cmd.to_lowercase();
                        cmd_lower.starts_with(word.as_str()) || cmd_lower.contains(word.as_str())
                    })
                    .map(|cmd| Pair {
                        display: cmd.clone(),
                        replacement: cmd.clone(),
                    }),
            );
        }

        if word.starts_with("./") || word.starts_with('/') || !word.contains('/') {
            if let Ok(entries) = std::fs::read_dir(".") {
                matches.extend(
                    entries
                        .filter_map(Result::ok)
                        .filter(|entry| {
                            let name = entry.file_name().to_string_lossy().to_lowercase();
                            name.contains(word)
                        })
                        .map(|entry| {
                            let name = entry.file_name().to_string_lossy().to_string();
                            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                            let display = if is_dir {
                                format!("{}/", name)
                            } else {
                                name.clone()
                            };
                            Pair {
                                display,
                                replacement: name,
                            }
                        }),
                );
            }
        }

        // Sort matches: exact prefix matches first, then contained matches
        matches.sort_by(|a, b| {
            let a_lower = a.display.to_lowercase();
            let b_lower = b.display.to_lowercase();
            let a_starts = a_lower.starts_with(word);
            let b_starts = b_lower.starts_with(word);

            match (a_starts, b_starts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.display.cmp(&b.display),
            }
        });

        Ok((start, matches))
    }
}

// Add missing trait implementations
impl Validator for CommandCompleter {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        MatchingBracketValidator::new().validate(ctx)
    }
}

impl Hinter for CommandCompleter {
    type Hint = String;
}

impl Highlighter for CommandCompleter {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        rustyline::highlight::MatchingBracketHighlighter::new().highlight(line, pos)
    }
}

impl Helper for CommandCompleter {}
