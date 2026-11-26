pub mod terminal;
pub mod files;
pub mod monitor;
pub mod about;
pub mod settings;
pub mod disks;
pub mod software_manager;
pub mod jpkg;
pub mod network_dns;

pub use terminal::TerminalApp;
pub use files::{FileManagerApp, CachedFileEntry};
pub use monitor::SystemMonitorApp;
pub use about::AboutApp;
pub use settings::SettingsApp;
pub use disks::DiskManagerApp;
pub use software_manager::SoftwareManagerApp;
pub use network_dns::NetworkDnsApp;
