//! Gerenciador Dinâmico de Janelas, Z-Order e Decorações de Superfície

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::CapabilityMode;
use crate::contracts::input::{ClientEventQueue, ClientWindowEvent};
use super::window::{Window, TITLE_BAR_HEIGHT};

pub const MAX_WINDOWS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowOpError {
    NotFound,
    PermissionDenied,
    InvalidDimensions,
}

pub struct WindowManager {
    pub windows: [Option<Window>; MAX_WINDOWS],
    pub order: [usize; MAX_WINDOWS], // 0 = fundo, MAX-1 = topo
    pub client_buffers: [Option<alloc::vec::Vec<u32>>; MAX_WINDOWS],
    pub client_events: [ClientEventQueue; MAX_WINDOWS],
}

impl WindowManager {
    pub const fn new() -> Self {
        const NONE_BUF: Option<alloc::vec::Vec<u32>> = None;
        Self {
            windows: [None; MAX_WINDOWS],
            order: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            client_buffers: [NONE_BUF; MAX_WINDOWS],
            client_events: [ClientEventQueue::new(); MAX_WINDOWS],
        }
    }

    pub fn top_window_id(&self) -> usize {
        for i in (0..MAX_WINDOWS).rev() {
            let wid = self.order[i];
            if let Some(w) = self.windows[wid].as_ref() {
                if w.is_open && !w.is_minimized {
                    return wid;
                }
            }
        }
        for i in (0..MAX_WINDOWS).rev() {
            let wid = self.order[i];
            if self.windows[wid].is_some() {
                return wid;
            }
        }
        self.order[MAX_WINDOWS - 1]
    }

    pub fn bring_to_front(&mut self, target_idx: usize) {
        if target_idx >= MAX_WINDOWS { return; }
        let mut cur_pos = None;
        for i in 0..MAX_WINDOWS {
            if self.order[i] == target_idx {
                cur_pos = Some(i);
                break;
            }
        }
        if let Some(pos) = cur_pos {
            for i in pos..(MAX_WINDOWS - 1) {
                self.order[i] = self.order[i + 1];
            }
            self.order[MAX_WINDOWS - 1] = target_idx;
        }
    }

    pub fn open_window(&mut self, mut window: Window) -> Result<usize, ()> {
        // Encontra primeiro slot livre
        for i in 0..MAX_WINDOWS {
            if self.windows[i].is_none() {
                window.id = i;
                self.windows[i] = Some(window);
                self.bring_to_front(i);
                return Ok(i);
            }
        }
        Err(())
    }

    pub fn close_window(&mut self, idx: usize) -> Option<DirtyRect> {
        if idx < MAX_WINDOWS {
            let mut cur_pos = None;
            for i in 0..MAX_WINDOWS {
                if self.order[i] == idx {
                    cur_pos = Some(i);
                    break;
                }
            }
            if let Some(pos) = cur_pos {
                for i in (1..=pos).rev() {
                    self.order[i] = self.order[i - 1];
                }
                self.order[0] = idx;
            }
            self.client_buffers[idx] = None;
            self.client_events[idx].clear();
            if let Some(win) = self.windows[idx].take() {
                return Some(win.bounds());
            }
        }
        None
    }

    pub fn create_client_window(
        &mut self,
        owner_pid: usize,
        title: &str,
        x: usize,
        y: usize,
        client_w: usize,
        client_h: usize,
    ) -> Result<usize, ()> {
        if client_w == 0 || client_h == 0 {
            return Err(());
        }
        let client_pixels = client_w.checked_mul(client_h).ok_or(())?;
        if client_pixels > 1024 * 768 {
            return Err(());
        }
        let mut win = Window::new_with_client_size(0, title, x, y, client_w, client_h);
        win.owner_pid = Some(owner_pid);
        let id = self.open_window(win)?;
        self.client_buffers[id] = Some(alloc::vec![0x000F_172A; client_pixels]);
        self.client_events[id].clear();
        Ok(id)
    }

    pub const MAX_CLIENT_BUFFER_PIXELS: usize = 1920 * 1080;

    pub fn update_client_buffer(
        &mut self,
        owner_pid: usize,
        win_id: usize,
        buffer: &[u32],
    ) -> Result<(), WindowOpError> {
        if win_id >= MAX_WINDOWS { return Err(WindowOpError::NotFound); }
        let win = self.windows[win_id].as_ref().ok_or(WindowOpError::NotFound)?;
        if win.owner_pid != Some(owner_pid) {
            return Err(WindowOpError::PermissionDenied);
        }
        if buffer.is_empty() || buffer.len() > Self::MAX_CLIENT_BUFFER_PIXELS {
            return Err(WindowOpError::InvalidDimensions);
        }
        let expected_len = win.client_width() * win.client_height();
        let client_buf = self.client_buffers[win_id].as_mut().ok_or(WindowOpError::NotFound)?;

        if buffer.len() == expected_len {
            if client_buf.len() != expected_len {
                client_buf.resize(expected_len, 0x0018_1825);
            }
            client_buf.copy_from_slice(buffer);
            Ok(())
        } else if buffer.len() == client_buf.len() {
            // Tolerância transitória a redimensionamento em voo: aceita o quadro anterior
            client_buf.copy_from_slice(buffer);
            Ok(())
        } else {
            Err(WindowOpError::InvalidDimensions)
        }
    }

    pub fn close_client_window(
        &mut self,
        owner_pid: usize,
        win_id: usize,
    ) -> Result<(), WindowOpError> {
        if win_id >= MAX_WINDOWS { return Err(WindowOpError::NotFound); }
        let win = self.windows[win_id].as_ref().ok_or(WindowOpError::NotFound)?;
        if win.owner_pid != Some(owner_pid) {
            return Err(WindowOpError::PermissionDenied);
        }
        self.close_window(win_id);
        Ok(())
    }

    pub fn push_client_event(&mut self, win_id: usize, event: ClientWindowEvent) -> bool {
        if win_id < MAX_WINDOWS && self.windows[win_id].is_some() {
            self.client_events[win_id].push(event)
        } else {
            false
        }
    }

    pub fn poll_client_event(
        &mut self,
        owner_pid: usize,
        win_id: usize,
    ) -> Result<Option<ClientWindowEvent>, WindowOpError> {
        if win_id >= MAX_WINDOWS { return Err(WindowOpError::NotFound); }
        let win = self.windows[win_id].as_ref().ok_or(WindowOpError::NotFound)?;
        if win.owner_pid != Some(owner_pid) {
            return Err(WindowOpError::PermissionDenied);
        }
        Ok(self.client_events[win_id].pop())
    }

    pub fn close_all_for_pid(&mut self, pid: usize) {
        for i in 0..MAX_WINDOWS {
            if let Some(win) = self.windows[i].as_ref() {
                if win.owner_pid == Some(pid) {
                    self.close_window(i);
                }
            }
        }
    }

    pub fn find_window_at(&self, px: usize, py: usize) -> Option<usize> {
        // Itera de cima para baixo no Z-order
        for &idx in self.order.iter().rev() {
            if let Some(win) = self.windows[idx] {
                if win.contains_point(px, py) {
                    return Some(idx);
                }
            }
        }
        None
    }

    pub fn render_snap_preview(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        snap_x: usize,
        snap_y: usize,
        snap_w: usize,
        snap_h: usize,
    ) {
        if snap_w == 0 || snap_h == 0 { return; }
        draw_rect_outline(buf, screen_w, screen_h, snap_x, snap_y, snap_w, snap_h, COLOR_CYAN_NEON);
        if snap_w > 4 && snap_h > 4 {
            draw_rect_outline(buf, screen_w, screen_h, snap_x + 1, snap_y + 1, snap_w - 2, snap_h - 2, COLOR_BORDER_ACT);
        }
    }

    pub fn render_window_chrome(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        win: &Window,
        is_active: bool,
        mode: CapabilityMode,
    ) {
        let title_color = if is_active { COLOR_TOPPANEL } else { 0x0011_1827 };
        let border_color = if is_active { COLOR_BORDER_ACT } else { COLOR_BORDER };

        // Sombras projetadas curvas conforme CapabilityMode
        if mode == CapabilityMode::Advanced {
            draw_drop_shadow_rounded(buf, screen_w, screen_h, win.x, win.y, win.width, win.height, 6, 4, 0x0005_0710);
            draw_drop_shadow_rounded(buf, screen_w, screen_h, win.x, win.y, win.width, win.height, 6, 2, 0x000A_0F1D);
        } else if mode == CapabilityMode::Balanced && is_active {
            draw_drop_shadow_rounded(buf, screen_w, screen_h, win.x, win.y, win.width, win.height, 6, 3, 0x000A_0F1D);
        }

        // Corpo da janela com cantos inferiores arredondados (R=6)
        fill_rounded_rect(
            buf,
            screen_w,
            screen_h,
            win.x,
            win.y,
            win.width,
            win.height,
            6,
            COLOR_CARD,
        );

        // Barra de título com cantos superiores arredondados (R=6)
        fill_rounded_top(buf, screen_w, screen_h, win.x, win.y, win.width, TITLE_BAR_HEIGHT, 6, title_color);

        // Borda externa arredondada de precisão
        draw_rounded_outline(
            buf,
            screen_w,
            screen_h,
            win.x,
            win.y,
            win.width,
            win.height,
            6,
            border_color,
        );

        // Bisel superior sutil (1px highlight para efeito acrílico moderno)
        if win.width > 12 {
            let bevel_color = if is_active { 0x0047_5569 } else { 0x0024_3042 };
            draw_line(
                buf,
                screen_w,
                screen_h,
                (win.x + 6) as isize,
                win.y as isize,
                (win.x + win.width.saturating_sub(6)) as isize,
                win.y as isize,
                bevel_color,
            );
        }

        // Acento luminescente sob o título na janela ativa (Garuda Neon Dr460nized)
        if is_active {
            draw_line(
                buf,
                screen_w,
                screen_h,
                win.x as isize,
                (win.y + TITLE_BAR_HEIGHT) as isize,
                (win.x + win.width) as isize,
                (win.y + TITLE_BAR_HEIGHT) as isize,
                COLOR_CYAN,
            );
            if mode == CapabilityMode::Advanced {
                draw_line(
                    buf,
                    screen_w,
                    screen_h,
                    win.x as isize,
                    (win.y + TITLE_BAR_HEIGHT + 1) as isize,
                    (win.x + win.width) as isize,
                    (win.y + TITLE_BAR_HEIGHT + 1) as isize,
                    COLOR_PURPLE,
                );
            }
        }

        // Título da janela
        let text_color = if is_active { COLOR_TEXT_PRI } else { COLOR_TEXT_MUTED };
        draw_string(buf, screen_w, screen_h, win.title_str(), win.x + 10, win.y + 4, text_color, Some(title_color));

        // Trio de botões semáforo arredondados (Traffic-Light / elementary OS)
        let btn_y = win.y + 4;

        if win.can_minimize {
            let btn_min_x = win.x + win.width.saturating_sub(64);
            fill_rounded_rect(buf, screen_w, screen_h, btn_min_x, btn_y, 16, 16, 4, COLOR_AMBER);
            draw_rounded_outline(buf, screen_w, screen_h, btn_min_x, btn_y, 16, 16, 4, 0x00D9_7706);
            draw_char(buf, screen_w, screen_h, b'-', btn_min_x + 4, btn_y, COLOR_WHITE, Some(COLOR_AMBER));
        }

        if win.can_maximize {
            let btn_max_x = win.x + win.width.saturating_sub(44);
            let max_char = if win.is_maximized || win.is_snapped_left || win.is_snapped_right { b'^' } else { b'+' };
            fill_rounded_rect(buf, screen_w, screen_h, btn_max_x, btn_y, 16, 16, 4, COLOR_EMERALD);
            draw_rounded_outline(buf, screen_w, screen_h, btn_max_x, btn_y, 16, 16, 4, 0x0016_A34A);
            draw_char(buf, screen_w, screen_h, max_char, btn_max_x + 4, btn_y, COLOR_WHITE, Some(COLOR_EMERALD));
        }

        if win.can_close {
            let btn_close_x = win.x + win.width.saturating_sub(24);
            fill_rounded_rect(buf, screen_w, screen_h, btn_close_x, btn_y, 16, 16, 4, COLOR_RUBY);
            draw_rounded_outline(buf, screen_w, screen_h, btn_close_x, btn_y, 16, 16, 4, 0x00DC_2626);
            draw_char(buf, screen_w, screen_h, b'X', btn_close_x + 4, btn_y, COLOR_WHITE, Some(COLOR_RUBY));
        }
    }
}
