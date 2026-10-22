//! Aplicativo de Configurações e Personalização do Aladdin (Settings App)

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, SystemHost};

pub struct SettingsApp {
    pub accent_color: u32,
}

impl SettingsApp {
    pub const fn new() -> Self {
        Self {
            accent_color: COLOR_CYAN,
        }
    }

    pub fn render(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        win_x: usize,
        win_y: usize,
        win_w: usize,
        _win_h: usize,
        mode: CapabilityMode,
        sys: &dyn SystemHost,
    ) {
        draw_string(buf, screen_w, screen_h, "--- Ajustes do Aladdin Desktop ---", win_x + 12, win_y + 32, self.accent_color, Some(COLOR_CARD));

        // 1. Seção de Modo Gráfico (CapabilityMode)
        draw_string(buf, screen_w, screen_h, "Modo de Composicao Visual:", win_x + 12, win_y + 54, COLOR_TEXT_PRI, Some(COLOR_CARD));

        let btn_w = 100;
        let btn_h = 22;
        let b0_x = win_x + 12;
        let b1_x = b0_x + btn_w + 8;
        let b2_x = b1_x + btn_w + 8;
        let btn_y = win_y + 72;

        // Botão Básico (Eco)
        let is_m0 = mode == CapabilityMode::Basic;
        fill_rect(buf, screen_w, screen_h, b0_x, btn_y, btn_w, btn_h, if is_m0 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b0_x, btn_y, btn_w, btn_h, if is_m0 { COLOR_EMERALD } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "1. Eco (0% FX)", b0_x + 6, btn_y + 6, if is_m0 { COLOR_EMERALD } else { COLOR_TEXT_MUTED }, None);

        // Botão Equilibrado
        let is_m1 = mode == CapabilityMode::Balanced;
        fill_rect(buf, screen_w, screen_h, b1_x, btn_y, btn_w, btn_h, if is_m1 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b1_x, btn_y, btn_w, btn_h, if is_m1 { COLOR_CYAN } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "2. Equilibrado", b1_x + 6, btn_y + 6, if is_m1 { COLOR_CYAN } else { COLOR_TEXT_MUTED }, None);

        // Botão Avançado (Glow)
        let is_m2 = mode == CapabilityMode::Advanced;
        fill_rect(buf, screen_w, screen_h, b2_x, btn_y, btn_w, btn_h, if is_m2 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b2_x, btn_y, btn_w, btn_h, if is_m2 { COLOR_PURPLE } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "3. Glow Neon", b2_x + 8, btn_y + 6, if is_m2 { COLOR_PURPLE } else { COLOR_TEXT_MUTED }, None);

        // 2. Seção de Hardware e Tela
        let hw_y = win_y + 104;
        fill_rect(buf, screen_w, screen_h, win_x + 12, hw_y, win_w.saturating_sub(24), 58, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, hw_y, win_w.saturating_sub(24), 58, COLOR_BORDER);

        draw_string(buf, screen_w, screen_h, "Resolucao : 1024x768x32 bpp (Limine GOP)", win_x + 20, hw_y + 8, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Cursor    : Rastreamento Rapido < 5 us (DirtyRect)", win_x + 20, hw_y + 24, COLOR_EMERALD, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Storage   : Perfil JSA ativo: ", win_x + 20, hw_y + 40, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, sys.storage_profile_name(), win_x + 244, hw_y + 40, COLOR_CYAN, Some(COLOR_CARD_ALT));

        // 3. Botões de Ação Rápida
        let act_y = win_y + 172;

        // Botão Ciclar Storage
        fill_rect(buf, screen_w, screen_h, win_x + 12, act_y, 140, 24, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, act_y, 140, 24, 0x0093_C5FD);
        draw_string(buf, screen_w, screen_h, "Ciclar JSA", win_x + 24, act_y + 6, COLOR_WHITE, Some(COLOR_BTN_BG));

        // Botão Reiniciar Máquina
        fill_rect(buf, screen_w, screen_h, win_x + 160, act_y, 140, 24, 0x002D_1215);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 160, act_y, 140, 24, COLOR_RUBY);
        draw_string(buf, screen_w, screen_h, "Reiniciar SO", win_x + 180, act_y + 6, 0x00F8_7171, Some(0x002D_1215));
    }
}
