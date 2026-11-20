//! Atalhos na Área de Trabalho (Desktop Shortcuts)
//! Coluna lateral direita inspirada no portal Jinn OS com ícones squircle vibrantes

use crate::compositor::*;
use crate::font::*;

pub const SHORTCUT_BOX_SIZE: usize = 44;
pub const SHORTCUT_RADIUS: usize = 12;
pub const SHORTCUT_SPACING: usize = 70;

pub struct DesktopShortcutItem {
    pub label: &'static str,
    pub icon_text: &'static str,
    pub bg_color: u32,
    pub fg_color: u32,
    pub border_color: u32,
}

pub const DESKTOP_SHORTCUTS: [DesktopShortcutItem; 6] = [
    DesktopShortcutItem {
        label: "Historia",
        icon_text: "[=]",
        bg_color: 0x007C_3AED,
        fg_color: COLOR_WHITE,
        border_color: 0x00A7_8BFA,
    },
    DesktopShortcutItem {
        label: "O Criador",
        icon_text: "(o)",
        bg_color: 0x0002_84C7,
        fg_color: COLOR_WHITE,
        border_color: 0x0038_BDF8,
    },
    DesktopShortcutItem {
        label: "Terminal",
        icon_text: ">_",
        bg_color: 0x000B_1320,
        fg_color: COLOR_EMERALD,
        border_color: 0x0010_B981,
    },
    DesktopShortcutItem {
        label: "Codigo VFS",
        icon_text: "[D]",
        bg_color: 0x00EA_580C,
        fg_color: COLOR_WHITE,
        border_color: 0x00FB_923C,
    },
    DesktopShortcutItem {
        label: "Browser",
        icon_text: "(W)",
        bg_color: 0x000D_9488,
        fg_color: COLOR_WHITE,
        border_color: 0x002D_D4BF,
    },
    DesktopShortcutItem {
        label: "Baixar ISO",
        icon_text: "|v|",
        bg_color: 0x00E1_1D48,
        fg_color: COLOR_WHITE,
        border_color: 0x00FB_7185,
    },
];

pub fn get_shortcut_pos(screen_w: usize, index: usize) -> (usize, usize) {
    let x = screen_w.saturating_sub(76);
    let y = 46 + (index * SHORTCUT_SPACING);
    (x, y)
}

pub fn render_desktop_shortcuts(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mouse_x: usize,
    mouse_y: usize,
) {
    if screen_w < 500 { return; }

    for (i, item) in DESKTOP_SHORTCUTS.iter().enumerate() {
        let (x, y) = get_shortcut_pos(screen_w, i);
        if y + SHORTCUT_BOX_SIZE + 20 >= screen_h { break; }

        let is_hover = mouse_x >= x && mouse_x < x + SHORTCUT_BOX_SIZE + 10
            && mouse_y >= y && mouse_y < y + SHORTCUT_BOX_SIZE + 20;

        let icon_y = if is_hover { y.saturating_sub(2) } else { y };

        // 1. Sombra de elevação suave sob o squircle
        draw_drop_shadow_rounded(buf, screen_w, screen_h, x, icon_y, SHORTCUT_BOX_SIZE, SHORTCUT_BOX_SIZE, SHORTCUT_RADIUS, 3, COLOR_SHADOW);

        // 2. Fundo colorido em squircle (cantos arredondados R=12)
        fill_rounded_rect(buf, screen_w, screen_h, x, icon_y, SHORTCUT_BOX_SIZE, SHORTCUT_BOX_SIZE, SHORTCUT_RADIUS, item.bg_color);
        draw_rounded_outline(buf, screen_w, screen_h, x, icon_y, SHORTCUT_BOX_SIZE, SHORTCUT_BOX_SIZE, SHORTCUT_RADIUS, item.border_color);

        // Se em hover, adiciona uma aura sutil de brilho branco
        if is_hover {
            draw_rounded_outline(buf, screen_w, screen_h, x.saturating_sub(1), icon_y.saturating_sub(1), SHORTCUT_BOX_SIZE + 2, SHORTCUT_BOX_SIZE + 2, SHORTCUT_RADIUS + 1, COLOR_WHITE);
        }

        // 3. Ícone monospaçado no centro do squircle
        let icon_w = item.icon_text.len() * FONT_WIDTH;
        let icon_x = x + (SHORTCUT_BOX_SIZE.saturating_sub(icon_w)) / 2;
        let icon_text_y = icon_y + (SHORTCUT_BOX_SIZE.saturating_sub(FONT_HEIGHT)) / 2;
        draw_string(buf, screen_w, screen_h, item.icon_text, icon_x, icon_text_y, item.fg_color, Some(item.bg_color));

        // 4. Rótulo inferior com sombra suave
        let label_w = item.label.len() * FONT_WIDTH;
        let label_x = (x + SHORTCUT_BOX_SIZE / 2).saturating_sub(label_w / 2);
        let label_y = icon_y + SHORTCUT_BOX_SIZE + 5;

        // Sombra de texto para legibilidade sobre qualquer fundo
        draw_string(buf, screen_w, screen_h, item.label, label_x + 1, label_y + 1, 0x0005_070A, None);
        draw_string(buf, screen_w, screen_h, item.label, label_x, label_y, if is_hover { COLOR_CYAN_NEON } else { COLOR_TEXT_PRI }, None);
    }
}

pub fn handle_shortcuts_click(
    screen_w: usize,
    screen_h: usize,
    mx: usize,
    my: usize,
) -> Option<usize> {
    if screen_w < 500 { return None; }

    for i in 0..DESKTOP_SHORTCUTS.len() {
        let (x, y) = get_shortcut_pos(screen_w, i);
        if y + SHORTCUT_BOX_SIZE + 20 >= screen_h { break; }

        if mx >= x && mx < x + SHORTCUT_BOX_SIZE && my >= y && my < y + SHORTCUT_BOX_SIZE + 20 {
            return Some(i);
        }
    }
    None
}
