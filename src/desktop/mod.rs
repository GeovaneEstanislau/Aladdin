pub mod wingpanel;
pub mod dock;
pub mod start_menu;
pub mod notifications;
pub mod quick_settings;
pub mod switcher;
pub mod conky;
pub mod shortcuts;
pub mod session;

pub use wingpanel::{render_wingpanel, WINGPANEL_HEIGHT};
pub use dock::{render_dock, DOCK_WIDTH, DOCK_HEIGHT};
pub use conky::render_conky_hud;
pub use shortcuts::{render_desktop_shortcuts, handle_shortcuts_click};
pub use start_menu::render_start_menu;
pub use notifications::{NotificationCenter, ToastNotification, TOAST_WIDTH, TOAST_HEIGHT};
pub use quick_settings::{QuickSettingsPopover, QuickSettingsAction, QUICK_SETTINGS_W, QUICK_SETTINGS_H};
pub use switcher::{AppSwitcherHUD, SwitcherAction, SWITCHER_WIDTH, SWITCHER_MAX_HEIGHT};
pub use session::AladdinSession;
