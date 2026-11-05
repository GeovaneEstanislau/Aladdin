//! Menu Iniciar Aladdin Applications Popup (Design System 0.5)

use crate::compositor::*;
use crate::font::*;

pub const MENU_X: usize = 8;
pub const MENU_Y: usize = 28;
pub const MENU_W: usize = 270;
pub const MENU_H: usize = 280;

pub fn render_start_menu(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mouse_x: usize,
    mouse_y: usize,
) {
    // 1. Sombra de elevação arredondada
    draw_drop_shadow_rounded(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, 8, 4, COLOR_SHADOW);

    // 2. Fundo acrílico escuro com borda luminescente (R=8)
    fill_rounded_rect(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, 8, COLOR_SURFACE_L1);
    draw_rounded_outline(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, 8, COLOR_CYAN_NEON);

    // 3. Cabeçalho moderno
    fill_rounded_top(buf, screen_w, screen_h, MENU_X + 1, MENU_Y + 1, MENU_W - 2, 26, 7, COLOR_SURFACE_L0);
    draw_string(buf, screen_w, screen_h, "Aladdin Aplicativos", MENU_X + 12, MENU_Y + 7, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L0));
    draw_line(buf, screen_w, screen_h, MENU_X as isize, (MENU_Y + 27) as isize, (MENU_X + MENU_W) as isize, (MENU_Y + 27) as isize, COLOR_BORDER_SUBTLE);

    let items = [
        ("Monitor de Recursos", COLOR_EMERALD, false),
        ("Terminal Interativo", COLOR_CYAN_NEON, false),
        ("Arquivos /disk (FAT32)", 0x003B_82F6, false),
        ("Jinn Browser (Web)", COLOR_PURPLE, false),
        ("Gerenciador de Discos", COLOR_PURPLE, false),
        ("Sobre o Jinn OS", COLOR_TEXT_SEC, false),
        ("Ciclar Perfil JSA", COLOR_AMBER, false),
        ("Reiniciar Maquina", COLOR_AMBER, false),
        ("Sair para Shell (ESC)", COLOR_RUBY, true),
    ];

    for (idx, &(title, item_color, is_exit)) in items.iter().enumerate() {
        let item_y = MENU_Y + 32 + (idx * 27);
        let is_hover = mouse_x >= MENU_X + 2 && mouse_x <= MENU_X + MENU_W - 2 && mouse_y >= item_y && mouse_y < item_y + 26;
        let item_bg = if is_hover { COLOR_SURFACE_HOVER } else { COLOR_SURFACE_L1 };

        if is_hover {
            fill_rounded_rect(buf, screen_w, screen_h, MENU_X + 4, item_y, MENU_W - 8, 24, 4, item_bg);
            draw_rounded_outline(buf, screen_w, screen_h, MENU_X + 4, item_y, MENU_W - 8, 24, 4, COLOR_BORDER_SUBTLE);
        }

        // Tag [X] apenas para ação de encerramento / saída
        if is_exit {
            draw_string(buf, screen_w, screen_h, "[X]", MENU_X + 10, item_y + 6, COLOR_RUBY, Some(item_bg));
            let text_color = if is_hover { COLOR_WHITE } else { COLOR_TEXT_PRI };
            draw_string(buf, screen_w, screen_h, title, MENU_X + 40, item_y + 6, text_color, Some(item_bg));
        } else {
            let text_color = if is_hover { COLOR_WHITE } else { item_color };
            draw_string(buf, screen_w, screen_h, title, MENU_X + 14, item_y + 6, text_color, Some(item_bg));
        }

        // Divisor sutil antes das ações de energia (Reiniciar/Sair)
        if idx == 6 {
            draw_line(buf, screen_w, screen_h, (MENU_X + 6) as isize, (item_y + 26) as isize, (MENU_X + MENU_W - 6) as isize, (item_y + 26) as isize, COLOR_BORDER_SUBTLE);
        }
    }
}
