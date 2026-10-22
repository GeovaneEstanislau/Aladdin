//! Gerenciamento de Damage Tracking (Regiões Danificadas) e Recorte Geométrico (Clipping)

pub use crate::contracts::framebuffer::DirtyRect;

/// Região de recorte para restringir o desenho dentro dos limites de uma janela
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRect {
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl ClipRect {
    pub const fn new(min_x: usize, min_y: usize, max_x: usize, max_y: usize) -> Self {
        Self { min_x, min_y, max_x, max_y }
    }

    pub const fn from_rect(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x + width,
            max_y: y + height,
        }
    }

    pub fn contains_pixel(&self, x: usize, y: usize) -> bool {
        x >= self.min_x && x < self.max_x && y >= self.min_y && y < self.max_y
    }

    pub fn intersect(&self, other: &ClipRect) -> Option<ClipRect> {
        let min_x = self.min_x.max(other.min_x);
        let min_y = self.min_y.max(other.min_y);
        let max_x = self.max_x.min(other.max_x);
        let max_y = self.max_y.min(other.max_y);

        if min_x < max_x && min_y < max_y {
            Some(ClipRect { min_x, min_y, max_x, max_y })
        } else {
            None
        }
    }
}

/// Gerenciador de regiões danificadas para composição em lote
pub struct DamageTracker {
    current: Option<DirtyRect>,
}

impl DamageTracker {
    pub const fn new() -> Self {
        Self { current: None }
    }

    pub fn add(&mut self, rect: DirtyRect) {
        if rect.width == 0 || rect.height == 0 { return; }
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
