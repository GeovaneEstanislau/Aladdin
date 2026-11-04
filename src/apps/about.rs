//! Aplicativo Sobre o Aladdin OS e o Ecossistema Jinn

use crate::compositor::*;
use crate::font::*;

pub struct AboutApp;

impl AboutApp {
    pub const fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        win_x: usize,
        win_y: usize,
    ) {
        // Banner de cabeçalho
        fill_rect(buf, screen_w, screen_h, win_x + 12, win_y + 32, 396, 32, COLOR_SURFACE_L1);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, win_y + 32, 396, 32, COLOR_BORDER_SUBTLE);
        draw_char(buf, screen_w, screen_h, b'#', win_x + 20, win_y + 40, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L1));
        draw_string(buf, screen_w, screen_h, "Aladdin Desktop 7.0 (Dr460nized)", win_x + 32, win_y + 40, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L1));

        // Badges de identidade
        let badge_y = win_y + 70;
        draw_pill(buf, screen_w, screen_h, win_x + 12, badge_y, 90, 20, COLOR_SURFACE_L3, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Kernel Jinn", win_x + 18, badge_y + 4, COLOR_EMERALD, Some(COLOR_SURFACE_L3));

        draw_pill(buf, screen_w, screen_h, win_x + 108, badge_y, 80, 20, COLOR_SURFACE_L3, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "x86_64 ELF", win_x + 114, badge_y + 4, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L3));

        draw_pill(buf, screen_w, screen_h, win_x + 194, badge_y, 110, 20, COLOR_SURFACE_L3, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Ring 0/Ring 3", win_x + 200, badge_y + 4, COLOR_PURPLE, Some(COLOR_SURFACE_L3));

        draw_pill(buf, screen_w, screen_h, win_x + 310, badge_y, 98, 20, COLOR_SURFACE_L3, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Zero-Forks", win_x + 316, badge_y + 4, COLOR_AMBER, Some(COLOR_SURFACE_L3));

        let specs = [
            ("Nucleo Compartilhado", "Jinn Microkernel (ABI v0.3.0 Canônica)"),
            ("Design Visual       ", "elementary OS 7 + Garuda Dr460nized"),
            ("Compositor Grafico  ", "Acelerado por Software com Scissor Clipping"),
            ("Sistemas de Arquivo ", "VFS com Suporte a FAT32 (RW) e Ext2 (RO)"),
            ("Firewall Integrado  ", "JinnWall SPI com Filtragem de Estado"),
        ];

        for (i, &(label, val)) in specs.iter().enumerate() {
            let row_y = win_y + 100 + (i * 18);
            draw_string(buf, screen_w, screen_h, label, win_x + 14, row_y, COLOR_TEXT_MUTED, Some(COLOR_CARD));
            draw_string(buf, screen_w, screen_h, ": ", win_x + 160, row_y, COLOR_BORDER, Some(COLOR_CARD));
            draw_string(buf, screen_w, screen_h, val, win_x + 172, row_y, COLOR_TEXT_PRI, Some(COLOR_CARD));
        }

        // Rodapé de créditos
        draw_line(buf, screen_w, screen_h, (win_x + 12) as isize, (win_y + 192) as isize, (win_x + 408) as isize, (win_y + 192) as isize, COLOR_BORDER_SUBTLE);
        draw_string(buf, screen_w, screen_h, "github.com/GeovaneEstanislau/Jinn_Project", win_x + 24, win_y + 198, COLOR_CYAN_NEON, Some(COLOR_CARD));
    }
}
