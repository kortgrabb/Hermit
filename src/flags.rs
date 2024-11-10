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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flags() {
        let args = vec!["-a", "-b", "value1", "-c", "value2"];
        let flags = Flags::new(&args);

        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));

        assert_eq!(flags.values(), &["value1", "value2"]);
    }

    #[test]
    fn test_flags_no_values() {
        let args = vec!["-a", "-b", "-c"];
        let flags = Flags::new(&args);

        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert!(flags.has_flag('c'));

        assert_eq!(flags.values(), &[] as &[String]);
    }

    #[test]
    fn test_compact_flags() {
        let args = vec!["-abc"];
        let flags = Flags::new(&args);

        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert!(flags.has_flag('c'));
    }
}
