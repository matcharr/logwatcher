use crate::config::Config;
use crate::highlighter::{Highlighter, WatcherStats};
use crate::matcher::Matcher;
use crate::notifier::Notifier;
use crate::utils::{get_file_size, validate_files};
use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Debug)]
pub struct LogWatcher {
    config: Config,
    matcher: Matcher,
    highlighter: Highlighter,
    notifier: Notifier,
    stats: WatcherStats,
}

impl LogWatcher {
    pub fn new(config: Config) -> Self {
        let matcher = Matcher::new(config.clone());
        let highlighter = Highlighter::new(config.clone());
        let notifier = Notifier::new(config.clone());

        Self {
            config,
            matcher,
            highlighter,
            notifier,
            stats: WatcherStats::default(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Validate files
        let valid_files = validate_files(&self.config.files)?;
        self.stats.files_watched = valid_files.len();

        // Print startup information
        self.highlighter.print_startup_info()?;

        if self.config.dry_run {
            self.run_dry_mode(&valid_files).await?;
        } else {
            self.run_tail_mode(&valid_files).await?;
        }

        // Print shutdown summary
        self.highlighter.print_shutdown_summary(&self.stats)?;

        Ok(())
    }

    async fn run_dry_mode(&mut self, files: &[PathBuf]) -> Result<()> {
        info!("Running in dry-run mode");
        
        let mut pattern_counts: HashMap<String, usize> = HashMap::new();

        for file_path in files {
            match self.process_existing_file(file_path).await {
                Ok(matches) => {
                    for (pattern, count) in matches {
                        *pattern_counts.entry(pattern).or_insert(0) += count;
                    }
                }
                Err(e) => {
                    self.highlighter.print_file_error(
                        &file_path.display().to_string(),
                        &e.to_string(),
                    )?;
                }
            }
        }

        // Print summary
        let summary: Vec<(String, usize)> = pattern_counts.into_iter().collect();
        self.highlighter.print_dry_run_summary(&summary)?;

        Ok(())
    }

    async fn run_tail_mode(&mut self, files: &[PathBuf]) -> Result<()> {
        info!("Running in tail mode");

        // Create channels for file events
        let (tx, mut rx) = mpsc::channel::<FileEvent>(100);

        // Start file watchers
        let mut watchers = Vec::new();
        for file_path in files {
            let tx_clone = tx.clone();
            let file_path_clone = file_path.clone();
            
            match self.start_file_watcher(file_path_clone, tx_clone).await {
                Ok(watcher) => watchers.push(watcher),
                Err(e) => {
                    self.highlighter.print_file_error(
                        &file_path.display().to_string(),
                        &e.to_string(),
                    )?;
                }
            }
        }

        // Process file events
        while let Some(event) = rx.recv().await {
            match event {
                FileEvent::NewLine { file_path, line } => {
                    self.process_line(&file_path, &line).await?;
                }
                FileEvent::FileRotated { file_path } => {
                    self.handle_file_rotation(&file_path).await?;
                }
                FileEvent::FileError { file_path, error } => {
                    self.highlighter.print_file_error(
                        &file_path.display().to_string(),
                        &error.to_string(),
                    )?;
                }
            }
        }

        Ok(())
    }

    async fn start_file_watcher(
        &self,
        file_path: PathBuf,
        tx: mpsc::Sender<FileEvent>,
    ) -> Result<RecommendedWatcher> {
        let file_path_clone = file_path.clone();
        let tx_clone = tx.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        // File was modified, we'll poll for new content
                    }
                }
                Err(e) => {
                    let _ = tx_clone.try_send(FileEvent::FileError {
                        file_path: file_path_clone.clone(),
                        error: e,
                    });
                }
            }
        })?;

        watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

        // Start polling task for this file
        let file_path_clone = file_path.clone();
        let tx_clone = tx.clone();
        let poll_interval = self.config.poll_interval;
        let buffer_size = self.config.buffer_size;

        tokio::spawn(async move {
            let mut last_size = get_file_size(&file_path_clone).unwrap_or(0);
            
            loop {
                sleep(Duration::from_millis(poll_interval)).await;
                
                match Self::poll_file_changes(&file_path_clone, last_size, buffer_size).await {
                    Ok((new_size, new_lines)) => {
                        last_size = new_size;
                        
                        for line in new_lines {
                            if let Err(e) = tx_clone.send(FileEvent::NewLine {
                                file_path: file_path_clone.clone(),
                                line,
                            }).await {
                                error!("Failed to send line event: {}", e);
                                break;
                            }
                        }
                    }
                Err(e) => {
                    let _ = tx_clone.send(FileEvent::FileError {
                        file_path: file_path_clone.clone(),
                        error: notify::Error::generic(&e.to_string()),
                    }).await;
                    break;
                }
                }
            }
        });

        Ok(watcher)
    }

    async fn poll_file_changes(
        file_path: &PathBuf,
        last_size: u64,
        buffer_size: usize,
    ) -> Result<(u64, Vec<String>)> {
        let current_size = get_file_size(file_path)?;
        
        if current_size < last_size {
            // File was rotated
            return Err(anyhow::anyhow!("File rotation detected"));
        }
        
        if current_size > last_size {
            // File has new content
            let file = File::open(file_path)?;
            let mut reader = BufReader::with_capacity(buffer_size, file);
            
            // Seek to last position
            reader.seek(SeekFrom::Start(last_size))?;
            
            let mut lines = Vec::new();
            let mut line = String::new();
            
            while reader.read_line(&mut line)? > 0 {
                if !line.trim().is_empty() {
                    lines.push(line.trim().to_string());
                }
                line.clear();
            }
            
            Ok((current_size, lines))
        } else {
            Ok((current_size, Vec::new()))
        }
    }

    async fn process_existing_file(&mut self, file_path: &PathBuf) -> Result<HashMap<String, usize>> {
        let mut pattern_counts: HashMap<String, usize> = HashMap::new();
        
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (_line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            self.stats.lines_processed += 1;
            let match_result = self.matcher.match_line(&line);
            
            if match_result.matched {
                self.stats.matches_found += 1;
                if let Some(pattern) = &match_result.pattern {
                    *pattern_counts.entry(pattern.clone()).or_insert(0) += 1;
                }
                
                self.highlighter.print_line(
                    &line,
                    Some(&file_path.file_name().unwrap().to_string_lossy()),
                    &match_result,
                    true, // dry run
                )?;
            }
        }
        
        Ok(pattern_counts)
    }

    async fn process_line(&mut self, file_path: &PathBuf, line: &str) -> Result<()> {
        self.stats.lines_processed += 1;
        
        let match_result = self.matcher.match_line(line);
        
        if match_result.matched {
            self.stats.matches_found += 1;
            
            // Send notification if needed
            if match_result.should_notify {
                if let Some(pattern) = &match_result.pattern {
                    self.notifier.send_notification(
                        pattern,
                        line,
                        Some(&file_path.file_name().unwrap().to_string_lossy()),
                    ).await?;
                    self.stats.notifications_sent += 1;
                }
            }
        }
        
        // Print the line
        self.highlighter.print_line(
            line,
            Some(&file_path.file_name().unwrap().to_string_lossy()),
            &match_result,
            false, // not dry run
        )?;
        
        Ok(())
    }

    async fn handle_file_rotation(&mut self, file_path: &PathBuf) -> Result<()> {
        self.highlighter.print_file_rotation(&file_path.display().to_string())?;
        
        // Wait a bit for the new file to be created
        sleep(Duration::from_millis(1000)).await;
        
        // Try to reopen the file
        if file_path.exists() {
            self.highlighter.print_file_reopened(&file_path.display().to_string())?;
        } else {
            self.highlighter.print_file_error(
                &file_path.display().to_string(),
                "File not found after rotation",
            )?;
        }
        
        Ok(())
    }
}

#[derive(Debug)]
enum FileEvent {
    NewLine {
        file_path: PathBuf,
        line: String,
    },
    FileRotated {
        file_path: PathBuf,
    },
    FileError {
        file_path: PathBuf,
        error: notify::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_config() -> Config {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: None,
            notify: false,
            notify_patterns: None,
            notify_throttle: 5,
            dry_run: true,
            quiet: false,
            no_color: true,
            prefix_file: None,
            poll_interval: 100,
            buffer_size: 8192,
        };
        Config::from_args(&args).unwrap()
    }

    #[tokio::test]
    async fn test_dry_run_mode() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "This is an ERROR message").unwrap();
        writeln!(temp_file, "This is a normal message").unwrap();
        writeln!(temp_file, "Another ERROR message").unwrap();
        temp_file.flush().unwrap();

        let mut config = create_test_config();
        config.files = vec![temp_file.path().to_path_buf()];
        
        let mut watcher = LogWatcher::new(config);
        let result = watcher.run().await;
        
        assert!(result.is_ok());
        assert_eq!(watcher.stats.matches_found, 2);
    }

    #[test]
    fn test_poll_file_changes() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line 1").unwrap();
        temp_file.flush().unwrap();
        
        let initial_size = get_file_size(temp_file.path()).unwrap();
        
        writeln!(temp_file, "line 2").unwrap();
        temp_file.flush().unwrap();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(LogWatcher::poll_file_changes(
            &temp_file.path().to_path_buf(),
            initial_size,
            1024,
        ));
        
        assert!(result.is_ok());
        let (new_size, lines) = result.unwrap();
        assert!(new_size > initial_size);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "line 2");
    }
}
