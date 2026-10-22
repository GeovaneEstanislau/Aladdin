//! Gerenciador de Janelas, Z-Order e Decorações de Superfície

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::CapabilityMode;
use super::window::{Window, TITLE_BAR_HEIGHT};

pub const MAX_WINDOWS: usize = 4;

pub struct WindowManager {
    pub windows: [Option<Window>; MAX_WINDOWS],
    pub order: [usize; MAX_WINDOWS], // 0 = fundo, 3 = topo
}

impl WindowManager {
    pub const fn new() -> Self {
        Self {
            windows: [None; MAX_WINDOWS],
            order: [0, 1, 2, 3],
        }
    }

    pub fn top_window_id(&self) -> usize {
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

        // Sombras projetadas conforme CapabilityMode
        if mode == CapabilityMode::Advanced {
            fill_rect(buf, screen_w, screen_h, win.x + 4, win.y + 4, win.width, win.height, 0x0005_0710);
            fill_rect(buf, screen_w, screen_h, win.x + 2, win.y + 2, win.width, win.height, 0x000A_0F1D);
        } else if mode == CapabilityMode::Balanced && is_active {
            fill_rect(buf, screen_w, screen_h, win.x + 3, win.y + 3, win.width, win.height, 0x000A_0F1D);
        }

        // Borda externa
        draw_rect_outline(
            buf,
            screen_w,
            screen_h,
            win.x.saturating_sub(1),
            win.y.saturating_sub(1),
            win.width + 2,
            win.height + 2,
            border_color,
        );

        // Barra de título
        fill_rect(buf, screen_w, screen_h, win.x, win.y, win.width, TITLE_BAR_HEIGHT, title_color);

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

        // Título da janela e indicador de foco
        if is_active {
            draw_char(buf, screen_w, screen_h, b'*', win.x + 8, win.y + 4, COLOR_CYAN, Some(title_color));
            draw_string(buf, screen_w, screen_h, win.title, win.x + 20, win.y + 4, COLOR_TEXT_PRI, Some(title_color));
        } else {
            draw_string(buf, screen_w, screen_h, win.title, win.x + 10, win.y + 4, COLOR_TEXT_MUTED, Some(title_color));
        }

        // Trio de botões semáforo (Traffic-Light / elementary OS)
        let btn_y = win.y + 4;

        // Minimizar [-] em âmbar
        let btn_min_x = win.x + win.width.saturating_sub(64);
        fill_rect(buf, screen_w, screen_h, btn_min_x, btn_y, 16, 16, COLOR_AMBER);
        draw_char(buf, screen_w, screen_h, b'-', btn_min_x + 4, btn_y, COLOR_WHITE, Some(COLOR_AMBER));

        // Maximizar [+] / [^] em esmeralda
        let btn_max_x = win.x + win.width.saturating_sub(44);
        let max_char = if win.is_maximized { b'^' } else { b'+' };
        fill_rect(buf, screen_w, screen_h, btn_max_x, btn_y, 16, 16, COLOR_EMERALD);
        draw_char(buf, screen_w, screen_h, max_char, btn_max_x + 4, btn_y, COLOR_WHITE, Some(COLOR_EMERALD));

        // Fechar [X] em rubi
        let btn_close_x = win.x + win.width.saturating_sub(24);
        fill_rect(buf, screen_w, screen_h, btn_close_x, btn_y, 16, 16, COLOR_RUBY);
        draw_char(buf, screen_w, screen_h, b'X', btn_close_x + 4, btn_y, COLOR_WHITE, Some(COLOR_RUBY));

        // Corpo da janela
        fill_rect(
            buf,
            screen_w,
            screen_h,
            win.x,
            win.y + TITLE_BAR_HEIGHT + 1,
            win.width,
            win.height.saturating_sub(TITLE_BAR_HEIGHT + 1),
            COLOR_CARD,
        );
    }
}
