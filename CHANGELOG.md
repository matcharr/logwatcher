# Changelog

All notable changes to LogWatcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Performance benchmarks
- Comprehensive test suite
- CI/CD pipeline with multi-platform builds
- Code coverage reporting
- Static analysis with SonarCloud

### Changed
- Updated performance claims with actual benchmark results
- Improved documentation and examples

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
