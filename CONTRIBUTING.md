# Contributing to LogWatcher

Thank you for your interest in contributing to LogWatcher! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/logwatcher.git`
3. Create a feature branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Run clippy: `cargo clippy`
7. Format code: `cargo fmt`
8. Commit your changes: `git commit -m "Add your feature"`
9. Push to your fork: `git push origin feature/your-feature-name`
10. Create a Pull Request

## Development Setup

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/matcharr/logwatcher.git
cd logwatcher

# Build the project
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Install development tools
rustup component add clippy rustfmt
```

## Code Style

- Follow Rust's official style guidelines
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Write tests for new functionality
- Add documentation for public APIs

## Testing

- All tests must pass: `cargo test`
- Integration tests must pass: `cargo test --test integration`
- Benchmarks should not regress: `cargo bench`

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Add tests for new features
4. Update CHANGELOG.md if applicable
5. Request review from maintainers

## Reporting Issues

When reporting issues, please include:
- Operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected vs actual behavior
- Log output (if applicable)

## Feature Requests

For feature requests, please:
- Check existing issues first
- Provide a clear description
- Explain the use case
- Consider implementation complexity

## License

By contributing to LogWatcher, you agree that your contributions will be licensed under the MIT License.
