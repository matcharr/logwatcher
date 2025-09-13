use criterion::{black_box, criterion_group, criterion_main, Criterion};
use log_watcher::cli::Args;
use log_watcher::config::Config;
use log_watcher::matcher::Matcher;
use std::path::PathBuf;

fn create_test_config(patterns: &str, regex: bool) -> Config {
    let args = Args {
        files: vec![PathBuf::from("test.log")],
        patterns: patterns.to_string(),
        regex,
        case_insensitive: false,
        color_map: None,
        notify: false,
        notify_patterns: None,
        notify_throttle: 5,
        dry_run: false,
        quiet: false,
        no_color: true,
        prefix_file: None,
        poll_interval: 100,
        buffer_size: 8192,
    };
    Config::from_args(&args).unwrap()
}

fn benchmark_literal_matching(c: &mut Criterion) {
    let config = create_test_config("ERROR,WARN,INFO", false);
    let matcher = Matcher::new(config);

    let test_lines = vec![
        "This is a normal log line",
        "ERROR: Something went wrong",
        "WARN: This is a warning",
        "INFO: This is informational",
        "DEBUG: This is debug information",
        "TRACE: This is trace information",
    ];

    c.bench_function("literal_matching", |b| {
        b.iter(|| {
            for line in &test_lines {
                black_box(matcher.match_line(line));
            }
        })
    });
}

fn benchmark_regex_matching(c: &mut Criterion) {
    let config = create_test_config(r"user_id=\d+|session_\w+|error_\d+", true);
    let matcher = Matcher::new(config);

    let test_lines = vec![
        "Login successful for user_id=12345",
        "Session created: session_abc123",
        "Error occurred: error_404",
        "Normal log message",
        "Another user_id=67890",
        "Session expired: session_def456",
    ];

    c.bench_function("regex_matching", |b| {
        b.iter(|| {
            for line in &test_lines {
                black_box(matcher.match_line(line));
            }
        })
    });
}

fn benchmark_case_insensitive_matching(c: &mut Criterion) {
    let config = create_test_config("ERROR,WARN,INFO", false);
    let mut config = config;
    config.case_insensitive = true;
    let matcher = Matcher::new(config);

    let test_lines = vec![
        "This is a normal log line",
        "error: Something went wrong",
        "WARN: This is a warning",
        "info: This is informational",
        "ERROR: This is an error",
        "warn: This is another warning",
    ];

    c.bench_function("case_insensitive_matching", |b| {
        b.iter(|| {
            for line in &test_lines {
                black_box(matcher.match_line(line));
            }
        })
    });
}

fn benchmark_multiple_patterns(c: &mut Criterion) {
    let config = create_test_config("ERROR,WARN,INFO,DEBUG,TRACE,FATAL,CRITICAL", false);
    let matcher = Matcher::new(config);

    let test_lines = vec![
        "This is a normal log line",
        "ERROR: Critical error occurred",
        "WARN: Warning message",
        "INFO: Information message",
        "DEBUG: Debug information",
        "TRACE: Trace information",
        "FATAL: Fatal error",
        "CRITICAL: Critical system failure",
    ];

    c.bench_function("multiple_patterns", |b| {
        b.iter(|| {
            for line in &test_lines {
                black_box(matcher.match_line(line));
            }
        })
    });
}

fn benchmark_long_line_matching(c: &mut Criterion) {
    let config = create_test_config("ERROR", false);
    let matcher = Matcher::new(config);

    let long_line = "This is a very long log line that contains a lot of information about what happened in the system. It includes details about the request, the user, the timestamp, and various other metadata. ERROR: Something went wrong in the middle of this long line. The rest of the line continues with more information about the context and the state of the system at the time of the error.";

    let test_lines = vec![long_line; 100];

    c.bench_function("long_line_matching", |b| {
        b.iter(|| {
            for line in &test_lines {
                black_box(matcher.match_line(line));
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_literal_matching,
    benchmark_regex_matching,
    benchmark_case_insensitive_matching,
    benchmark_multiple_patterns,
    benchmark_long_line_matching
);
criterion_main!(benches);
