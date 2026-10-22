//! Gerenciamento de Damage Tracking (Regiões Danificadas)

pub use crate::contracts::framebuffer::DirtyRect;

/// Gerenciador de regiões danificadas para composição em lote
pub struct DamageTracker {
    current: Option<DirtyRect>,
}

impl DamageTracker {
    pub const fn new() -> Self {
        Self { current: None }
    }

    pub fn add(&mut self, rect: DirtyRect) {
        self.current = match self.current {
            Some(existing) => Some(existing.union(&rect)),
            None => Some(rect),
        };
    }

    pub fn take(&mut self) -> Option<DirtyRect> {
        self.current.take()
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_none()
    }
}
