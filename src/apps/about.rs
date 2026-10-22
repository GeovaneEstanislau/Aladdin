//! Aplicativo Sobre o Jinn OS

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
        let lines = [
            "Jinn OS - Sistema Operacional em Rust",
            "Foco: Leveza, responsividade e preempcao.",
            "Zero abstracoes pesadas no escalonador.",
            "Comunidade: github.com/GeovaneEstanislau/Jinn_Project",
            "Enalteca a tecnologia brasileira!",
            "Pressione [ESC] para retornar ao Shell texto.",
        ];

        for (i, &line) in lines.iter().enumerate() {
            let line_y = win_y + 36 + (i * 20);
            draw_string(buf, screen_w, screen_h, line, win_x + 12, line_y, COLOR_TEXT_PRI, Some(COLOR_CARD));
        }
    }
}
