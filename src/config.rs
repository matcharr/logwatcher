use crate::cli::Args;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use termcolor::Color;

#[derive(Debug, Clone)]
pub struct Config {
    pub files: Vec<PathBuf>,
    pub patterns: Vec<String>,
    pub regex_patterns: Vec<Regex>,
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

        // Validate and compile regex patterns if needed
        let regex_patterns = if args.regex {
            Self::compile_regex_patterns(&patterns, args.case_insensitive)?
        } else {
            vec![]
        };

        // Parse color mappings
        let color_mappings = Self::parse_color_mappings(&args.color_mappings())?;

        Ok(Config {
            files: args.files().to_vec(),
            patterns,
            regex_patterns,
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

            let regex = regex_builder
                .build()
                .with_context(|| format!("Invalid regex pattern: {}", pattern))?;

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
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: None,
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

        let config = Config::from_args(&args).unwrap();

        // Test that default color mappings work
        assert_eq!(config.get_color_for_pattern("ERROR"), Some(Color::Red));
        assert_eq!(config.get_color_for_pattern("WARN"), Some(Color::Yellow));
        assert_eq!(config.get_color_for_pattern("INFO"), Some(Color::Green));
        assert_eq!(config.get_color_for_pattern("DEBUG"), Some(Color::Cyan));
        assert_eq!(config.get_color_for_pattern("UNKNOWN"), None);
    }
}
