use crate::cli::Args;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use termcolor::Color;

/// Maximum size limit for regex patterns to prevent ReDoS attacks
const REGEX_SIZE_LIMIT: usize = 10 * 1024 * 1024; // 10 MB

#[derive(Debug, Clone)]
pub struct Config {
    pub files: Vec<PathBuf>,
    pub patterns: Vec<String>,
    pub regex_patterns: Vec<Regex>,
    pub exclude_patterns: Vec<String>,
    pub exclude_patterns_lowercase: Vec<String>, // Pre-computed for case-insensitive matching
    pub exclude_regex_patterns: Vec<Regex>,
    pub case_insensitive: bool,
    pub color_mappings: HashMap<String, Color>,
    pub notify_enabled: bool,
    pub notify_patterns: Vec<String>,
    pub notify_throttle: u32,
    pub dry_run: bool,
    pub quiet: bool,
    pub no_color: bool,
    pub prefix_files: bool,
    pub poll_interval: u64,
    pub buffer_size: usize,
}

impl Config {
    pub fn from_args(args: &Args) -> Result<Self> {
        let patterns = args.patterns();
        let notify_patterns = args.notify_patterns();
        let exclude_patterns = args.exclude_patterns();

        // Validate and compile regex patterns if needed
        let regex_patterns = if args.regex {
            Self::compile_regex_patterns(&patterns, args.case_insensitive)?
        } else {
            vec![]
        };

        // Compile exclude patterns as regex if regex mode is enabled
        let exclude_regex_patterns = if args.regex && !exclude_patterns.is_empty() {
            Self::compile_regex_patterns(&exclude_patterns, args.case_insensitive)?
        } else {
            vec![]
        };

        // Pre-compute lowercase exclude patterns for case-insensitive matching
        let exclude_patterns_lowercase = if args.case_insensitive {
            exclude_patterns.iter().map(|p| p.to_lowercase()).collect()
        } else {
            vec![]
        };

        // Parse color mappings
        let color_mappings = Self::parse_color_mappings(&args.color_mappings())?;

        Ok(Config {
            files: args.files().to_vec(),
            patterns,
            regex_patterns,
            exclude_patterns,
            exclude_patterns_lowercase,
            exclude_regex_patterns,
            case_insensitive: args.case_insensitive,
            color_mappings,
            notify_enabled: args.notify,
            notify_patterns,
            notify_throttle: args.notify_throttle,
            dry_run: args.dry_run,
            quiet: args.quiet,
            no_color: args.no_color,
            prefix_files: args.should_prefix_files(),
            poll_interval: args.poll_interval,
            buffer_size: args.buffer_size,
        })
    }

    fn compile_regex_patterns(patterns: &[String], case_insensitive: bool) -> Result<Vec<Regex>> {
        let mut compiled = Vec::new();

        for pattern in patterns {
            let mut regex_builder = regex::RegexBuilder::new(pattern);
            regex_builder.case_insensitive(case_insensitive);
            // ReDoS protection: limit compiled regex size to prevent catastrophic backtracking
            regex_builder.size_limit(REGEX_SIZE_LIMIT);
            // Also limit DFA size for additional protection
            regex_builder.dfa_size_limit(REGEX_SIZE_LIMIT);

            let regex = regex_builder
                .build()
                .with_context(|| format!("Invalid or too complex regex pattern: {}", pattern))?;

            compiled.push(regex);
        }

        Ok(compiled)
    }

    fn parse_color_mappings(mappings: &[(String, String)]) -> Result<HashMap<String, Color>> {
        let mut color_map = HashMap::new();

        for (pattern, color_name) in mappings {
            let color = Self::parse_color(color_name)?;
            color_map.insert(pattern.clone(), color);
        }

        // Add default color mappings if not specified
        Self::add_default_color_mappings(&mut color_map);

        Ok(color_map)
    }

    fn parse_color(color_name: &str) -> Result<Color> {
        match color_name.to_lowercase().as_str() {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "white" => Ok(Color::White),
            _ => Err(anyhow::anyhow!("Unknown color: {}", color_name)),
        }
    }

    fn add_default_color_mappings(color_map: &mut HashMap<String, Color>) {
        let defaults = [
            ("ERROR", Color::Red),
            ("WARN", Color::Yellow),
            ("WARNING", Color::Yellow),
            ("INFO", Color::Green),
            ("DEBUG", Color::Cyan),
            ("TRACE", Color::Magenta),
            ("FATAL", Color::Red),
            ("CRITICAL", Color::Red),
        ];

        for (pattern, color) in defaults {
            color_map.entry(pattern.to_string()).or_insert(color);
        }
    }

    /// Check if a pattern should trigger notifications
    pub fn should_notify_for_pattern(&self, pattern: &str) -> bool {
        self.notify_enabled && self.notify_patterns.contains(&pattern.to_string())
    }

    /// Get color for a pattern
    pub fn get_color_for_pattern(&self, pattern: &str) -> Option<Color> {
        self.color_mappings.get(pattern).copied()
    }

    /// Check if a line should be excluded based on exclude patterns
    pub fn should_exclude(&self, line: &str) -> bool {
        if self.exclude_patterns.is_empty() {
            return false;
        }

        // If regex mode, use compiled exclude regex patterns
        if !self.exclude_regex_patterns.is_empty() {
            for regex in &self.exclude_regex_patterns {
                if regex.is_match(line) {
                    return true;
                }
            }
        } else if self.case_insensitive {
            // Use pre-computed lowercase patterns for case-insensitive matching
            let search_line = line.to_lowercase();
            for pattern in &self.exclude_patterns_lowercase {
                if search_line.contains(pattern) {
                    return true;
                }
            }
        } else {
            // Case-sensitive literal matching (no allocation needed for patterns)
            for pattern in &self.exclude_patterns {
                if line.contains(pattern.as_str()) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_color_parsing() {
        // Test all color mappings to cover the uncovered lines
        assert_eq!(Config::parse_color("yellow").unwrap(), Color::Yellow);
        assert_eq!(Config::parse_color("magenta").unwrap(), Color::Magenta);
        assert_eq!(Config::parse_color("cyan").unwrap(), Color::Cyan);
        assert_eq!(Config::parse_color("white").unwrap(), Color::White);
    }

    #[test]
    fn test_parse_color_unknown_coverage_line_100() {
        // Test unknown color to cover line 100 (_ => Err(...))
        let result = Config::parse_color("unknown_color");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown color: unknown_color"));
    }

    #[test]
    fn test_get_color_for_pattern() {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            completions: None,
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: None,
            notify: false,
            notify_patterns: None,
            quiet: false,
            dry_run: false,
            exclude: None,
            prefix_file: Some(false),
            poll_interval: 1000,
            buffer_size: 8192,
            no_color: false,
            notify_throttle: 0,
        };

        let config = Config::from_args(&args).unwrap();

        // Test that default color mappings work
        assert_eq!(config.get_color_for_pattern("ERROR"), Some(Color::Red));
        assert_eq!(config.get_color_for_pattern("WARN"), Some(Color::Yellow));
        assert_eq!(config.get_color_for_pattern("INFO"), Some(Color::Green));
        assert_eq!(config.get_color_for_pattern("DEBUG"), Some(Color::Cyan));
        assert_eq!(config.get_color_for_pattern("UNKNOWN"), None);
    }

    #[test]
    fn test_should_exclude_literal() {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            completions: None,
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: None,
            notify: false,
            notify_patterns: None,
            quiet: false,
            dry_run: false,
            exclude: Some("DEBUG,TRACE".to_string()),
            prefix_file: Some(false),
            poll_interval: 1000,
            buffer_size: 8192,
            no_color: false,
            notify_throttle: 0,
        };

        let config = Config::from_args(&args).unwrap();

        assert!(config.should_exclude("DEBUG: Some debug message"));
        assert!(config.should_exclude("TRACE: Some trace message"));
        assert!(!config.should_exclude("ERROR: Some error message"));
        assert!(!config.should_exclude("INFO: Some info message"));
    }

    #[test]
    fn test_should_exclude_case_insensitive() {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            completions: None,
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: true,
            color_map: None,
            notify: false,
            notify_patterns: None,
            quiet: false,
            dry_run: false,
            exclude: Some("debug".to_string()),
            prefix_file: Some(false),
            poll_interval: 1000,
            buffer_size: 8192,
            no_color: false,
            notify_throttle: 0,
        };

        let config = Config::from_args(&args).unwrap();

        assert!(config.should_exclude("DEBUG: Some debug message"));
        assert!(config.should_exclude("debug: Some debug message"));
        assert!(!config.should_exclude("ERROR: Some error message"));
    }

    #[test]
    fn test_should_exclude_regex() {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            completions: None,
            patterns: "ERROR".to_string(),
            regex: true,
            case_insensitive: false,
            color_map: None,
            notify: false,
            notify_patterns: None,
            quiet: false,
            dry_run: false,
            exclude: Some(r"DEBUG|TRACE".to_string()),
            prefix_file: Some(false),
            poll_interval: 1000,
            buffer_size: 8192,
            no_color: false,
            notify_throttle: 0,
        };

        let config = Config::from_args(&args).unwrap();

        assert!(config.should_exclude("DEBUG: Some debug message"));
        assert!(config.should_exclude("TRACE: Some trace message"));
        assert!(!config.should_exclude("ERROR: Some error message"));
    }

    #[test]
    fn test_should_exclude_empty() {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            completions: None,
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: None,
            notify: false,
            notify_patterns: None,
            quiet: false,
            dry_run: false,
            exclude: None,
            prefix_file: Some(false),
            poll_interval: 1000,
            buffer_size: 8192,
            no_color: false,
            notify_throttle: 0,
        };

        let config = Config::from_args(&args).unwrap();

        assert!(!config.should_exclude("DEBUG: Some debug message"));
        assert!(!config.should_exclude("ERROR: Some error message"));
    }
}
