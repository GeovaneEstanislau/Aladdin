//! Plank Dock (Barra de Lançamento Inferior Flutuante)
//! Inspirada no visual elegante e translúcido do elementary OS 7 com acentos luminosos

use crate::compositor::*;
use crate::font::*;
use crate::wm::WindowManager;

pub const DOCK_WIDTH: usize = 672;
pub const DOCK_HEIGHT: usize = 44;

pub fn render_dock(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mouse_x: usize,
    mouse_y: usize,
    wm: &WindowManager,
    browser_win_id: Option<usize>,
) {
    let dock_x = (screen_w / 2).saturating_sub(DOCK_WIDTH / 2);
    let dock_y = screen_h.saturating_sub(DOCK_HEIGHT + 10);
    let top_idx = wm.top_window_id();

    // 1. Sombra suave de elevação (Drop Shadow)
    draw_drop_shadow(buf, screen_w, screen_h, dock_x, dock_y, DOCK_WIDTH, DOCK_HEIGHT, 3, COLOR_SHADOW);

    // 2. Superfície flutuante translúcida do Dock
    fill_rect(buf, screen_w, screen_h, dock_x, dock_y, DOCK_WIDTH, DOCK_HEIGHT, COLOR_DOCK);
    draw_rect_outline(buf, screen_w, screen_h, dock_x, dock_y, DOCK_WIDTH, DOCK_HEIGHT, COLOR_BORDER_ACT);

    let slots = [
        ("[#] Monitor", dock_x + 6, 0),
        ("[>] Console", dock_x + 100, 1),
        ("[~] Arquivos", dock_x + 194, 3),
        ("[D] Discos", dock_x + 288, 5),
        ("[@] Browser", dock_x + 382, browser_win_id.unwrap_or(usize::MAX)),
        ("[%] Ajustes", dock_x + 476, 4),
        ("[?] Sobre", dock_x + 570, 2),
    ];

    for &(label, slot_x, win_id) in slots.iter() {
        let is_hover = mouse_x >= slot_x && mouse_x < slot_x + 90 && mouse_y >= dock_y && mouse_y < dock_y + DOCK_HEIGHT;
        let slot_bg = if is_hover { COLOR_SURFACE_HOVER } else { COLOR_DOCK };
        let slot_bd = if is_hover { COLOR_CYAN_NEON } else { COLOR_BORDER_SUBTLE };

        fill_rect(buf, screen_w, screen_h, slot_x, dock_y + 4, 88, 34, slot_bg);
        if is_hover {
            draw_rect_outline(buf, screen_w, screen_h, slot_x, dock_y + 4, 88, 34, slot_bd);
        }

        let text_color = if is_hover { COLOR_WHITE } else { COLOR_TEXT_PRI };
        draw_string(buf, screen_w, screen_h, label, slot_x + 6, dock_y + 12, text_color, Some(slot_bg));

        // Indicador de app ativa (elementary OS dot / line indicator)
        if win_id != usize::MAX {
            if let Some(w) = wm.windows[win_id] {
                if w.is_open && !w.is_minimized && top_idx == win_id {
                    // Janela com foco ativo: traço ciano luminescente
                    fill_rect(buf, screen_w, screen_h, slot_x + 20, dock_y + 36, 48, 2, COLOR_CYAN_NEON);
                } else if w.is_open && !w.is_minimized {
                    // Aberta em segundo plano: ponto indicador esmeralda
                    fill_rect(buf, screen_w, screen_h, slot_x + 38, dock_y + 36, 12, 2, COLOR_EMERALD);
                } else if w.is_minimized {
                    // Minimizada: ponto indicador âmbar
                    fill_rect(buf, screen_w, screen_h, slot_x + 38, dock_y + 36, 12, 2, COLOR_AMBER);
                }
            }
        }
    }
}
