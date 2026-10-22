pub mod wingpanel;
pub mod dock;
pub mod start_menu;
pub mod session;

pub use wingpanel::{render_wingpanel, WINGPANEL_HEIGHT};
pub use dock::{render_dock, DOCK_WIDTH, DOCK_HEIGHT};
pub use start_menu::render_start_menu;
pub use session::AladdinSession;
