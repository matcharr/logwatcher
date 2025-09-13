# LogWatcher

[![CI](https://github.com/matcharr/logwatcher/actions/workflows/basic.yml/badge.svg)](https://github.com/matcharr/logwatcher/actions)
[![codecov](https://codecov.io/gh/matcharr/logwatcher/branch/main/graph/badge.svg?timestamp=1757736369)](https://codecov.io/gh/matcharr/logwatcher)
[![Crates.io](https://img.shields.io/crates/v/log-watcher.svg)](https://crates.io/crates/log-watcher)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful CLI tool for real-time log file monitoring with pattern highlighting and desktop notifications.

## Features

- **Real-time file tailing** - Monitor log files as they're written
- **Pattern highlighting** - Color-code different log levels and patterns
- **Desktop notifications** - Get alerted for critical patterns even when terminal isn't visible
- **Multiple file support** - Monitor multiple log files simultaneously
- **File rotation handling** - Automatically detect and handle log rotation
- **Regex support** - Use regular expressions for advanced pattern matching
- **Dry-run mode** - Test patterns without continuous monitoring
- **Throttled notifications** - Prevent notification spam

## Installation

### From Source

```bash
git clone https://github.com/matcharr/logwatcher.git
cd logwatcher
cargo build --release
sudo cp target/release/logwatcher /usr/local/bin/
```

### Using Cargo

```bash
cargo install log-watcher
```

## Quick Start

### Basic Usage

Monitor a single log file for ERROR and WARN patterns:

```bash
logwatcher -f /var/log/app.log
```

### Multiple Files

Monitor multiple log files simultaneously:

```bash
logwatcher -f app.log -f error.log -f access.log
```

### Custom Patterns

Specify custom patterns to match:

```bash
logwatcher -f app.log -p "ERROR,CRITICAL,timeout"
```

### Regex Patterns

Use regular expressions for advanced matching:

```bash
logwatcher -f app.log -r -p "user_id=\d+|session_\w+"
```

### Dry-Run Mode

Test patterns on existing file content:

```bash
logwatcher -f app.log --dry-run -p "ERROR,WARN"
```

### Quiet Mode

Only show lines that match patterns:

```bash
logwatcher -f app.log -q -p "ERROR"
```

## Command Line Options

### Required Arguments

| Flag | Short | Description |
|------|-------|-------------|
| `--file` | `-f` | Path(s) to log file(s) to watch (can be specified multiple times) |

### Pattern Configuration

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--pattern` | `-p` | `ERROR,WARN` | Comma-separated patterns to match |
| `--regex` | `-r` | `false` | Treat patterns as regular expressions |
| `--case-insensitive` | `-i` | `false` | Case-insensitive pattern matching |
| `--color-map` | `-c` | (see below) | Custom pattern:color mappings |

### Notification Control

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--notify` | `-n` | `true` | Enable desktop notifications |
| `--notify-patterns` | | (all patterns) | Specific patterns that trigger notifications |
| `--notify-throttle` | | `5` | Maximum notifications per second |

### Output Control

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--dry-run` | `-d` | `false` | Preview mode (no tailing, no notifications) |
| `--quiet` | `-q` | `false` | Suppress non-matching lines |
| `--no-color` | | `false` | Disable ANSI colors |
| `--prefix-file` | | `auto` | Prefix lines with filename |

### Performance Tuning

| Flag | Default | Description |
|------|---------|-------------|
| `--poll-interval` | `100` | File polling interval in milliseconds |
| `--buffer-size` | `8192` | Read buffer size in bytes |

## Default Color Mappings

- **ERROR** → Red
- **WARN/WARNING** → Yellow
- **INFO** → Green
- **DEBUG** → Cyan
- **TRACE** → Magenta
- **FATAL/CRITICAL** → Red + Bold

## Examples

### Basic Error Monitoring

```bash
# Monitor application logs for errors
logwatcher -f /var/log/app.log

# Output:
# [2025-01-07 15:00:01] Starting application server...
# [2025-01-07 15:00:02] Database connection established
# [2025-01-07 15:00:03] ERROR Failed to bind to port 8080  # (shown in red)
# [Desktop notification appears: "ERROR detected in app.log"]
```

### Multiple Files with Custom Patterns

```bash
logwatcher -f app.log -f nginx.log -p "ERROR,404,timeout" --color-map "404:yellow,timeout:magenta"

# Output:
# [app.log] [2025-01-07 15:00:01] Request processed successfully
# [nginx.log] [2025-01-07 15:00:02] 404 Not Found: /api/users  # (shown in yellow)
# [app.log] [2025-01-07 15:00:03] ERROR Database timeout  # (shown in red, "timeout" in magenta)
```

### Dry-Run Testing

```bash
logwatcher -f app.log --dry-run -p "ERROR,WARN"

# Output:
# Reading existing content from app.log...
# [DRY-RUN] Line 42: ERROR Connection refused  # (shown in red)
# [DRY-RUN] Line 89: WARN Memory usage at 85%  # (shown in yellow)
# Found 2 matching lines (1 ERROR, 1 WARN)
# Dry-run complete. No notifications sent.
```

### Regex Pattern Matching

```bash
logwatcher -f app.log -q -r -p "user_id=\d+|session_\w+"

# Output:
# [2025-01-07 15:00:01] Login successful for user_id=12345
# [2025-01-07 15:00:15] Order placed by user_id=67890
# [2025-01-07 15:00:30] Session created: session_abc123
```

## File Rotation Handling

LogWatcher automatically detects and handles log file rotation:

- **Truncation detection** - Detects when file size decreases
- **Automatic reopening** - Reopens files after rotation
- **Rotation notifications** - Logs when rotation is detected

```bash
# LogWatcher automatically handles rotation
logwatcher -f /var/log/app.log

# When rotation occurs:
# Warning: File rotation detected for /var/log/app.log
# Info: Reopened file: /var/log/app.log
```

## Desktop Notifications

LogWatcher supports desktop notifications on Linux, macOS, and Windows:

- **Pattern-based alerts** - Notifications for specific patterns
- **Throttling** - Prevents notification spam
- **Truncated content** - Long lines are truncated in notifications
- **Respects system settings** - Honors Do Not Disturb settings

### Notification Examples

```bash
# Enable notifications for all patterns
logwatcher -f app.log --notify

# Only notify for critical patterns
logwatcher -f app.log --notify-patterns "ERROR,FATAL,CRITICAL"

# Throttle notifications to 2 per second
logwatcher -f app.log --notify-throttle 2
```

## Performance Considerations

- **Memory efficient** - Uses streaming I/O for large files
- **Configurable polling** - Adjust polling interval for your needs
- **Buffer sizing** - Tune buffer size for optimal performance
- **Fast pattern matching** - ~0.7-11µs per line (benchmarked)
- **Small binary** - Only 2.0MB in release mode
- **Async I/O** - Non-blocking file operations

```bash
# Optimize for high-frequency logs
logwatcher -f app.log --poll-interval 50 --buffer-size 16384

# Optimize for large files
logwatcher -f large.log --poll-interval 500 --buffer-size 32768
```

## Troubleshooting

### Common Issues

**File not found:**
```bash
# Check file permissions and path
ls -la /var/log/app.log
logwatcher -f /var/log/app.log
```

**No notifications:**
```bash
# Test notification system
logwatcher -f app.log --dry-run --notify -p "TEST"
```

**High CPU usage:**
```bash
# Increase polling interval
logwatcher -f app.log --poll-interval 500
```

**Memory usage:**
```bash
# Reduce buffer size
logwatcher -f app.log --buffer-size 4096
```

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug logwatcher -f app.log
```

## Exit Codes

- **0** - Success
- **1** - File access error
- **2** - Invalid pattern/regex
- **3** - Notification system error
- **130** - Interrupted (Ctrl+C)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Testing & Coverage

LogWatcher has comprehensive test coverage:

- **50 Tests Total**: 13 integration tests + 36 unit tests + 1 main test
- **Integration Tests**: End-to-end CLI functionality testing
- **Unit Tests**: Core component testing (matcher, highlighter, notifier, watcher, etc.)
- **Performance Benchmarks**: Real performance measurements
- **Cross-platform Testing**: Linux, macOS, Windows
- **Coverage**: 80.84% (above 80% professional standard)

### Running Tests

```bash
# Run all tests
cargo test

# Run only integration tests
cargo test --test integration

# Run benchmarks
cargo bench

# Check test coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

Coverage reports are automatically generated and uploaded to [Codecov](https://codecov.io/gh/matcharr/logwatcher) on every commit.

## Changelog

### v0.1.0
- Initial release
- Real-time file tailing
- Pattern highlighting
- Desktop notifications
- File rotation handling
- Regex support
- Dry-run mode
