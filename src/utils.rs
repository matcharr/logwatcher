use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Read all lines from a file
pub fn read_file_from_end<P: AsRef<Path>>(path: P, _buffer_size: usize) -> Result<Vec<String>> {
    let file = File::open(&path)
        .with_context(|| format!("Failed to open file: {}", path.as_ref().display()))?;

    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line_result in reader.lines() {
        let line = line_result?;
        if !line.trim().is_empty() {
            lines.push(line.trim().to_string());
        }
    }

    Ok(lines)
}

/// Check if a file exists and is readable
pub fn is_file_readable<P: AsRef<Path>>(path: P) -> bool {
    File::open(path).is_ok()
}

/// Get file size
pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let metadata = std::fs::metadata(path).with_context(|| "Failed to get file metadata")?;
    Ok(metadata.len())
}

/// Check if a file has been rotated (size decreased)
pub fn is_file_rotated<P: AsRef<Path>>(path: P, previous_size: u64) -> Result<bool> {
    let current_size = get_file_size(path)?;
    Ok(current_size < previous_size)
}

/// Validate that all files exist and are readable
pub fn validate_files<P: AsRef<Path> + Clone>(files: &[P]) -> Result<Vec<P>> {
    let mut valid_files = Vec::new();
    let mut errors = Vec::new();

    for file in files {
        if is_file_readable(file) {
            valid_files.push(file.clone());
        } else {
            errors.push(format!("File not readable: {}", file.as_ref().display()));
        }
    }

    if valid_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No valid files to watch: {}",
            errors.join(", ")
        ));
    }

    if !errors.is_empty() {
        eprintln!(
            "Warning: Some files are not accessible: {}",
            errors.join(", ")
        );
    }

    Ok(valid_files)
}

/// Format file size in human-readable format
pub fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Get filename from path
pub fn get_filename<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Check if a path is a symlink
pub fn is_symlink<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref()
        .symlink_metadata()
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

/// Resolve symlink to actual path
pub fn resolve_symlink<P: AsRef<Path>>(path: P) -> Result<std::path::PathBuf> {
    let resolved = path
        .as_ref()
        .read_link()
        .with_context(|| format!("Failed to read symlink: {}", path.as_ref().display()))?;
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(1023), "1023 B");
    }

    #[test]
    fn test_get_filename() {
        assert_eq!(get_filename("/path/to/file.log"), "file.log");
        assert_eq!(get_filename("file.log"), "file.log");
        assert_eq!(get_filename("/"), "unknown");
    }

    #[test]
    fn test_validate_files() {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        // Test with valid file
        let valid_files = vec![temp_path];
        let result = validate_files(&valid_files);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);

        // Test with invalid file
        let invalid_files = vec![std::path::Path::new("/nonexistent/file.log")];
        let result = validate_files(&invalid_files);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_from_end() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line 1").unwrap();
        writeln!(temp_file, "line 2").unwrap();
        writeln!(temp_file, "line 3").unwrap();
        temp_file.flush().unwrap();

        let lines = read_file_from_end(temp_file.path(), 1024).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line 1");
        assert_eq!(lines[1], "line 2");
        assert_eq!(lines[2], "line 3");
    }
}
