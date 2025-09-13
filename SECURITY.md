# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in LogWatcher, please report it responsibly:

1. **Do not** open a public issue
2. Email the maintainer directly: mathis.charretier@protonmail.com
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Security Considerations

LogWatcher is designed with security in mind:

- **File Access**: Only reads files specified by the user
- **No Network Access**: Does not send data over the network
- **Local Processing**: All pattern matching and processing happens locally
- **Minimal Permissions**: Only requires read access to log files

## Security Best Practices

When using LogWatcher:

1. **File Permissions**: Ensure log files have appropriate permissions
2. **Pattern Validation**: Be careful with regex patterns to avoid ReDoS attacks
3. **Resource Limits**: Monitor memory usage with large log files
4. **Regular Updates**: Keep LogWatcher updated to the latest version

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 1 week
- **Fix Development**: Within 2-4 weeks (depending on severity)
- **Release**: As soon as fix is tested and verified

## Security Features

- Input validation for all CLI arguments
- Safe file handling with proper error checking
- Memory-efficient streaming to prevent OOM attacks
- No execution of external commands or scripts
