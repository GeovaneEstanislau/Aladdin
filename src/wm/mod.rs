pub mod window;
pub mod manager;

pub use window::{Window, TITLE_BAR_HEIGHT, BTN_SIZE};
pub use manager::{WindowManager, MAX_WINDOWS, WindowOpError};
