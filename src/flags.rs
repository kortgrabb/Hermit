use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Flags {
    flags: HashSet<char>,
}

impl Flags {
    pub fn new(args: &[&str]) -> Self {
        let mut flags = HashSet::new();
        let mut values = Vec::new();

        for &arg in args {
            if let Some(flag_chars) = arg.strip_prefix('-') {
                flags.extend(flag_chars.chars());
            } else {
                values.push(arg.to_string());
            }
        }

        Self { flags }
    }

    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.contains(&flag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_flags() {
        let args = vec!["-abc"];
        let flags = Flags::new(&args);

        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert!(flags.has_flag('c'));
    }

    #[test]
    fn test_spaced_flags() {
        let args = vec!["-a", "-b", "-c"];
        let flags = Flags::new(&args);

        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert!(flags.has_flag('c'));
    }
}
