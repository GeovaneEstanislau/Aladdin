pub mod terminal;
pub mod files;
pub mod monitor;
pub mod about;
pub mod settings;

pub use terminal::TerminalApp;
pub use files::{FileManagerApp, CachedFileEntry};
pub use monitor::SystemMonitorApp;
pub use about::AboutApp;
pub use settings::SettingsApp;
