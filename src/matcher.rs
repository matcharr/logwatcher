use crate::config::Config;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub matched: bool,
    pub pattern: Option<String>,
    pub color: Option<termcolor::Color>,
    pub should_notify: bool,
}

#[derive(Debug)]
pub struct Matcher {
    config: Config,
    literal_patterns: Vec<String>,
    regex_patterns: Vec<Regex>,
    pattern_colors: HashMap<String, termcolor::Color>,
}

impl Matcher {
    pub fn new(config: Config) -> Self {
        let mut literal_patterns = Vec::new();
        let mut regex_patterns = Vec::new();
        
        if config.regex_patterns.is_empty() {
            // Use literal patterns
            literal_patterns = config.patterns.clone();
        } else {
            // Use regex patterns
            regex_patterns = config.regex_patterns.clone();
        }

        let pattern_colors = config.color_mappings.clone();

        Self {
            config,
            literal_patterns,
            regex_patterns,
            pattern_colors,
        }
    }

    pub fn match_line(&self, line: &str) -> MatchResult {
        if self.config.regex_patterns.is_empty() {
            self.match_literal(line)
        } else {
            self.match_regex(line)
        }
    }

    fn match_literal(&self, line: &str) -> MatchResult {
        let search_line = if self.config.case_insensitive {
            line.to_lowercase()
        } else {
            line.to_string()
        };

        for pattern in &self.literal_patterns {
            let search_pattern = if self.config.case_insensitive {
                pattern.to_lowercase()
            } else {
                pattern.clone()
            };

            if search_line.contains(&search_pattern) {
                let color = self.pattern_colors.get(pattern).copied();
                let should_notify = self.config.should_notify_for_pattern(pattern);
                
                return MatchResult {
                    matched: true,
                    pattern: Some(pattern.clone()),
                    color,
                    should_notify,
                };
            }
        }

        MatchResult {
            matched: false,
            pattern: None,
            color: None,
            should_notify: false,
        }
    }

    fn match_regex(&self, line: &str) -> MatchResult {
        for (i, regex) in self.regex_patterns.iter().enumerate() {
            if regex.is_match(line) {
                let pattern = self.config.patterns.get(i).cloned().unwrap_or_default();
                let color = self.pattern_colors.get(&pattern).copied();
                let should_notify = self.config.should_notify_for_pattern(&pattern);
                
                return MatchResult {
                    matched: true,
                    pattern: Some(pattern),
                    color,
                    should_notify,
                };
            }
        }

        MatchResult {
            matched: false,
            pattern: None,
            color: None,
            should_notify: false,
        }
    }

    /// Check if any pattern matches (for quiet mode filtering)
    pub fn has_match(&self, line: &str) -> bool {
        self.match_line(line).matched
    }

    /// Get all patterns that match a line
    pub fn get_all_matches(&self, line: &str) -> Vec<String> {
        let mut matches = Vec::new();
        
        if self.config.regex_patterns.is_empty() {
            let search_line = if self.config.case_insensitive {
                line.to_lowercase()
            } else {
                line.to_string()
            };

            for pattern in &self.literal_patterns {
                let search_pattern = if self.config.case_insensitive {
                    pattern.to_lowercase()
                } else {
                    pattern.clone()
                };

                if search_line.contains(&search_pattern) {
                    matches.push(pattern.clone());
                }
            }
        } else {
            for (i, regex) in self.regex_patterns.iter().enumerate() {
                if regex.is_match(line) {
                    if let Some(pattern) = self.config.patterns.get(i) {
                        matches.push(pattern.clone());
                    }
                }
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use std::path::PathBuf;

    fn create_test_config(patterns: &str, regex: bool, case_insensitive: bool) -> Config {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            patterns: patterns.to_string(),
            regex,
            case_insensitive,
            color_map: None,
            notify: true,
            notify_patterns: None,
            notify_throttle: 5,
            dry_run: false,
            quiet: false,
            no_color: false,
            prefix_file: None,
            poll_interval: 100,
            buffer_size: 8192,
        };
        Config::from_args(&args).unwrap()
    }

    #[test]
    fn test_literal_matching() {
        let config = create_test_config("ERROR,WARN", false, false);
        let matcher = Matcher::new(config);

        let result = matcher.match_line("This is an ERROR message");
        assert!(result.matched);
        assert_eq!(result.pattern, Some("ERROR".to_string()));

        let result = matcher.match_line("This is a WARN message");
        assert!(result.matched);
        assert_eq!(result.pattern, Some("WARN".to_string()));

        let result = matcher.match_line("This is a normal message");
        assert!(!result.matched);
    }

    #[test]
    fn test_case_insensitive_matching() {
        let config = create_test_config("ERROR", false, true);
        let matcher = Matcher::new(config);

        let result = matcher.match_line("This is an error message");
        assert!(result.matched);
        assert_eq!(result.pattern, Some("ERROR".to_string()));

        let result = matcher.match_line("This is an ERROR message");
        assert!(result.matched);
        assert_eq!(result.pattern, Some("ERROR".to_string()));
    }

    #[test]
    fn test_regex_matching() {
        let config = create_test_config(r"user_id=\d+", true, false);
        let matcher = Matcher::new(config);

        let result = matcher.match_line("Login successful for user_id=12345");
        assert!(result.matched);

        let result = matcher.match_line("Login successful for user_id=abc");
        assert!(!result.matched);
    }

    #[test]
    fn test_multiple_matches() {
        let config = create_test_config("ERROR,WARN", false, false);
        let matcher = Matcher::new(config);

        let matches = matcher.get_all_matches("ERROR: This is a WARN message");
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&"ERROR".to_string()));
        assert!(matches.contains(&"WARN".to_string()));
    }
}
