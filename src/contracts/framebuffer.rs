//! Contrato de Framebuffer e Apresentação de Vídeo

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl DirtyRect {
    pub const fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self { x, y, width, height }
    }

    pub fn union(&self, other: &DirtyRect) -> DirtyRect {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);
        DirtyRect {
            x: min_x,
            y: min_y,
            width: max_x.saturating_sub(min_x),
            height: max_y.saturating_sub(min_y),
        }
    }

    pub fn intersects(&self, other: &DirtyRect) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    pub fn intersect(&self, other: &DirtyRect) -> Option<DirtyRect> {
        if !self.intersects(other) {
            return None;
        }
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = (self.x + self.width).min(other.x + other.width);
        let y1 = (self.y + self.height).min(other.y + other.height);
        if x1 > x0 && y1 > y0 {
            Some(DirtyRect {
                x: x0,
                y: y0,
                width: x1 - x0,
                height: y1 - y0,
            })
        } else {
            None
        }
    }

    pub fn clamp(&self, max_w: usize, max_h: usize) -> DirtyRect {
        let x = self.x.min(max_w);
        let y = self.y.min(max_h);
        let width = self.width.min(max_w.saturating_sub(x));
        let height = self.height.min(max_h.saturating_sub(y));
        DirtyRect { x, y, width, height }
    }
}

/// Formato de pixel suportado pelo Framebuffer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Argb32,
    Rgb24,
}

/// Metadados do Framebuffer físico
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfo {
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bpp: u16,
    pub format: PixelFormat,
}

/// Trait implementado pelo kernel/plataforma para fornecer saída gráfica ao Aladdin
pub trait FramebufferDevice {
    /// Retorna (width, height, pitch_bytes)
    fn dimensions(&self) -> (usize, usize, usize);

    /// Apresenta uma região retangular suja do backbuffer na VRAM (caminho rápido)
    fn swap_rect(&mut self, rect: &DirtyRect, buffer: &[u32]);

    /// Apresenta o buffer completo na VRAM
    fn swap_full(&mut self, buffer: &[u32]);
}
