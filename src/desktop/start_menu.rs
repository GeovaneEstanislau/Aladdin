//! Menu Iniciar Aladdin Applications Popup

use crate::compositor::*;
use crate::font::*;

pub const MENU_X: usize = 8;
pub const MENU_Y: usize = 28;
pub const MENU_W: usize = 260;
pub const MENU_H: usize = 244;

pub fn render_start_menu(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mouse_x: usize,
    mouse_y: usize,
) {
    // Sombra do menu
    fill_rect(buf, screen_w, screen_h, MENU_X + 3, MENU_Y + 3, MENU_W, MENU_H, 0x0005_0710);
    // Superfície escura
    fill_rect(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, 0x0011_1827);
    draw_rect_outline(buf, screen_w, screen_h, MENU_X, MENU_Y, MENU_W, MENU_H, COLOR_CYAN);

    // Cabeçalho do menu
    fill_rect(buf, screen_w, screen_h, MENU_X + 1, MENU_Y + 1, MENU_W - 2, 24, 0x000F_172A);
    draw_string(buf, screen_w, screen_h, "Aladdin OS 7 Apps", MENU_X + 10, MENU_Y + 5, COLOR_CYAN, Some(0x000F_172A));

    let items = [
        "1. Monitor de Recursos",
        "2. Terminal Interativo",
        "3. Arquivos /disk (FAT32)",
        "4. Executar ola-mundo (U3)",
        "5. Sobre o Jinn OS",
        "6. Ciclar Perfil JSA",
        "7. Reiniciar Maquina",
        "8. Sair para Shell (ESC)",
    ];

    for (idx, &item) in items.iter().enumerate() {
        let item_y = MENU_Y + 28 + (idx * 26);
        let is_hover = mouse_x >= MENU_X && mouse_x <= MENU_X + MENU_W && mouse_y >= item_y && mouse_y < item_y + 26;
        let item_bg = if is_hover { COLOR_CARD_ALT } else { 0x0011_1827 };
        let item_fg = if is_hover { COLOR_CYAN } else { COLOR_TEXT_PRI };
        fill_rect(buf, screen_w, screen_h, MENU_X + 2, item_y, MENU_W - 4, 25, item_bg);
        draw_string(buf, screen_w, screen_h, item, MENU_X + 10, item_y + 5, item_fg, Some(item_bg));
    }
}
