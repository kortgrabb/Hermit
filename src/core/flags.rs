use std::collections::{HashMap, HashSet};

/// Represents command-line flags and their associated values
#[derive(Debug, Clone, Default)]
pub struct Flags {
    flags: HashSet<char>,
    values: HashMap<char, String>,
}

/// Represents errors that can occur during flag parsing
#[derive(Debug, thiserror::Error)]
pub enum FlagError {
    #[error("Invalid flag format: {0}")]
    InvalidFormat(String),
    #[error("Missing value for flag: {0}")]
    MissingValue(char),
    #[error("Duplicate flag: {0}")]
    DuplicateFlag(char),
}

impl Flags {
    /// Creates a new Flags instance from command-line arguments
    ///
    /// # Arguments
    /// * `args` - Slice of argument strings
    /// * `value_flags` - Set of flags that require values
    ///
    /// # Returns
    /// * `Result<Self, FlagError>` - New Flags instance or error
    pub fn new(args: &[&str]) -> Result<Self, FlagError> {
        Self::with_value_flags(args, &[])
    }

    /// Creates a new Flags instance with specified value flags
    ///
    /// # Arguments
    /// * `args` - Slice of argument strings
    /// * `value_flags` - Slice of flags that require values
    ///
    /// # Returns
    /// * `Result<Self, FlagError>` - New Flags instance or error
    pub fn with_value_flags(args: &[&str], value_flags: &[char]) -> Result<Self, FlagError> {
        let mut flags = HashSet::new();
        let mut values = HashMap::new();
        let value_flags: HashSet<_> = value_flags.iter().copied().collect();

        let mut i = 0;
        while i < args.len() {
            let arg = args[i];

            if let Some(flag_chars) = arg.strip_prefix('-') {
                if flag_chars.is_empty() {
                    return Err(FlagError::InvalidFormat("Empty flag".to_string()));
                }

                for c in flag_chars.chars() {
                    if flags.contains(&c) {
                        return Err(FlagError::DuplicateFlag(c));
                    }

                    if value_flags.contains(&c) {
                        i += 1;
                        if i >= args.len() {
                            return Err(FlagError::MissingValue(c));
                        }
                        values.insert(c, args[i].to_string());
                    }
                    flags.insert(c);
                }
            }
            i += 1;
        }

        Ok(Self { flags, values })
    }

    /// Checks if a flag is present
    ///
    /// # Arguments
    /// * `flag` - The flag character to check
    pub fn has_flag(&self, flag: char) -> bool {
        self.flags.contains(&flag)
    }

    /// Gets the value associated with a flag
    ///
    /// # Arguments
    /// * `flag` - The flag character to get the value for
    pub fn get_value(&self, flag: char) -> Option<&str> {
        self.values.get(&flag).map(String::as_str)
    }

    /// Adds a flag
    ///
    /// # Arguments
    /// * `flag` - The flag character to add
    pub fn add_flag(&mut self, flag: char) {
        self.flags.insert(flag);
    }

    /// Adds a flag with a value
    ///
    /// # Arguments
    /// * `flag` - The flag character to add
    /// * `value` - The value to associate with the flag
    pub fn add_value(&mut self, flag: char, value: String) {
        self.flags.insert(flag);
        self.values.insert(flag, value);
    }

    /// Gets all flags
    pub fn flags(&self) -> &HashSet<char> {
        &self.flags
    }

    /// Gets all flag values
    pub fn values(&self) -> &HashMap<char, String> {
        &self.values
    }

    /// Creates flags from a string
    ///
    /// # Arguments
    /// * `s` - The string to parse flags from
    pub fn from_str(s: &str) -> Result<Self, FlagError> {
        let args: Vec<&str> = s.split_whitespace().collect();
        Self::new(&args)
    }

    /// Returns number of flags
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    /// Checks if there are no flags
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }

    /// Removes a flag
    ///
    /// # Arguments
    /// * `flag` - The flag character to remove
    pub fn remove_flag(&mut self, flag: char) {
        self.flags.remove(&flag);
        self.values.remove(&flag);
    }

    /// Clears all flags and values
    pub fn clear(&mut self) {
        self.flags.clear();
        self.values.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_flags() {
        let args = vec!["-abc"];
        let flags = Flags::new(&args).unwrap();
        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert!(flags.has_flag('c'));
        assert!(!flags.has_flag('d'));
    }

    #[test]
    fn test_spaced_flags() {
        let args = vec!["-a", "-b", "-c"];
        let flags = Flags::new(&args).unwrap();
        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert!(flags.has_flag('c'));
        assert!(!flags.has_flag('d'));
    }

    #[test]
    fn test_value_flags() {
        let args = vec!["-a", "value", "-b"];
        let value_flags = ['a'];
        let flags = Flags::with_value_flags(&args, &value_flags).unwrap();
        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert_eq!(flags.get_value('a'), Some("value"));
        assert_eq!(flags.get_value('b'), None);
    }

    #[test]
    fn test_missing_value() {
        let args = vec!["-a"];
        let value_flags = ['a'];
        let result = Flags::with_value_flags(&args, &value_flags);
        assert!(matches!(result, Err(FlagError::MissingValue('a'))));
    }

    #[test]
    fn test_duplicate_flags() {
        let args = vec!["-a", "-a"];
        let result = Flags::new(&args);
        assert!(matches!(result, Err(FlagError::DuplicateFlag('a'))));
    }

    #[test]
    fn test_from_str() {
        let flags = Flags::from_str("-abc -d value").unwrap();
        assert!(flags.has_flag('a'));
        assert!(flags.has_flag('b'));
        assert!(flags.has_flag('c'));
        assert!(flags.has_flag('d'));
    }

    #[test]
    fn test_add_and_remove() {
        let mut flags = Flags::default();
        flags.add_flag('a');
        assert!(flags.has_flag('a'));
        flags.remove_flag('a');
        assert!(!flags.has_flag('a'));
    }

    #[test]
    fn test_add_value() {
        let mut flags = Flags::default();
        flags.add_value('a', "value".to_string());
        assert!(flags.has_flag('a'));
        assert_eq!(flags.get_value('a'), Some("value"));
    }

    #[test]
    fn test_clear() {
        let mut flags = Flags::default();
        flags.add_flag('a');
        flags.add_value('b', "value".to_string());
        flags.clear();
        assert!(flags.is_empty());
        assert_eq!(flags.len(), 0);
    }
}
