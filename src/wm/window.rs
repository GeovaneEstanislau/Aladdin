//! Estrutura e Comportamento de Janelas com Recorte de Superfície

use crate::contracts::framebuffer::DirtyRect;
use crate::compositor::ClipRect;

pub const TITLE_BAR_HEIGHT: usize = 24;
pub const BTN_SIZE: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub id: usize,
    pub title: [u8; 48],
    pub title_len: usize,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub is_open: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub saved_x: usize,
    pub saved_y: usize,
    pub saved_width: usize,
    pub saved_height: usize,
    pub is_dragging: bool,
    pub drag_offset_x: usize,
    pub drag_offset_y: usize,
    pub can_minimize: bool,
    pub can_maximize: bool,
    pub can_close: bool,
    pub owner_pid: Option<usize>,
}

impl Window {
    pub const fn new(id: usize, title: &str, x: usize, y: usize, width: usize, height: usize) -> Self {
        let mut title_buf = [0u8; 48];
        let bytes = title.as_bytes();
        let len = if bytes.len() > 48 { 48 } else { bytes.len() };
        let mut i = 0;
        while i < len {
            title_buf[i] = bytes[i];
            i += 1;
        }

        Self {
            id,
            title: title_buf,
            title_len: len,
            x,
            y,
            width,
            height,
            is_open: true,
            is_minimized: false,
            is_maximized: false,
            saved_x: x,
            saved_y: y,
            saved_width: width,
            saved_height: height,
            is_dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
            can_minimize: true,
            can_maximize: true,
            can_close: true,
            owner_pid: None,
        }
    }

    pub fn new_with_client_size(id: usize, title: &str, x: usize, y: usize, client_w: usize, client_h: usize) -> Self {
        let total_w = client_w.saturating_add(2);
        let total_h = client_h.saturating_add(TITLE_BAR_HEIGHT + 2);
        Self::new(id, title, x, y, total_w, total_h)
    }

    pub fn title_str(&self) -> &str {
        core::str::from_utf8(&self.title[..self.title_len]).unwrap_or("Janela")
    }

    pub fn client_width(&self) -> usize {
        self.width.saturating_sub(2)
    }

    pub fn client_height(&self) -> usize {
        self.height.saturating_sub(TITLE_BAR_HEIGHT + 2)
    }

    pub fn bounds(&self) -> DirtyRect {
        DirtyRect::new(self.x, self.y, self.width, self.height)
    }

    /// Retorna a região de recorte da área de cliente (interior da janela)
    pub fn client_clip(&self) -> ClipRect {
        let x0 = self.x + 1;
        let y0 = self.y + TITLE_BAR_HEIGHT + 1;
        let x1 = (self.x + self.width).saturating_sub(1);
        let y1 = (self.y + self.height).saturating_sub(1);
        ClipRect::new(x0, y0, x1, y1)
    }

    pub fn contains_point(&self, px: usize, py: usize) -> bool {
        if !self.is_open || self.is_minimized { return false; }
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    pub fn is_in_title_bar(&self, px: usize, py: usize) -> bool {
        if !self.contains_point(px, py) { return false; }
        py < self.y + TITLE_BAR_HEIGHT
    }

    pub fn is_in_min_btn(&self, px: usize, py: usize) -> bool {
        if !self.can_minimize { return false; }
        let btn_x = self.x + self.width.saturating_sub(64);
        let btn_y = self.y + 4;
        px >= btn_x && px < btn_x + BTN_SIZE && py >= btn_y && py < btn_y + BTN_SIZE
    }

    pub fn is_in_max_btn(&self, px: usize, py: usize) -> bool {
        if !self.can_maximize { return false; }
        let btn_x = self.x + self.width.saturating_sub(44);
        let btn_y = self.y + 4;
        px >= btn_x && px < btn_x + BTN_SIZE && py >= btn_y && py < btn_y + BTN_SIZE
    }

    pub fn is_in_close_btn(&self, px: usize, py: usize) -> bool {
        if !self.can_close { return false; }
        let btn_x = self.x + self.width.saturating_sub(24);
        let btn_y = self.y + 4;
        px >= btn_x && px < btn_x + BTN_SIZE && py >= btn_y && py < btn_y + BTN_SIZE
    }

    pub fn toggle_maximize(&mut self, screen_w: usize, screen_h: usize, top_panel_h: usize, dock_h: usize) -> DirtyRect {
        if !self.can_maximize { return self.bounds(); }
        let old_rect = self.bounds();
        if self.is_maximized {
            self.x = self.saved_x;
            self.y = self.saved_y;
            self.width = self.saved_width;
            self.height = self.saved_height;
            self.is_maximized = false;
        } else {
            self.saved_x = self.x;
            self.saved_y = self.y;
            self.saved_width = self.width;
            self.saved_height = self.height;

            self.x = 0;
            self.y = top_panel_h + 1;
            self.width = screen_w;
            self.height = screen_h.saturating_sub(top_panel_h + dock_h + 10);
            self.is_maximized = true;
        }
        old_rect.union(&self.bounds())
    }
}
