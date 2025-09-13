use crate::config::Config;
use crate::matcher::MatchResult;
use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
pub struct Highlighter {
    config: Config,
    stdout: StandardStream,
    stderr: StandardStream,
}

impl Highlighter {
    pub fn new(config: Config) -> Self {
        let color_choice = if config.no_color {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        };

        Self {
            config,
            stdout: StandardStream::stdout(color_choice),
            stderr: StandardStream::stderr(color_choice),
        }
    }

    pub fn print_line(
        &mut self,
        line: &str,
        filename: Option<&str>,
        match_result: &MatchResult,
        dry_run: bool,
    ) -> Result<()> {
        // Skip non-matching lines in quiet mode
        if self.config.quiet && !match_result.matched {
            return Ok(());
        }

        let mut output_line = String::new();

        // Add dry-run prefix if needed
        if dry_run && match_result.matched {
            output_line.push_str("[DRY-RUN] ");
        }

        // Add filename prefix if needed
        if self.config.prefix_files {
            if let Some(filename) = filename {
                output_line.push_str(&format!("[{}] ", filename));
            }
        }

        // Add the actual line content
        output_line.push_str(line);

        // Print with or without color
        if match_result.matched && match_result.color.is_some() {
            self.print_colored(&output_line, match_result.color.unwrap())?;
        } else {
            self.print_plain(&output_line)?;
        }

        Ok(())
    }

    fn print_colored(&mut self, text: &str, color: Color) -> Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(color)))?;
        writeln!(self.stdout, "{}", text)?;
        self.stdout.reset()?;
        self.stdout.flush()?;
        Ok(())
    }

    fn print_plain(&mut self, text: &str) -> Result<()> {
        writeln!(self.stdout, "{}", text)?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn print_error(&mut self, message: &str) -> Result<()> {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(self.stderr, "Error: {}", message)?;
        self.stderr.reset()?;
        self.stderr.flush()?;
        Ok(())
    }

    pub fn print_warning(&mut self, message: &str) -> Result<()> {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
        writeln!(self.stderr, "Warning: {}", message)?;
        self.stderr.reset()?;
        self.stderr.flush()?;
        Ok(())
    }

    pub fn print_info(&mut self, message: &str) -> Result<()> {
        self.stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
        writeln!(self.stderr, "Info: {}", message)?;
        self.stderr.reset()?;
        self.stderr.flush()?;
        Ok(())
    }

    pub fn print_dry_run_summary(&mut self, matches: &[(String, usize)]) -> Result<()> {
        if matches.is_empty() {
            self.print_info("No matching lines found")?;
            return Ok(());
        }

        self.print_info("Dry-run summary:")?;
        for (pattern, count) in matches {
            self.print_plain(&format!("  {}: {} matches", pattern, count))?;
        }
        self.print_info("Dry-run complete. No notifications sent.")?;
        Ok(())
    }

    pub fn print_startup_info(&mut self) -> Result<()> {
        self.print_info(&format!("Watching {} file(s)", self.config.files.len()))?;

        if !self.config.patterns.is_empty() {
            self.print_info(&format!("Patterns: {}", self.config.patterns.join(", ")))?;
        }

        if self.config.notify_enabled {
            self.print_info("Desktop notifications enabled")?;
        }

        if self.config.dry_run {
            self.print_info("Dry-run mode: reading existing content only")?;
        }

        Ok(())
    }

    pub fn print_file_rotation(&mut self, filename: &str) -> Result<()> {
        self.print_warning(&format!("File rotation detected for {}", filename))?;
        Ok(())
    }

    pub fn print_file_reopened(&mut self, filename: &str) -> Result<()> {
        self.print_info(&format!("Reopened file: {}", filename))?;
        Ok(())
    }

    pub fn print_file_error(&mut self, filename: &str, error: &str) -> Result<()> {
        self.print_error(&format!("Error watching {}: {}", filename, error))?;
        Ok(())
    }

    pub fn print_shutdown_summary(&mut self, stats: &WatcherStats) -> Result<()> {
        self.print_info("Shutdown summary:")?;
        self.print_plain(&format!("  Files watched: {}", stats.files_watched))?;
        self.print_plain(&format!("  Lines processed: {}", stats.lines_processed))?;
        self.print_plain(&format!("  Matches found: {}", stats.matches_found))?;
        self.print_plain(&format!(
            "  Notifications sent: {}",
            stats.notifications_sent
        ))?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct WatcherStats {
    pub files_watched: usize,
    pub lines_processed: usize,
    pub matches_found: usize,
    pub notifications_sent: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use std::path::PathBuf;

    fn create_test_config() -> Config {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: None,
            notify: true,
            notify_patterns: None,
            notify_throttle: 5,
            dry_run: false,
            quiet: false,
            no_color: true, // Disable colors for testing
            prefix_file: None,
            poll_interval: 100,
            buffer_size: 8192,
        };
        Config::from_args(&args).unwrap()
    }

    #[test]
    fn test_print_line_without_match() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);

        let match_result = MatchResult {
            matched: false,
            pattern: None,
            color: None,
            should_notify: false,
        };

        // This should not panic
        highlighter
            .print_line("Normal line", None, &match_result, false)
            .unwrap();
    }

    #[test]
    fn test_print_line_with_match() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);

        let match_result = MatchResult {
            matched: true,
            pattern: Some("ERROR".to_string()),
            color: Some(Color::Red),
            should_notify: true,
        };

        // This should not panic
        highlighter
            .print_line("ERROR: Something went wrong", None, &match_result, false)
            .unwrap();
    }

    #[test]
    fn test_dry_run_prefix() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);

        let match_result = MatchResult {
            matched: true,
            pattern: Some("ERROR".to_string()),
            color: Some(Color::Red),
            should_notify: true,
        };

        // This should not panic
        highlighter
            .print_line("ERROR: Something went wrong", None, &match_result, true)
            .unwrap();
    }

    #[test]
    fn test_print_file_error() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);
        let result = highlighter.print_file_error("test.log", "Permission denied");
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_shutdown_summary() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);
        let stats = WatcherStats {
            files_watched: 2,
            lines_processed: 100,
            matches_found: 5,
            notifications_sent: 3,
        };
        let result = highlighter.print_shutdown_summary(&stats);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_file_rotation() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);
        let result = highlighter.print_file_rotation("test.log");
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_file_reopened() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);
        let result = highlighter.print_file_reopened("test.log");
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_startup_info() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);
        let result = highlighter.print_startup_info();
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_colored_with_custom_color() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);
        let result = highlighter.print_colored("Custom message", Color::Magenta);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_plain() {
        let config = create_test_config();
        let mut highlighter = Highlighter::new(config);
        let result = highlighter.print_plain("Plain message");
        assert!(result.is_ok());
    }
}
