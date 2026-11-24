//! Cairo-Dock Flutuante com Reflexos Verticais (Inspirado no visual XFCE / Cairo-Dock / Web Portal)
//! Superfície curva translúcida (Glassmorphism R=24) com ícones squircle vibrantes e reflexo espelhado.

use crate::compositor::*;
use crate::font::*;
use crate::wm::WindowManager;

pub const DOCK_WIDTH: usize = 580;
pub const DOCK_HEIGHT: usize = 62;
pub const DOCK_ICON_SIZE: usize = 40;
pub const DOCK_ICON_RADIUS: usize = 11;
pub const DOCK_REFLECTION_H: usize = 12;

pub struct DockSlotInfo {
    pub label: &'static str,
    pub icon_text: &'static str,
    pub tip: &'static str,
    pub bg_color: u32,
    pub fg_color: u32,
    pub border_color: u32,
}

pub const DOCK_SLOTS: [DockSlotInfo; 10] = [
    DockSlotInfo {
        label: "Menu",
        icon_text: "[:]",
        tip: "Menu Iniciar (Whisker)",
        bg_color: 0x0010_B981,
        fg_color: COLOR_WHITE,
        border_color: 0x0034_D399,
    },
    DockSlotInfo {
        label: "História",
        icon_text: "[=]",
        tip: "A História do Jinn OS",
        bg_color: 0x007C_3AED,
        fg_color: COLOR_WHITE,
        border_color: 0x00A7_8BFA,
    },
    DockSlotInfo {
        label: "Criador",
        icon_text: "(o)",
        tip: "O Criador (Geovane Estanislau)",
        bg_color: 0x0002_84C7,
        fg_color: COLOR_WHITE,
        border_color: 0x0038_BDF8,
    },
    DockSlotInfo {
        label: "Terminal",
        icon_text: ">_",
        tip: "Terminal Shell (Ring 3)",
        bg_color: 0x000B_1320,
        fg_color: COLOR_EMERALD,
        border_color: 0x0010_B981,
    },
    DockSlotInfo {
        label: "Arquivos",
        icon_text: "[D]",
        tip: "Arquivos VFS (FAT32/Ext2)",
        bg_color: 0x00EA_580C,
        fg_color: COLOR_WHITE,
        border_color: 0x00FB_923C,
    },
    DockSlotInfo {
        label: "Browser",
        icon_text: "(W)",
        tip: "Jinn Browser Engine (JBE)",
        bg_color: 0x000D_9488,
        fg_color: COLOR_WHITE,
        border_color: 0x002D_D4BF,
    },
    DockSlotInfo {
        label: "Monitor",
        icon_text: "^v^",
        tip: "Monitor de Recursos (JAHA)",
        bg_color: 0x0025_63EB,
        fg_color: COLOR_WHITE,
        border_color: 0x0060_A5FA,
    },
    DockSlotInfo {
        label: "Ajustes",
        icon_text: "[#]",
        tip: "Ajustes do Desktop",
        bg_color: 0x0093_33EA,
        fg_color: COLOR_WHITE,
        border_color: 0x00C0_84FC,
    },
    DockSlotInfo {
        label: "Discos",
        icon_text: "|v|",
        tip: "Gerenciador de Discos / Baixar ISO",
        bg_color: 0x00E1_1D48,
        fg_color: COLOR_WHITE,
        border_color: 0x00FB_7185,
    },
    DockSlotInfo {
        label: "Sobre",
        icon_text: "<G>",
        tip: "Sobre o Ecossistema / GitHub",
        bg_color: 0x001E_293B,
        fg_color: COLOR_WHITE,
        border_color: 0x0047_5569,
    },
];

pub fn get_slot_window_id(slot_idx: usize, browser_win_id: Option<usize>) -> usize {
    match slot_idx {
        0 => usize::MAX, // Menu Iniciar
        1 => 2,          // História / Sobre
        2 => 2,          // Criador / Sobre
        3 => 1,          // Terminal
        4 => 3,          // Arquivos
        5 => browser_win_id.unwrap_or(usize::MAX), // Browser
        6 => 0,          // Monitor
        7 => 4,          // Ajustes
        8 => 5,          // Discos
        9 => 2,          // Sobre
        _ => usize::MAX,
    }
}

pub fn render_dock(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mouse_x: usize,
    mouse_y: usize,
    wm: &WindowManager,
    browser_win_id: Option<usize>,
) {
    if screen_w < DOCK_WIDTH + 20 || screen_h < DOCK_HEIGHT + 20 { return; }

    let dock_x = (screen_w / 2).saturating_sub(DOCK_WIDTH / 2);
    let dock_y = screen_h.saturating_sub(DOCK_HEIGHT + 10);
    let top_idx = wm.top_window_id();

    // 1. Sombra suave de elevação (Drop Shadow)
    draw_drop_shadow_rounded(buf, screen_w, screen_h, dock_x, dock_y, DOCK_WIDTH, DOCK_HEIGHT, 22, 4, COLOR_SHADOW);

    // 2. Superfície flutuante translúcida do Dock (Frosted Glass R=22 suave com alpha 140)
    fill_acrylic_glass(buf, screen_w, screen_h, dock_x, dock_y, DOCK_WIDTH, DOCK_HEIGHT, 22, 0x000B_0F19, 140);
    draw_rounded_outline(buf, screen_w, screen_h, dock_x, dock_y, DOCK_WIDTH, DOCK_HEIGHT, 22, 0x002A_3E58);

    let slot_span = 53;
    let start_x = dock_x + 25;
    let mut hovered_tip: Option<(&str, usize)> = None;

    for (i, slot) in DOCK_SLOTS.iter().enumerate() {
        let sx = start_x + (i * slot_span);
        let win_id = get_slot_window_id(i, browser_win_id);

        let is_hover = mouse_x >= sx && mouse_x < sx + DOCK_ICON_SIZE
            && mouse_y >= dock_y && mouse_y < dock_y + DOCK_HEIGHT;

        // Efeito de elevação ao passar o mouse
        let icon_y = if is_hover { dock_y + 4 } else { dock_y + 8 };

        if is_hover {
            hovered_tip = Some((slot.tip, sx));
        }

        // 3. Sombra de elevação sob o squircle do ícone
        draw_drop_shadow_rounded(buf, screen_w, screen_h, sx, icon_y, DOCK_ICON_SIZE, DOCK_ICON_SIZE, DOCK_ICON_RADIUS, 2, COLOR_SHADOW);

        // 4. Squircle com cor vibrante e borda luminescente
        fill_rounded_rect(buf, screen_w, screen_h, sx, icon_y, DOCK_ICON_SIZE, DOCK_ICON_SIZE, DOCK_ICON_RADIUS, slot.bg_color);
        draw_rounded_outline(buf, screen_w, screen_h, sx, icon_y, DOCK_ICON_SIZE, DOCK_ICON_SIZE, DOCK_ICON_RADIUS, slot.border_color);

        if is_hover {
            draw_rounded_outline(buf, screen_w, screen_h, sx.saturating_sub(1), icon_y.saturating_sub(1), DOCK_ICON_SIZE + 2, DOCK_ICON_SIZE + 2, DOCK_ICON_RADIUS + 1, COLOR_WHITE);
        }

        // 5. Glifo/Ícone centralizado com tipografia antialiased
        let icon_w = text_width(slot.icon_text);
        let tx = sx + (DOCK_ICON_SIZE.saturating_sub(icon_w)) / 2;
        let ty = icon_y + (DOCK_ICON_SIZE.saturating_sub(FONT_HEIGHT)) / 2;
        draw_string_antialiased(buf, screen_w, screen_h, slot.icon_text, tx, ty, slot.fg_color, Some(slot.bg_color));

        // 6. Reflexo característico do Cairo-Dock (Vertical Inverted Glow Reflection)
        let refl_start_y = icon_y + DOCK_ICON_SIZE + 1;
        for dy in 0..DOCK_REFLECTION_H {
            let ry = refl_start_y + dy;
            if ry >= screen_h || ry >= dock_y + DOCK_HEIGHT - 3 { break; }

            // Gradiente suave de transparência: mais visível próximo à base, desvanecendo a zero
            let alpha = (((DOCK_REFLECTION_H - dy) * 75) / DOCK_REFLECTION_H) as u8;
            let refl_color = slot.bg_color;

            let row_start = ry * screen_w;
            // Reflexo com ligeiro estreitamento de perspectiva
            let cut = (dy * 2) / 3;
            let rx0 = (sx + cut).min(screen_w);
            let rx1 = (sx + DOCK_ICON_SIZE).saturating_sub(cut).min(screen_w);

            for px in rx0..rx1 {
                let cur = buf[row_start + px];
                buf[row_start + px] = alpha_blend(cur, refl_color, alpha);
            }
        }

        // 7. Luz Indicadora de Janela Ativa (Dot/Pill refinado na base do dock)
        if win_id != usize::MAX && win_id < wm.windows.len() {
            if let Some(w) = wm.windows[win_id] {
                let dot_y = dock_y + DOCK_HEIGHT.saturating_sub(6);
                let center_x = sx + (DOCK_ICON_SIZE / 2);
                if w.is_open && !w.is_minimized && top_idx == win_id {
                    // Janela em foco ativo: cápsula ciano luminescente com halo
                    let dot_x = center_x.saturating_sub(9);
                    fill_rounded_rect(buf, screen_w, screen_h, dot_x, dot_y, 18, 3, 1, COLOR_CYAN_NEON);
                    fill_rounded_rect(buf, screen_w, screen_h, dot_x + 2, dot_y + 1, 14, 1, 0, COLOR_WHITE);
                } else if w.is_open && !w.is_minimized {
                    // Janela em segundo plano: dot circular esmeralda de 4px
                    let dot_x = center_x.saturating_sub(3);
                    fill_rounded_rect(buf, screen_w, screen_h, dot_x, dot_y, 6, 3, 1, 0x0010_B981);
                } else if w.is_minimized {
                    // Janela minimizada: dot circular âmbar de 4px
                    let dot_x = center_x.saturating_sub(3);
                    fill_rounded_rect(buf, screen_w, screen_h, dot_x, dot_y, 6, 3, 1, 0x00F5_9E0B);
                }
            }
        }
    }

    // 8. Tooltip flutuante superior ao pairar sobre qualquer ícone
    if let Some((tip_text, sx)) = hovered_tip {
        let tip_w = text_width(tip_text) + 16;
        let tip_x = (sx + DOCK_ICON_SIZE / 2).saturating_sub(tip_w / 2).max(10);
        let tip_y = dock_y.saturating_sub(26);

        draw_drop_shadow_rounded(buf, screen_w, screen_h, tip_x, tip_y, tip_w, 22, 6, 4, COLOR_SHADOW);
        fill_rounded_rect(buf, screen_w, screen_h, tip_x, tip_y, tip_w, 22, 6, COLOR_SURFACE_L0);
        draw_rounded_outline(buf, screen_w, screen_h, tip_x, tip_y, tip_w, 22, 6, COLOR_CYAN_NEON);
        draw_string_antialiased(buf, screen_w, screen_h, tip_text, tip_x + 8, tip_y + 5, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L0));
    }
}
