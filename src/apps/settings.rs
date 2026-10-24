//! Aplicativo de Configuracoes e Personalizacao do Aladdin (Settings App)

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost};

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

        // 1. Secao de Modo Grafico (CapabilityMode)
        draw_string(buf, screen_w, screen_h, "Modo de Composicao Visual:", win_x + 12, win_y + 52, COLOR_TEXT_PRI, Some(COLOR_CARD));

        let btn_w = 100;
        let btn_h = 22;
        let b0_x = win_x + 12;
        let b1_x = b0_x + btn_w + 8;
        let b2_x = b1_x + btn_w + 8;
        let btn_y = win_y + 70;

        // Botao Basico (Eco)
        let is_m0 = mode == CapabilityMode::Basic;
        fill_rect(buf, screen_w, screen_h, b0_x, btn_y, btn_w, btn_h, if is_m0 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b0_x, btn_y, btn_w, btn_h, if is_m0 { COLOR_EMERALD } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "1. Eco (0% FX)", b0_x + 6, btn_y + 6, if is_m0 { COLOR_EMERALD } else { COLOR_TEXT_MUTED }, None);

        // Botao Equilibrado
        let is_m1 = mode == CapabilityMode::Balanced;
        fill_rect(buf, screen_w, screen_h, b1_x, btn_y, btn_w, btn_h, if is_m1 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b1_x, btn_y, btn_w, btn_h, if is_m1 { COLOR_CYAN } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "2. Equilibrado", b1_x + 6, btn_y + 6, if is_m1 { COLOR_CYAN } else { COLOR_TEXT_MUTED }, None);

        // Botao Avancado (Glow)
        let is_m2 = mode == CapabilityMode::Advanced;
        fill_rect(buf, screen_w, screen_h, b2_x, btn_y, btn_w, btn_h, if is_m2 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b2_x, btn_y, btn_w, btn_h, if is_m2 { COLOR_PURPLE } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "3. Glow Neon", b2_x + 8, btn_y + 6, if is_m2 { COLOR_PURPLE } else { COLOR_TEXT_MUTED }, None);

        // 2. Secao de Hardware e Tela
        let hw_y = win_y + 100;
        fill_rect(buf, screen_w, screen_h, win_x + 12, hw_y, win_w.saturating_sub(24), 58, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, hw_y, win_w.saturating_sub(24), 58, COLOR_BORDER);

        draw_string(buf, screen_w, screen_h, "Resolucao : 1024x768x32 bpp (Limine GOP)", win_x + 20, hw_y + 8, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Cursor    : Rastreamento Rapido < 5 us (DirtyRect)", win_x + 20, hw_y + 24, COLOR_EMERALD, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Storage   : Perfil JSA ativo: ", win_x + 20, hw_y + 40, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, sys.storage_profile_name(), win_x + 244, hw_y + 40, COLOR_CYAN, Some(COLOR_CARD_ALT));

        // 3. Secao do Firewall SPI JinnWall
        let fw_y = win_y + 166;
        let fw_w = win_w.saturating_sub(24);
        fill_rect(buf, screen_w, screen_h, win_x + 12, fw_y, fw_w, 62, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, fw_y, fw_w, 62, COLOR_BORDER);

        let prof = sys.jinnwall_profile();
        let is_sec = prof == JinnWallProfile::PublicSecure;
        draw_string(buf, screen_w, screen_h, "JinnWall SPI: ", win_x + 20, fw_y + 8, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        let badge_color = if is_sec { COLOR_EMERALD } else { COLOR_CYAN };
        draw_string(buf, screen_w, screen_h, if is_sec { "[ PublicSecure ]" } else { "[ Development ]" }, win_x + 130, fw_y + 8, badge_color, Some(COLOR_CARD_ALT));

        let stats = sys.stats();
        draw_label_and_num(buf, screen_w, screen_h, "Conntrack : ", stats.jinnwall_active_conns, " fluxos ativos", win_x + 20, fw_y + 24, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        draw_label_and_num(buf, screen_w, screen_h, " | Drops: ", stats.jinnwall_drops as usize, " pkts", win_x + 230, fw_y + 24, COLOR_PURPLE, Some(COLOR_CARD_ALT));

        // Botao de Alternancia do Firewall
        let btn_fw_y = fw_y + 40;
        fill_rect(buf, screen_w, screen_h, win_x + 20, btn_fw_y, 220, 18, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 20, btn_fw_y, 220, 18, 0x0038_BDF8);
        draw_string(buf, screen_w, screen_h, "Alternar Perfil Firewall", win_x + 28, btn_fw_y + 4, COLOR_WHITE, Some(COLOR_BTN_BG));

        // 4. Botoes de Acao Rapida
        let act_y = win_y + 236;

        // Botao Ciclar Storage
        fill_rect(buf, screen_w, screen_h, win_x + 12, act_y, 140, 24, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, act_y, 140, 24, 0x0093_C5FD);
        draw_string(buf, screen_w, screen_h, "Ciclar JSA", win_x + 24, act_y + 6, COLOR_WHITE, Some(COLOR_BTN_BG));

        // Botao Reiniciar Maquina
        fill_rect(buf, screen_w, screen_h, win_x + 160, act_y, 140, 24, 0x002D_1215);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 160, act_y, 140, 24, COLOR_RUBY);
        draw_string(buf, screen_w, screen_h, "Reiniciar SO", win_x + 180, act_y + 6, 0x00F8_7171, Some(0x002D_1215));
    }
}
