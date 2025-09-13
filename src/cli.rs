use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "logwatcher",
    about = "Real-time log file monitoring with pattern highlighting and desktop notifications",
    version = "0.1.0",
    long_about = "LogWatcher is a CLI tool for monitoring log files in real-time. It provides pattern highlighting, desktop notifications, and handles file rotation automatically."
)]
pub struct Args {
    /// Path(s) to log file(s) to watch
    #[arg(short = 'f', long = "file", required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,

    /// Comma-separated patterns to match
    #[arg(short = 'p', long = "pattern", default_value = "ERROR,WARN")]
    pub patterns: String,

    /// Treat patterns as regular expressions
    #[arg(short = 'r', long = "regex")]
    pub regex: bool,

    /// Case-insensitive pattern matching
    #[arg(short = 'i', long = "case-insensitive")]
    pub case_insensitive: bool,

    /// Custom pattern:color mappings (e.g., "ERROR:red,WARN:yellow")
    #[arg(short = 'c', long = "color-map")]
    pub color_map: Option<String>,

    /// Enable desktop notifications
    #[arg(short = 'n', long = "notify", default_value = "true")]
    pub notify: bool,

    /// Specific patterns that trigger notifications (default: all patterns)
    #[arg(long = "notify-patterns")]
    pub notify_patterns: Option<String>,

    /// Maximum notifications per second
    #[arg(long = "notify-throttle", default_value = "5")]
    pub notify_throttle: u32,

    /// Preview mode (no tailing, no notifications)
    #[arg(short = 'd', long = "dry-run")]
    pub dry_run: bool,

    /// Suppress non-matching lines
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Disable ANSI colors
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Prefix lines with filename (auto: true for multiple files)
    #[arg(long = "prefix-file")]
    pub prefix_file: Option<bool>,

    /// File polling interval in milliseconds
    #[arg(long = "poll-interval", default_value = "100")]
    pub poll_interval: u64,

    /// Read buffer size in bytes
    #[arg(long = "buffer-size", default_value = "8192")]
    pub buffer_size: usize,
}

impl Args {
    /// Get the list of files to watch
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Get the patterns as a vector of strings
    pub fn patterns(&self) -> Vec<String> {
        self.patterns
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Get notification patterns as a vector of strings
    pub fn notify_patterns(&self) -> Vec<String> {
        if let Some(ref patterns) = self.notify_patterns {
            patterns
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            self.patterns()
        }
    }

    /// Get color mappings as a vector of (pattern, color) tuples
    pub fn color_mappings(&self) -> Vec<(String, String)> {
        if let Some(ref color_map) = self.color_map {
            color_map
                .split(',')
                .filter_map(|mapping| {
                    let parts: Vec<&str> = mapping.split(':').collect();
                    if parts.len() == 2 {
                        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Determine if filename prefixing should be enabled
    pub fn should_prefix_files(&self) -> bool {
        if let Some(prefix) = self.prefix_file {
            prefix
        } else {
            self.files.len() > 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mappings_invalid_format() {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: Some("invalid_format".to_string()),
            notify: false,
            notify_patterns: None,
            quiet: false,
            dry_run: false,
            prefix_file: Some(false),
            poll_interval: 1000,
            buffer_size: 8192,
            no_color: false,
            notify_throttle: 0,
        };

        let mappings = args.color_mappings();
        assert_eq!(mappings.len(), 0); // Should return empty map for invalid format
    }
}
