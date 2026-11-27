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
        if let Some(ref mut buf) = self.client_buffers[id] {
            buf.clear();
            buf.resize(client_pixels, 0x000F_172A);
        } else {
            self.client_buffers[id] = Some(alloc::vec![0x000F_172A; client_pixels]);
        }
        self.client_events[id].clear();
        Ok(id)
    }

    pub const MAX_CLIENT_BUFFER_PIXELS: usize = 1920 * 1080;

    /// Sincroniza dimensões de janela cliente (incl. entrada/saída de tela cheia 1024x768)
    pub fn sync_client_window_size(
        &mut self,
        owner_pid: usize,
        win_id: usize,
        new_w: usize,
        new_h: usize,
        screen_w: usize,
        screen_h: usize,
    ) -> Result<bool, WindowOpError> {
        if win_id >= MAX_WINDOWS { return Err(WindowOpError::NotFound); }
        if new_w == 0 || new_h == 0 || new_w > 1920 || new_h > 1080 {
            return Err(WindowOpError::InvalidDimensions);
        }
        let new_pixels = new_w.checked_mul(new_h).ok_or(WindowOpError::InvalidDimensions)?;
        if new_pixels > Self::MAX_CLIENT_BUFFER_PIXELS {
            return Err(WindowOpError::InvalidDimensions);
        }

        let entering_fs = new_w == screen_w && new_h == screen_h;
        {
            let win = self.windows[win_id].as_mut().ok_or(WindowOpError::NotFound)?;
            if win.owner_pid != Some(owner_pid) {
                return Err(WindowOpError::PermissionDenied);
            }
            if win.client_width() == new_w && win.client_height() == new_h {
                return Ok(false);
            }
            if entering_fs {
                if !win.is_fullscreen {
                    win.saved_x = win.x;
                    win.saved_y = win.y;
                    win.saved_width = win.width;
                    win.saved_height = win.height;
                }
                win.is_fullscreen = true;
                win.is_maximized = false;
                win.is_snapped_left = false;
                win.is_snapped_right = false;
                win.x = 0;
                win.y = 0;
                win.width = new_w;
                win.height = new_h;
                win.anim_ticks_remaining = 0;
            } else {
                if win.is_fullscreen {
                    win.is_fullscreen = false;
                    win.x = win.saved_x;
                    win.y = win.saved_y;
                }
                win.width = new_w.saturating_add(2);
                win.height = new_h.saturating_add(TITLE_BAR_HEIGHT + 2);
                if !win.is_maximized {
                    let safe_dock_top = screen_h.saturating_sub(crate::desktop::dock::DOCK_HEIGHT + 12);
                    let max_y = safe_dock_top
                        .saturating_sub(win.height)
                        .max(crate::desktop::wingpanel::WINGPANEL_HEIGHT + 2);
                    if win.y > max_y {
                        win.y = max_y;
                    }
                }
                win.anim_ticks_remaining = 0;
            }
        }

        // Libera o buffer antigo antes de reservar o novo tamanho para evitar pico duplo no heap estático
        self.client_buffers[win_id] = None;
        let mut buf = alloc::vec::Vec::new();
        if buf.try_reserve_exact(new_pixels).is_err() {
            return Err(WindowOpError::InvalidDimensions);
        }
        buf.resize(new_pixels, 0x0018_1825);
        self.client_buffers[win_id] = Some(buf);
        if entering_fs {
            self.bring_to_front(win_id);
        }
        Ok(true)
    }

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

    pub fn tick_animations(&mut self) -> bool {
        let mut any_animated = false;
        for opt_win in &mut self.windows {
            if let Some(win) = opt_win.as_mut() {
                if win.tick_animation() {
                    any_animated = true;
                }
            }
        }
        any_animated
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
        let rx = win.render_x();
        let ry = win.render_y();
        let rw = win.render_width();
        let rh = win.render_height();

        let win_r = 14;
        let title_color = if is_active { 0x0016_2032 } else { 0x000F_172A };

        // 1. Sombras projetadas suaves multicamada de 24px (Drop Shadows de elevação realista)
        if is_active {
            draw_drop_shadow_rounded(buf, screen_w, screen_h, rx, ry, rw, rh, win_r, 24, 0x0003_050A);
            draw_drop_shadow_rounded(buf, screen_w, screen_h, rx, ry, rw, rh, win_r, 14, 0x0006_0912);
            draw_drop_shadow_rounded(buf, screen_w, screen_h, rx, ry, rw, rh, win_r, 6, 0x000B_1020);
            draw_drop_shadow_rounded(buf, screen_w, screen_h, rx, ry, rw, rh, win_r, 2, 0x0012_1A30);
        } else if mode != CapabilityMode::Basic {
            draw_drop_shadow_rounded(buf, screen_w, screen_h, rx, ry, rw, rh, win_r, 10, 0x0004_060C);
            draw_drop_shadow_rounded(buf, screen_w, screen_h, rx, ry, rw, rh, win_r, 4, 0x0008_0D18);
        }

        // 2. Fundo Multicamadas com Desfoque Acrílico Real e Gradiente Vertical:
        //    rgba(18, 24, 38, 0.68) [alpha 173] a rgba(12, 16, 26, 0.78) [alpha 199]
        if mode != CapabilityMode::Basic {
            let (top_a, bot_a) = if is_active { (173, 199) } else { (155, 185) };
            fill_acrylic_glass_gradient(
                buf,
                screen_w,
                screen_h,
                rx,
                ry,
                rw,
                rh,
                win_r,
                0x0012_1826,
                top_a,
                0x000C_101A,
                bot_a,
            );
        } else {
            fill_rounded_rect(buf, screen_w, screen_h, rx, ry, rw, rh, win_r, 0x000F_172A);
        }

        // 3. Barra de título contemporânea (R=14 no topo) com translucidez acrílica
        let title_alpha = if is_active { 165 } else { 145 };
        fill_rounded_top_alpha(buf, screen_w, screen_h, rx, ry, rw, TITLE_BAR_HEIGHT, win_r, title_color, title_alpha);

        // 4. Camada de brilho especular no topo da janela (rgba(255, 255, 255, 0.22))
        if rw > 30 {
            let spec_color = 0x00FF_FFFF;
            draw_line_alpha(buf, screen_w, screen_h, (rx + win_r) as isize, (ry + 1) as isize, (rx + rw.saturating_sub(win_r)) as isize, (ry + 1) as isize, spec_color, 56);
            draw_line_alpha(buf, screen_w, screen_h, (rx + win_r + 2) as isize, (ry + 2) as isize, (rx + rw.saturating_sub(win_r + 2)) as isize, (ry + 2) as isize, spec_color, 28);
        }

        // 5. Borda interna luminosa sutil de 1px (rgba(255, 255, 255, 0.22) -> alpha 56)
        if rw > 30 && rh > 30 {
            draw_rounded_outline_alpha(buf, screen_w, screen_h, rx + 1, ry + 1, rw.saturating_sub(2), rh.saturating_sub(2), win_r - 1, 0x00FF_FFFF, 56);
        }

        // 6. Borda externa translúcida de 1px com cantos suaves
        let out_border_alpha = if is_active { 75 } else { 45 };
        draw_rounded_outline_alpha(
            buf,
            screen_w,
            screen_h,
            rx,
            ry,
            rw,
            rh,
            win_r,
            0x00FF_FFFF,
            out_border_alpha,
        );

        // 7. Divisor sob a barra de título
        let div_color = if is_active { COLOR_CYAN } else { 0x001E_293B };
        draw_line(
            buf,
            screen_w,
            screen_h,
            rx as isize,
            (ry + TITLE_BAR_HEIGHT) as isize,
            (rx + rw) as isize,
            (ry + TITLE_BAR_HEIGHT) as isize,
            div_color,
        );
        if is_active && mode == CapabilityMode::Advanced {
            draw_line(
                buf,
                screen_w,
                screen_h,
                rx as isize,
                (ry + TITLE_BAR_HEIGHT + 1) as isize,
                (rx + rw) as isize,
                (ry + TITLE_BAR_HEIGHT + 1) as isize,
                COLOR_PURPLE,
            );
        }

        // 8. Título da janela com hierarquia tipográfica nítida (16px Header semi-bold)
        let text_color = if is_active { COLOR_WHITE } else { COLOR_TEXT_MUTED };
        draw_string_antialiased(buf, screen_w, screen_h, win.title_str(), rx + 16, ry + 10, text_color, Some(title_color));

        // 9. Controles modernos de janela estilo semáforo (Traffic-Light circular com brilho)
        let btn_y = ry + 8;

        if win.can_minimize {
            let btn_min_x = rx + rw.saturating_sub(66);
            fill_rounded_rect(buf, screen_w, screen_h, btn_min_x, btn_y, 16, 16, 8, 0x00F5_9E0B);
            draw_rounded_outline(buf, screen_w, screen_h, btn_min_x, btn_y, 16, 16, 8, 0x00D9_7706);
            draw_char_antialiased(buf, screen_w, screen_h, '-', btn_min_x + 4, btn_y + 1, COLOR_WHITE, Some(0x00F5_9E0B), None);
        }

        if win.can_maximize {
            let btn_max_x = rx + rw.saturating_sub(44);
            let max_char = if win.is_maximized || win.is_snapped_left || win.is_snapped_right { '^' } else { '+' };
            fill_rounded_rect(buf, screen_w, screen_h, btn_max_x, btn_y, 16, 16, 8, 0x0010_B981);
            draw_rounded_outline(buf, screen_w, screen_h, btn_max_x, btn_y, 16, 16, 8, 0x0005_9669);
            draw_char_antialiased(buf, screen_w, screen_h, max_char, btn_max_x + 4, btn_y + 1, COLOR_WHITE, Some(0x0010_B981), None);
        }

        if win.can_close {
            let btn_close_x = rx + rw.saturating_sub(22);
            fill_rounded_rect(buf, screen_w, screen_h, btn_close_x, btn_y, 16, 16, 8, 0x00EF_4444);
            draw_rounded_outline(buf, screen_w, screen_h, btn_close_x, btn_y, 16, 16, 8, 0x00DC_2626);
            draw_char_antialiased(buf, screen_w, screen_h, 'x', btn_close_x + 4, btn_y + 1, COLOR_WHITE, Some(0x00EF_4444), None);
        }
    }
}
