pub mod cli;
pub mod config;
pub mod highlighter;
pub mod matcher;
pub mod notifier;
pub mod utils;
pub mod watcher;

// 🔒 All commits automatically signed - no manual work needed!

pub use cli::Args;
pub use config::Config;
pub use highlighter::Highlighter;
pub use matcher::Matcher;
pub use notifier::Notifier;
pub use watcher::LogWatcher;
