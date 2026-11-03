//! Aladdin UI — Biblioteca de Componentes Reutilizáveis (Design System 0.5)
//! Fornece widgets padronizados para aplicações nativas e extensões gráficas:
//! Buttons, Text Inputs, Badges, Dialogs, TabBars, ProgressBars e Tooltips.

use crate::compositor::*;
use crate::font::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Normal,
    Hover,
    Active,
    Disabled,
}

/// Renderiza um botão estilizado de acordo com o Design System Aladdin
pub fn draw_button(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    text: &str,
    style: ButtonStyle,
    state: WidgetState,
) {
    let (bg_color, border_color, text_color) = match (style, state) {
        (ButtonStyle::Primary, WidgetState::Normal) => (COLOR_BLUE_PRIMARY, COLOR_CYAN_NEON, COLOR_WHITE),
        (ButtonStyle::Primary, WidgetState::Hover) => (COLOR_BLUE_HOVER, COLOR_CYAN_NEON, COLOR_WHITE),
        (ButtonStyle::Primary, WidgetState::Active) => (0x001D_4ED8, COLOR_WHITE, COLOR_WHITE),
        (ButtonStyle::Primary, WidgetState::Disabled) => (COLOR_SURFACE_L3, COLOR_BORDER_SUBTLE, COLOR_TEXT_MUTED),

        (ButtonStyle::Secondary, WidgetState::Normal) => (COLOR_SURFACE_L3, COLOR_BORDER, COLOR_TEXT_PRI),
        (ButtonStyle::Secondary, WidgetState::Hover) => (COLOR_SURFACE_HOVER, COLOR_CYAN_NEON, COLOR_TEXT_PRI),
        (ButtonStyle::Secondary, WidgetState::Active) => (COLOR_SURFACE_ACTIVE, COLOR_CYAN_NEON, COLOR_TEXT_PRI),
        (ButtonStyle::Secondary, WidgetState::Disabled) => (COLOR_SURFACE_L1, COLOR_BORDER_SUBTLE, COLOR_TEXT_MUTED),

        (ButtonStyle::Success, WidgetState::Normal) => (0x0015_803D, COLOR_EMERALD, COLOR_WHITE),
        (ButtonStyle::Success, WidgetState::Hover) => (COLOR_EMERALD, COLOR_WHITE, COLOR_WHITE),
        (ButtonStyle::Success, WidgetState::Active) => (0x0016_6534, COLOR_EMERALD, COLOR_WHITE),
        (ButtonStyle::Success, WidgetState::Disabled) => (COLOR_SURFACE_L3, COLOR_BORDER_SUBTLE, COLOR_TEXT_MUTED),

        (ButtonStyle::Danger, WidgetState::Normal) => (COLOR_RUBY, 0x00F8_7171, COLOR_WHITE),
        (ButtonStyle::Danger, WidgetState::Hover) => (0x00DC_2626, COLOR_WHITE, COLOR_WHITE),
        (ButtonStyle::Danger, WidgetState::Active) => (0x00B9_1C1C, COLOR_RUBY, COLOR_WHITE),
        (ButtonStyle::Danger, WidgetState::Disabled) => (COLOR_SURFACE_L3, COLOR_BORDER_SUBTLE, COLOR_TEXT_MUTED),

        (ButtonStyle::Ghost, WidgetState::Normal) => (0, 0, COLOR_TEXT_SEC),
        (ButtonStyle::Ghost, WidgetState::Hover) => (COLOR_SURFACE_L3, COLOR_BORDER_SUBTLE, COLOR_CYAN_NEON),
        (ButtonStyle::Ghost, WidgetState::Active) => (COLOR_SURFACE_ACTIVE, COLOR_CYAN_NEON, COLOR_CYAN_NEON),
        (ButtonStyle::Ghost, WidgetState::Disabled) => (0, 0, COLOR_TEXT_MUTED),
    };

    if bg_color != 0 {
        fill_rect(buf, screen_w, screen_h, x, y, width, height, bg_color);
    }
    if border_color != 0 {
        draw_rect_outline(buf, screen_w, screen_h, x, y, width, height, border_color);
    }

    let text_w = text.len() * FONT_WIDTH;
    let text_x = x + (width.saturating_sub(text_w) / 2);
    let text_y = y + (height.saturating_sub(FONT_HEIGHT) / 2);
    draw_string(buf, screen_w, screen_h, text, text_x, text_y, text_color, if bg_color != 0 { Some(bg_color) } else { None });
}

/// Renderiza um campo de entrada de texto estilizado
pub fn draw_text_input(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    text: &str,
    focused: bool,
    cursor_pos: usize,
) {
    let bg_color = COLOR_SURFACE_L3;
    let border_color = if focused { COLOR_CYAN_NEON } else { COLOR_BORDER };

    fill_rect(buf, screen_w, screen_h, x, y, width, height, bg_color);
    draw_rect_outline(buf, screen_w, screen_h, x, y, width, height, border_color);

    let text_x = x + 6;
    let text_y = y + (height.saturating_sub(FONT_HEIGHT) / 2);
    draw_string(buf, screen_w, screen_h, text, text_x, text_y, COLOR_TEXT_PRI, Some(bg_color));

    if focused {
        let cursor_x = text_x + cursor_pos.min(text.len()) * FONT_WIDTH;
        if cursor_x + 2 < x + width {
            fill_rect(buf, screen_w, screen_h, cursor_x, text_y, 2, FONT_HEIGHT, COLOR_CYAN_NEON);
        }
    }
}

/// Renderiza uma pílula / badge de telemetria ou status
pub fn draw_badge(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    text: &str,
    color: u32,
    bg_color: u32,
) {
    let width = text.len() * FONT_WIDTH + 12;
    let height = FONT_HEIGHT + 6;
    fill_rect(buf, screen_w, screen_h, x, y, width, height, bg_color);
    draw_rect_outline(buf, screen_w, screen_h, x, y, width, height, color);
    draw_string(buf, screen_w, screen_h, text, x + 6, y + 3, color, Some(bg_color));
}

/// Renderiza uma barra de abas (TabBar) moderna
pub fn draw_tab_bar(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    tabs: &[&str],
    active_tab: usize,
) {
    fill_rect(buf, screen_w, screen_h, x, y, width, height, COLOR_SURFACE_L1);
    draw_line(buf, screen_w, screen_h, x as isize, (y + height - 1) as isize, (x + width) as isize, (y + height - 1) as isize, COLOR_BORDER);

    if tabs.is_empty() { return; }
    let tab_w = (width / tabs.len()).max(40);

    for (i, &label) in tabs.iter().enumerate() {
        let tx = x + i * tab_w;
        let is_active = i == active_tab;
        let tab_bg = if is_active { COLOR_SURFACE_L2 } else { COLOR_SURFACE_L1 };
        let tab_fg = if is_active { COLOR_CYAN_NEON } else { COLOR_TEXT_MUTED };

        fill_rect(buf, screen_w, screen_h, tx, y, tab_w, height - 1, tab_bg);
        let tw = label.len() * FONT_WIDTH;
        let text_x = tx + (tab_w.saturating_sub(tw) / 2);
        let text_y = y + (height.saturating_sub(FONT_HEIGHT) / 2);
        draw_string(buf, screen_w, screen_h, label, text_x, text_y, tab_fg, Some(tab_bg));

        if is_active {
            fill_rect(buf, screen_w, screen_h, tx, y + height - 2, tab_w, 2, COLOR_CYAN_NEON);
        }
    }
}

/// Renderiza uma caixa de diálogo modal de alerta / confirmação
pub fn draw_dialog(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    title: &str,
    message: &str,
) {
    // Sombra projetada
    draw_drop_shadow(buf, screen_w, screen_h, x, y, width, height, 4, COLOR_SHADOW);

    // Corpo do diálogo
    fill_rect(buf, screen_w, screen_h, x, y, width, height, COLOR_CARD);
    draw_rect_outline(buf, screen_w, screen_h, x, y, width, height, COLOR_CYAN_NEON);

    // Barra de título do diálogo
    fill_rect(buf, screen_w, screen_h, x, y, width, 24, COLOR_TOPPANEL);
    draw_line(buf, screen_w, screen_h, x as isize, (y + 24) as isize, (x + width) as isize, (y + 24) as isize, COLOR_CYAN_NEON);
    draw_string(buf, screen_w, screen_h, title, x + 8, y + 4, COLOR_TEXT_PRI, Some(COLOR_TOPPANEL));

    // Mensagem
    draw_string(buf, screen_w, screen_h, message, x + 16, y + 40, COLOR_TEXT_SEC, Some(COLOR_CARD));

    // Botão de confirmação [ OK ]
    let btn_w = 64;
    let btn_h = 24;
    let btn_x = x + width.saturating_sub(btn_w + 16);
    let btn_y = y + height.saturating_sub(btn_h + 12);
    draw_button(buf, screen_w, screen_h, btn_x, btn_y, btn_w, btn_h, "OK", ButtonStyle::Primary, WidgetState::Normal);
}

/// Renderiza tooltip flutuante
pub fn draw_tooltip(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    text: &str,
) {
    let width = text.len() * FONT_WIDTH + 8;
    let height = FONT_HEIGHT + 6;
    let safe_x = x.min(screen_w.saturating_sub(width + 4));
    let safe_y = y.min(screen_h.saturating_sub(height + 4));

    fill_rect(buf, screen_w, screen_h, safe_x, safe_y, width, height, COLOR_SURFACE_L1);
    draw_rect_outline(buf, screen_w, screen_h, safe_x, safe_y, width, height, COLOR_BORDER_ACT);
    draw_string(buf, screen_w, screen_h, text, safe_x + 4, safe_y + 3, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L1));
}
