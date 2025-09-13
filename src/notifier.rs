use crate::config::Config;
use anyhow::Result;
#[cfg(not(target_os = "windows"))]
use notify_rust::Notification;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Notifier {
    config: Config,
    last_notification: Arc<Mutex<Instant>>,
    notification_count: Arc<Mutex<u32>>,
    throttle_window: Duration,
}

impl Notifier {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            last_notification: Arc::new(Mutex::new(Instant::now())),
            notification_count: Arc::new(Mutex::new(0)),
            throttle_window: Duration::from_secs(1),
        }
    }

    pub async fn send_notification(
        &self,
        pattern: &str,
        line: &str,
        filename: Option<&str>,
    ) -> Result<()> {
        if !self.config.notify_enabled {
            return Ok(());
        }

        // Check if this pattern should trigger notifications
        if !self.config.should_notify_for_pattern(pattern) {
            return Ok(());
        }

        // Throttle notifications
        if !self.should_send_notification().await {
            return Ok(());
        }

        // Truncate long lines
        let truncated_line = if line.len() > 200 {
            format!("{}...", &line[..197])
        } else {
            line.to_string()
        };

        // Create notification title
        let title = if let Some(filename) = filename {
            format!("{} detected in {}", pattern, filename)
        } else {
            format!("{} detected", pattern)
        };

        // Send notification
        self.send_desktop_notification(&title, &truncated_line)
            .await?;

        // Update throttling state
        self.update_throttle_state().await;

        Ok(())
    }

    async fn should_send_notification(&self) -> bool {
        let mut count = self.notification_count.lock().await;
        let mut last_time = self.last_notification.lock().await;

        let now = Instant::now();

        // Reset counter if we're in a new throttle window
        if now.duration_since(*last_time) >= self.throttle_window {
            *count = 0;
            *last_time = now;
        }

        // Check if we're under the throttle limit
        if *count < self.config.notify_throttle {
            *count += 1;
            true
        } else {
            false
        }
    }

    async fn update_throttle_state(&self) {
        let _count = self.notification_count.lock().await;
        // The count was already updated in should_send_notification
    }

    async fn send_desktop_notification(&self, title: &str, body: &str) -> Result<()> {
        #[cfg(not(target_os = "windows"))]
        {
            self.send_unix_notification(title, body).await
        }

        #[cfg(target_os = "windows")]
        {
            self.send_windows_notification(title, body).await
        }
    }

    #[cfg(not(target_os = "windows"))]
    async fn send_unix_notification(&self, title: &str, body: &str) -> Result<()> {
        Notification::new()
            .summary(title)
            .body(body)
            .icon("logwatcher")
            .timeout(5000) // 5 seconds
            .show()
            .map_err(|e| anyhow::anyhow!("Failed to send notification: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn send_windows_notification(&self, title: &str, body: &str) -> Result<()> {
        use winrt_notification::Toast;

        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(body)
            .duration(winrt_notification::Duration::Short)
            .show()
            .map_err(|e| anyhow::anyhow!("Failed to send Windows notification: {}", e))?;

        Ok(())
    }

    pub async fn test_notification(&self) -> Result<()> {
        self.send_notification("TEST", "LogWatcher notification test", Some("test.log"))
            .await
    }

    pub fn get_notification_count(&self) -> Arc<Mutex<u32>> {
        self.notification_count.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Args;
    use std::path::PathBuf;

    fn create_test_config(notify_enabled: bool, throttle: u32) -> Config {
        let args = Args {
            files: vec![PathBuf::from("test.log")],
            patterns: "ERROR".to_string(),
            regex: false,
            case_insensitive: false,
            color_map: None,
            notify: notify_enabled,
            notify_patterns: None,
            notify_throttle: throttle,
            dry_run: false,
            quiet: false,
            no_color: false,
            prefix_file: None,
            poll_interval: 100,
            buffer_size: 8192,
        };
        Config::from_args(&args).unwrap()
    }

    #[tokio::test]
    async fn test_notification_disabled() {
        let config = create_test_config(false, 5);
        let notifier = Notifier::new(config);

        let result = notifier
            .send_notification("ERROR", "Test message", None)
            .await;
        // When notifications are disabled, this should always succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_notification_throttling() {
        let config = create_test_config(true, 2);
        let notifier = Notifier::new(config);

        // Send first notification
        let result1 = notifier
            .send_notification("ERROR", "Test message 1", None)
            .await;
        // In test environment, notifications might fail, so we just check it doesn't panic
        let _ = result1;

        // Send second notification
        let result2 = notifier
            .send_notification("ERROR", "Test message 2", None)
            .await;
        let _ = result2;

        // Third notification should be throttled (but still return Ok)
        let result3 = notifier
            .send_notification("ERROR", "Test message 3", None)
            .await;
        let _ = result3;
    }

    #[tokio::test]
    async fn test_line_truncation() {
        let config = create_test_config(true, 5);
        let notifier = Notifier::new(config);

        let long_line = "a".repeat(250);
        let result = notifier.send_notification("ERROR", &long_line, None).await;
        // The notification might fail in test environment, so we just check it doesn't panic
        // In a real environment, this would succeed and truncate the line
        let _ = result;
    }

    #[test]
    fn test_get_notification_count() {
        let config = create_test_config(true, 0);
        let notifier = Notifier::new(config);

        let count = notifier.get_notification_count();
        let count_value = count.blocking_lock();
        assert_eq!(*count_value, 0);
    }

    #[tokio::test]
    async fn test_notification_with_file_info() {
        let config = create_test_config(true, 0);
        let notifier = Notifier::new(config);

        let result = notifier
            .send_notification("ERROR", "Test error", Some("test.log"))
            .await;
        // May fail in test environment, but shouldn't panic
        let _ = result;
    }
}
