#![no_std]

extern crate alloc;

pub mod contracts;
pub mod compositor;
pub mod font;
pub mod wm;
pub mod apps;
pub mod desktop;

pub use contracts::*;
pub use compositor::*;
pub use font::*;
pub use wm::*;
pub use apps::*;
pub use desktop::*;
