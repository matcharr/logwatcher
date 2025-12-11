# Changelog

All notable changes to LogWatcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-12-11

### Changed
- **Performance optimization**: Pre-compute lowercase exclude patterns at config init
- **Better statistics**: Added `lines_excluded` counter for visibility into filtering
- **Improved accuracy**: `lines_processed` now only counts non-excluded lines
- Updated dependencies: `notify` 6→8, `crossterm` 0.27→0.29

### Fixed
- Clippy `unnecessary_unwrap` lint (compatibility with beta Rust)
- CI formatting and linting issues

### Documentation
- Added Zsh fpath setup instructions for shell completions

## [0.2.0] - 2025-09-13

### Added
- **Exclude patterns** (`--exclude` / `-e`) for inverse matching - filter out noisy log entries
- **Shell completions** (`--completions`) for bash, zsh, fish, and PowerShell
- **Docker support** with multi-stage Dockerfile for containerized deployment
- **ReDoS protection** with regex size limits to prevent catastrophic backtracking
- **Exceptional test coverage: 90.65%** (388/428 lines covered)
- Comprehensive test suite with 114 unit tests + 13 integration tests
- Performance benchmarks with detailed metrics
- CI/CD pipeline with multi-platform builds (Linux, macOS, Windows)
- Code coverage reporting with Codecov integration
- Cross-platform desktop notifications (macOS, Linux, Windows)
- Advanced pattern matching with regex support
- Case-insensitive pattern matching
- File rotation detection and handling
- Dry-run mode for testing without notifications
- Quiet mode for suppressing non-matching lines
- Custom color mappings for patterns
- Throttled notifications to prevent spam
- Comprehensive error handling and logging

### Changed
- **Major version bump** reflecting significant improvements
- Version now correctly reported from Cargo.toml using `env!("CARGO_PKG_VERSION")`
- Updated performance claims with actual benchmark results
- Improved documentation and examples
- Enhanced CLI interface with better argument handling
- Optimized file watching with efficient polling
- Better error messages and user feedback

### Fixed
- Version mismatch between CLI and Cargo.toml (was showing 0.1.0 instead of 0.2.0)
- Cross-platform compatibility issues
- Memory leaks in long-running processes
- File handle management during rotation
- Notification system reliability across platforms

### Removed
- SonarQube integration (replaced with native Rust tools)
- Deprecated GitHub Actions workflows

## [Unreleased]

## [0.1.0] - 2025-01-13

### Added
- Initial release of LogWatcher
- Real-time log file monitoring
- Pattern highlighting with ANSI colors
- Desktop notifications (cross-platform)
- Multiple file support
- File rotation handling
- Regex and literal pattern matching
- Dry-run mode
- Quiet mode
- Configurable polling intervals
- Buffer size configuration
- Notification throttling
- Cross-platform support (Linux, macOS, Windows)
- Comprehensive CLI interface with help system
- Performance benchmarks
- Integration tests
- CI/CD pipeline
- Code coverage reporting
- Static analysis integration

### Features
- **File Monitoring**: Watch single or multiple log files in real-time
- **Pattern Matching**: Support for both regex and literal string patterns
- **Color Highlighting**: ANSI color output for matched patterns
- **Desktop Notifications**: Cross-platform notifications with throttling
- **File Rotation**: Automatic detection and handling of log file rotation
- **Performance**: Optimized for high-frequency log processing
- **Cross-Platform**: Works on Linux, macOS, and Windows

### Performance
- Pattern matching: ~0.7-11µs per line
- Binary size: 2.0MB (release mode)
- Memory efficient streaming I/O
- Async file operations

### Installation
- From crates.io: `cargo install logwatcher`
- From source: `git clone && cargo build --release`
- Pre-built binaries available in GitHub releases
