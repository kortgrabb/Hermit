use std::fs::Metadata;

use colored::{ColoredString, Colorize};

pub fn term_width() -> usize {
    term_size::dimensions().map_or(80, |(w, _)| w)
}

pub fn colorize_file_name(file_name: &str, metadata: &Metadata) -> ColoredString {
    let final_name = if metadata.is_dir() {
        file_name.blue()
    } else if metadata.is_dir() && file_name.starts_with('.') {
        file_name.blue().dimmed()
    } else if file_name.starts_with('.') {
        file_name.dimmed()
    } else {
        file_name.normal()
    }
    .to_string();

    final_name.normal()
}
