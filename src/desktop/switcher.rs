//! Alternador de Tarefas e Multitarefa Rápida (App Switcher HUD)
//!
//! Inspirado no visual fluido do Gala (elementary OS 7) e Mission Control,
//! com a estética de alto contraste e acentos neon do Garuda Linux Dr460nized.
//! Projetado para navegação fluída via Tab e clique direto do mouse sem alocações.

use crate::compositor::*;
use crate::font::*;
use crate::wm::WindowManager;

pub const SWITCHER_WIDTH: usize = 480;
pub const SWITCHER_MAX_HEIGHT: usize = 250;
pub const ITEM_HEIGHT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitcherAction {
    None,
    FocusWindow(usize),
    Close,
}

pub struct AppSwitcherHUD;

impl AppSwitcherHUD {
    /// Renderiza o overlay central translúcido do Alternador de Tarefas
    pub fn render(
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        mouse_x: usize,
        mouse_y: usize,
        wm: &WindowManager,
        selected_slot: usize,
    ) {
        // Coleta janelas abertas
        let mut open_windows = [0usize; crate::wm::MAX_WINDOWS];
        let mut open_count = 0;

        for i in (0..crate::wm::MAX_WINDOWS).rev() {
            let wid = wm.order[i];
            if let Some(w) = &wm.windows[wid] {
                if w.is_open {
                    open_windows[open_count] = wid;
                    open_count += 1;
                }
            }
        }

        if open_count == 0 {
            return;
        }

        let hud_h = (44 + open_count * (ITEM_HEIGHT + 4) + 12).min(SWITCHER_MAX_HEIGHT);
        let hud_w = SWITCHER_WIDTH.min(screen_w.saturating_sub(40));
        let hud_x = (screen_w / 2).saturating_sub(hud_w / 2);
        let hud_y = (screen_h / 2).saturating_sub(hud_h / 2);

        // 1. Sombra suave profunda (Elevação nível 3)
        draw_drop_shadow_rounded(buf, screen_w, screen_h, hud_x, hud_y, hud_w, hud_h, 10, 6, COLOR_SHADOW);

        // 2. Fundo acrílico translúcido com cantos arredondados (R=10)
        fill_rounded_rect(buf, screen_w, screen_h, hud_x, hud_y, hud_w, hud_h, 10, COLOR_SURFACE_L1);
        draw_rounded_outline(buf, screen_w, screen_h, hud_x, hud_y, hud_w, hud_h, 10, COLOR_CYAN_NEON);

        // 3. Bisel superior sutil (1px highlight acrílico)
        if hud_w > 16 {
            draw_line(
                buf,
                screen_w,
                screen_h,
                (hud_x + 8) as isize,
                hud_y as isize,
                (hud_x + hud_w - 8) as isize,
                hud_y as isize,
                0x0047_5569,
            );
        }

        // 4. Cabeçalho elegante
        fill_rounded_top(buf, screen_w, screen_h, hud_x + 1, hud_y + 1, hud_w - 2, 28, 9, COLOR_SURFACE_L0);
        draw_char(buf, screen_w, screen_h, b'*', hud_x + 12, hud_y + 8, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L0));
        draw_string(
            buf,
            screen_w,
            screen_h,
            "Alternador de Tarefas  [Tab: Proximo | Enter: Foco]",
            hud_x + 26,
            hud_y + 8,
            COLOR_TEXT_PRI,
            Some(COLOR_SURFACE_L0),
        );
        draw_line(
            buf,
            screen_w,
            screen_h,
            hud_x as isize,
            (hud_y + 29) as isize,
            (hud_x + hud_w) as isize,
            (hud_y + 29) as isize,
            COLOR_BORDER_SUBTLE,
        );

        // 5. Linhas de tarefas abertas
        let top_id = wm.top_window_id();
        let mut item_y = hud_y + 36;

        for (idx, &wid) in open_windows[..open_count].iter().enumerate() {
            if let Some(win) = &wm.windows[wid] {
                let is_selected = idx == (selected_slot % open_count);
                let is_hover = mouse_x >= hud_x + 10 && mouse_x < hud_x + hud_w - 10
                    && mouse_y >= item_y && mouse_y < item_y + ITEM_HEIGHT;

                let row_bg = if is_selected {
                    COLOR_SURFACE_HOVER
                } else if is_hover {
                    COLOR_SURFACE_ACTIVE
                } else {
                    COLOR_SURFACE_L2
                };

                let row_bd = if is_selected {
                    COLOR_CYAN_NEON
                } else if is_hover {
                    COLOR_BORDER_ACT
                } else {
                    COLOR_BORDER_SUBTLE
                };

                let item_w = hud_w - 20;
                fill_rounded_rect(buf, screen_w, screen_h, hud_x + 10, item_y, item_w, ITEM_HEIGHT, 6, row_bg);
                draw_rounded_outline(buf, screen_w, screen_h, hud_x + 10, item_y, item_w, ITEM_HEIGHT, 6, row_bd);

                // Indicador de seleção
                if is_selected {
                    draw_char(buf, screen_w, screen_h, b'>', hud_x + 16, item_y + 10, COLOR_CYAN_NEON, Some(row_bg));
                }

                draw_string(buf, screen_w, screen_h, win.title_str(), hud_x + 28, item_y + 10, COLOR_TEXT_PRI, Some(row_bg));

                // Tag de estado da janela (Foco / Ativo / Minimizada)
                let (state_str, state_color) = if win.is_minimized {
                    ("Minimizada", COLOR_AMBER)
                } else if wid == top_id {
                    ("Foco Ativo", COLOR_EMERALD)
                } else {
                    ("Segundo Plano", COLOR_TEXT_MUTED)
                };

                let tag_x = hud_x + item_w - 100;
                draw_string(buf, screen_w, screen_h, state_str, tag_x, item_y + 10, state_color, Some(row_bg));

                item_y += ITEM_HEIGHT + 4;
            }
        }
    }

    /// Processa clique com o mouse no Alternador de Tarefas
    pub fn handle_click(
        screen_w: usize,
        screen_h: usize,
        mouse_x: usize,
        mouse_y: usize,
        wm: &WindowManager,
    ) -> SwitcherAction {
        let mut open_windows = [0usize; crate::wm::MAX_WINDOWS];
        let mut open_count = 0;

        for i in (0..crate::wm::MAX_WINDOWS).rev() {
            let wid = wm.order[i];
            if let Some(w) = &wm.windows[wid] {
                if w.is_open {
                    open_windows[open_count] = wid;
                    open_count += 1;
                }
            }
        }

        if open_count == 0 {
            return SwitcherAction::Close;
        }

        let hud_h = (44 + open_count * (ITEM_HEIGHT + 4) + 12).min(SWITCHER_MAX_HEIGHT);
        let hud_w = SWITCHER_WIDTH.min(screen_w.saturating_sub(40));
        let hud_x = (screen_w / 2).saturating_sub(hud_w / 2);
        let hud_y = (screen_h / 2).saturating_sub(hud_h / 2);

        // Clique fora fecha o switcher
        if mouse_x < hud_x || mouse_x >= hud_x + hud_w || mouse_y < hud_y || mouse_y >= hud_y + hud_h {
            return SwitcherAction::Close;
        }

        // Verifica qual item foi clicado
        let mut item_y = hud_y + 36;
        for &wid in open_windows[..open_count].iter() {
            if mouse_x >= hud_x + 10 && mouse_x < hud_x + hud_w - 10
                && mouse_y >= item_y && mouse_y < item_y + ITEM_HEIGHT {
                return SwitcherAction::FocusWindow(wid);
            }
            item_y += ITEM_HEIGHT + 4;
        }

        SwitcherAction::None
    }
}

