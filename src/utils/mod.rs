use colored::{ColoredString, Colorize};
use std::{
    fs::Metadata,
    time::{SystemTime, UNIX_EPOCH},
};

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

pub fn format_size(size: u64) -> String {
    if size == 0 {
        "0".to_string()
    } else {
        let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
        let size = size as f64;
        let digit_group = 1024_f64;
        let exponent = (size.ln() / digit_group.ln()).floor() as usize;
        let size = size / digit_group.powi(exponent as i32);
        format!("{:.1}{}", size, units[exponent])
    }
}

pub fn format_time(time: u64) -> String {
    let time = time as i64;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let time_diff = now - time;

    if time_diff < 60 {
        format!("{}s", time_diff)
    } else if time_diff < 3600 {
        format!("{}m", time_diff / 60)
    } else if time_diff < 86400 {
        format!("{}h", time_diff / 3600)
    } else {
        format!("{}d", time_diff / 86400)
    }
}
