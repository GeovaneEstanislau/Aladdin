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
    // 1. Sombra de elevação
    draw_drop_shadow(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, 4, COLOR_SHADOW);

    // 2. Fundo acrílico escuro com borda luminescente
    fill_rect(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, COLOR_SURFACE_L1);
    draw_rect_outline(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, COLOR_CYAN_NEON);

    // 3. Cabeçalho moderno
    fill_rect(buf, screen_w, screen_h, MENU_X + 1, MENU_Y + 1, MENU_W - 2, 26, COLOR_SURFACE_L0);
    draw_char(buf, screen_w, screen_h, b'#', MENU_X + 10, MENU_Y + 7, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L0));
    draw_string(buf, screen_w, screen_h, "Aladdin OS 7 Aplicativos", MENU_X + 22, MENU_Y + 7, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L0));
    draw_line(buf, screen_w, screen_h, MENU_X as isize, (MENU_Y + 27) as isize, (MENU_X + MENU_W) as isize, (MENU_Y + 27) as isize, COLOR_BORDER_SUBTLE);

    let items = [
        ("[#]", "Monitor de Recursos", COLOR_EMERALD),
        ("[>]", "Terminal Interativo", COLOR_CYAN_NEON),
        ("[~]", "Arquivos /disk (FAT32)", 0x003B_82F6),
        ("[@]", "Jinn Browser (Web)", COLOR_PURPLE),
        ("[D]", "Gerenciador de Discos", COLOR_PURPLE),
        ("[?]", "Sobre o Jinn OS", COLOR_TEXT_SEC),
        ("[%]", "Ciclar Perfil JSA", COLOR_AMBER),
        ("[R]", "Reiniciar Maquina", COLOR_AMBER),
        ("[X]", "Sair para Shell (ESC)", COLOR_RUBY),
    ];

    for (idx, &(icon, title, icon_color)) in items.iter().enumerate() {
        let item_y = MENU_Y + 32 + (idx * 27);
        let is_hover = mouse_x >= MENU_X + 2 && mouse_x <= MENU_X + MENU_W - 2 && mouse_y >= item_y && mouse_y < item_y + 26;
        let item_bg = if is_hover { COLOR_SURFACE_HOVER } else { COLOR_SURFACE_L1 };

        fill_rect(buf, screen_w, screen_h, MENU_X + 2, item_y, MENU_W - 4, 26, item_bg);
        if is_hover {
            draw_rect_outline(buf, screen_w, screen_h, MENU_X + 2, item_y, MENU_W - 4, 26, COLOR_BORDER_SUBTLE);
        }

        // Ícone colorido com espaçamento elegante
        draw_string(buf, screen_w, screen_h, icon, MENU_X + 8, item_y + 6, icon_color, Some(item_bg));

        let text_color = if is_hover { COLOR_WHITE } else { COLOR_TEXT_PRI };
        draw_string(buf, screen_w, screen_h, title, MENU_X + 40, item_y + 6, text_color, Some(item_bg));

        // Divisor sutil antes das ações de energia (Reiniciar/Sair)
        if idx == 6 {
            draw_line(buf, screen_w, screen_h, (MENU_X + 6) as isize, (item_y + 26) as isize, (MENU_X + MENU_W - 6) as isize, (item_y + 26) as isize, COLOR_BORDER_SUBTLE);
        }
    }
}
