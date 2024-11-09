use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Flags {
    flags: HashMap<char, bool>,
    values: Vec<String>,
}

impl Flags {
    pub fn new(args: &[&str]) -> Self {
        let mut flags = HashMap::new();
        let mut values = Vec::new();

        for arg in args {
            if arg.starts_with('-') {
                let chars = arg.trim_start_matches('-').chars();
                for c in chars {
                    flags.insert(c, true);
                }
            } else {
                values.push(arg.to_string());
            }
        }

        Self { flags, values }
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.get(&flag).copied().unwrap_or(false)
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }
}
