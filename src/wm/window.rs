//! Estrutura e Comportamento de Janelas

use crate::contracts::framebuffer::DirtyRect;

pub const TITLE_BAR_HEIGHT: usize = 24;
pub const BTN_SIZE: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct Window {
    pub id: usize,
    pub title: &'static str,
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
}

impl Window {
    pub const fn new(id: usize, title: &'static str, x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            id,
            title,
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
        }
    }

    pub fn bounds(&self) -> DirtyRect {
        DirtyRect::new(self.x, self.y, self.width, self.height)
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
        let btn_x = self.x + self.width.saturating_sub(64);
        let btn_y = self.y + 4;
        px >= btn_x && px < btn_x + BTN_SIZE && py >= btn_y && py < btn_y + BTN_SIZE
    }

    pub fn is_in_max_btn(&self, px: usize, py: usize) -> bool {
        let btn_x = self.x + self.width.saturating_sub(44);
        let btn_y = self.y + 4;
        px >= btn_x && px < btn_x + BTN_SIZE && py >= btn_y && py < btn_y + BTN_SIZE
    }

    pub fn is_in_close_btn(&self, px: usize, py: usize) -> bool {
        let btn_x = self.x + self.width.saturating_sub(24);
        let btn_y = self.y + 4;
        px >= btn_x && px < btn_x + BTN_SIZE && py >= btn_y && py < btn_y + BTN_SIZE
    }

    pub fn toggle_maximize(&mut self, screen_w: usize, screen_h: usize, top_panel_h: usize, dock_h: usize) -> DirtyRect {
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
