//! Sistema de Notificações Toast do Aladdin Desktop
//!
//! Exibe alertas visuais não bloqueantes com estilo translúcido e acentos luminosos.
//! Projetado para zero alocações dinâmicas no ciclo de renderização.

use crate::compositor::*;
use crate::font::*;

pub const MAX_NOTIFICATIONS: usize = 3;
pub const TOAST_WIDTH: usize = 310;
pub const TOAST_HEIGHT: usize = 44;
pub const TOAST_MARGIN_TOP: usize = 32;
pub const TOAST_MARGIN_RIGHT: usize = 12;

#[derive(Clone, Copy)]
pub struct ToastNotification {
    pub id: usize,
    pub title: [u8; 32],
    pub title_len: usize,
    pub message: [u8; 64],
    pub message_len: usize,
    pub icon_glyph: u8,
    pub accent_color: u32,
    pub created_tick: u64,
    pub duration_ticks: u64, // Padrão: 300 ticks = 3 segundos a 100 Hz
    pub is_active: bool,
}

impl ToastNotification {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            title: [0; 32],
            title_len: 0,
            message: [0; 64],
            message_len: 0,
            icon_glyph: b'i',
            accent_color: COLOR_CYAN_NEON,
            created_tick: 0,
            duration_ticks: 300,
            is_active: false,
        }
    }

    pub fn title_str(&self) -> &str {
        core::str::from_utf8(&self.title[..self.title_len]).unwrap_or("")
    }

    pub fn message_str(&self) -> &str {
        core::str::from_utf8(&self.message[..self.message_len]).unwrap_or("")
    }

    pub fn is_expired(&self, current_tick: u64) -> bool {
        if !self.is_active {
            true
        } else {
            current_tick.saturating_sub(self.created_tick) >= self.duration_ticks
        }
    }
}

pub struct NotificationCenter {
    pub items: [ToastNotification; MAX_NOTIFICATIONS],
    pub next_id: usize,
}

impl NotificationCenter {
    pub const fn new() -> Self {
        Self {
            items: [const { ToastNotification::empty() }; MAX_NOTIFICATIONS],
            next_id: 1,
        }
    }

    pub fn post(
        &mut self,
        icon: u8,
        title: &str,
        message: &str,
        color: u32,
        duration_ticks: u64,
        current_tick: u64,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        // Procura primeiro slot livre ou mais antigo
        let mut target_slot = 0;
        let mut found_free = false;
        let mut oldest_tick = u64::MAX;

        for (i, item) in self.items.iter().enumerate() {
            if !item.is_active || item.is_expired(current_tick) {
                target_slot = i;
                found_free = true;
                break;
            }
            if item.created_tick < oldest_tick {
                oldest_tick = item.created_tick;
                target_slot = i;
            }
        }

        if !found_free {
            // Reutiliza o mais antigo
        }

        let mut t_buf = [0u8; 32];
        let t_bytes = title.as_bytes();
        let t_len = t_bytes.len().min(32);
        for j in 0..t_len { t_buf[j] = t_bytes[j]; }

        let mut m_buf = [0u8; 64];
        let m_bytes = message.as_bytes();
        let m_len = m_bytes.len().min(64);
        for j in 0..m_len { m_buf[j] = m_bytes[j]; }

        self.items[target_slot] = ToastNotification {
            id,
            title: t_buf,
            title_len: t_len,
            message: m_buf,
            message_len: m_len,
            icon_glyph: icon,
            accent_color: color,
            created_tick: current_tick,
            duration_ticks: if duration_ticks == 0 { 300 } else { duration_ticks },
            is_active: true,
        };

        id
    }

    pub fn dismiss(&mut self, id: usize) {
        for item in self.items.iter_mut() {
            if item.is_active && item.id == id {
                item.is_active = false;
            }
        }
    }

    pub fn active_count(&self, current_tick: u64) -> usize {
        self.items.iter().filter(|it| it.is_active && !it.is_expired(current_tick)).count()
    }

    pub fn render(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        current_tick: u64,
        mouse_x: usize,
        mouse_y: usize,
    ) {
        let toast_x = screen_w.saturating_sub(TOAST_WIDTH + TOAST_MARGIN_RIGHT);
        let mut current_y = TOAST_MARGIN_TOP;

        for item in self.items.iter() {
            if !item.is_active || item.is_expired(current_tick) {
                continue;
            }

            let is_hover = mouse_x >= toast_x && mouse_x < toast_x + TOAST_WIDTH
                && mouse_y >= current_y && mouse_y < current_y + TOAST_HEIGHT;

            // 1. Sombra suave de elevação
            draw_drop_shadow(buf, screen_w, screen_h, toast_x, current_y, TOAST_WIDTH, TOAST_HEIGHT, 3, COLOR_SHADOW);

            // 2. Fundo acrílico escuro
            let bg_color = if is_hover { COLOR_SURFACE_HOVER } else { COLOR_SURFACE_L1 };
            fill_rect(buf, screen_w, screen_h, toast_x, current_y, TOAST_WIDTH, TOAST_HEIGHT, bg_color);
            draw_rect_outline(buf, screen_w, screen_h, toast_x, current_y, TOAST_WIDTH, TOAST_HEIGHT, COLOR_BORDER);

            // 3. Faixa de destaque lateral com cor de acento
            fill_rect(buf, screen_w, screen_h, toast_x, current_y, 4, TOAST_HEIGHT, item.accent_color);

            // 4. Ícone com moldura circular/quadrada
            fill_rect(buf, screen_w, screen_h, toast_x + 10, current_y + 10, 24, 24, COLOR_SURFACE_L2);
            draw_rect_outline(buf, screen_w, screen_h, toast_x + 10, current_y + 10, 24, 24, item.accent_color);
            draw_char(buf, screen_w, screen_h, item.icon_glyph, toast_x + 18, current_y + 14, item.accent_color, Some(COLOR_SURFACE_L2));

            // 5. Título e Mensagem
            draw_string(buf, screen_w, screen_h, item.title_str(), toast_x + 42, current_y + 8, COLOR_TEXT_PRI, Some(bg_color));
            draw_string(buf, screen_w, screen_h, item.message_str(), toast_x + 42, current_y + 24, COLOR_TEXT_SEC, Some(bg_color));

            // 6. Botão fechar [x] sutil à direita
            let close_x = toast_x + TOAST_WIDTH - 20;
            let close_color = if is_hover { COLOR_WHITE } else { COLOR_TEXT_MUTED };
            draw_char(buf, screen_w, screen_h, b'x', close_x, current_y + 8, close_color, Some(bg_color));

            // 7. Barra de progresso de vida restante (fluid decay bar)
            let elapsed = current_tick.saturating_sub(item.created_tick);
            if elapsed < item.duration_ticks && item.duration_ticks > 0 {
                let remaining = item.duration_ticks - elapsed;
                let bar_w = ((TOAST_WIDTH - 8) as u64 * remaining / item.duration_ticks) as usize;
                if bar_w > 0 {
                    fill_rect(buf, screen_w, screen_h, toast_x + 4, current_y + TOAST_HEIGHT - 2, bar_w, 2, item.accent_color);
                }
            }

            current_y += TOAST_HEIGHT + 8;
            if current_y + TOAST_HEIGHT >= screen_h {
                break;
            }
        }
    }

    pub fn handle_click(&mut self, mx: usize, my: usize, screen_w: usize, current_tick: u64) -> bool {
        let toast_x = screen_w.saturating_sub(TOAST_WIDTH + TOAST_MARGIN_RIGHT);
        let mut current_y = TOAST_MARGIN_TOP;

        for item in self.items.iter_mut() {
            if !item.is_active || item.is_expired(current_tick) {
                continue;
            }

            if mx >= toast_x && mx < toast_x + TOAST_WIDTH && my >= current_y && my < current_y + TOAST_HEIGHT {
                item.is_active = false;
                return true;
            }

            current_y += TOAST_HEIGHT + 8;
        }

        false
    }
}
