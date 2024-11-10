use colored::{ColoredString, Colorize};
use std::fs::Metadata;

pub fn term_width() -> usize {
    term_size::dimensions().map_or(80, |(w, _)| w)
}

pub fn colorize_file_name(file_name: &str, metadata: &Metadata) -> ColoredString {
    match (metadata.is_dir(), file_name.starts_with('.')) {
        (true, true) => file_name.blue().dimmed(),
        (true, false) => file_name.blue(),
        (false, true) => file_name.dimmed(),
        (false, false) => file_name.normal(),
    }
}
