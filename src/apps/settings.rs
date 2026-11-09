//! Aplicativo de Configuracoes e Personalizacao do Aladdin (Settings App)

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost};
use crate::contracts::vfs::VfsHost;

pub struct SettingsApp {
    pub accent_color: u32,
    pub wallpaper_id: u8,
    pub status_msg: &'static str,
}

impl SettingsApp {
    pub const fn new() -> Self {
        Self {
            accent_color: COLOR_CYAN,
            wallpaper_id: 0,
            status_msg: "[V] Preferencias ativas",
        }
    }

    /// Salva configurações atuais no arquivo `/disk/SETTINGS.CFG` no formato texto simples
    pub fn save_to_vfs(&mut self, vfs: &dyn VfsHost, mode: CapabilityMode) -> Result<(), &'static str> {
        let mode_val: u8 = match mode {
            CapabilityMode::Basic => 0,
            CapabilityMode::Balanced => 1,
            CapabilityMode::Advanced => 2,
        };

        let mut buf = [0u8; 96];
        let mut len = 0;

        let prefix_mode = b"MODE=";
        buf[len..len + prefix_mode.len()].copy_from_slice(prefix_mode);
        len += prefix_mode.len();
        buf[len] = b'0' + mode_val;
        len += 1;
        buf[len] = b'\n';
        len += 1;

        let prefix_wp = b"WALLPAPER=";
        buf[len..len + prefix_wp.len()].copy_from_slice(prefix_wp);
        len += prefix_wp.len();
        buf[len] = b'0' + self.wallpaper_id;
        len += 1;
        buf[len] = b'\n';
        len += 1;

        // Garante que o arquivo existe antes de gravar
        let _ = vfs.create_file("/disk/SETTINGS.CFG");
        if vfs.write_file("/disk/SETTINGS.CFG", &buf[..len]).is_err() {
            self.status_msg = "[X] Erro ao gravar /disk/SETTINGS.CFG";
            return Err("Falha na gravacao");
        }

        self.status_msg = "[V] Ajustes salvos no disco com sucesso";
        Ok(())
    }

    /// Carrega preferências do usuário a partir do arquivo `/disk/SETTINGS.CFG`
    pub fn load_from_vfs(&mut self, vfs: &dyn VfsHost) -> Option<CapabilityMode> {
        let mut buf = [0u8; 128];
        let bytes_read = vfs.read_file("/disk/SETTINGS.CFG", &mut buf).ok()?;
        if bytes_read == 0 {
            return None;
        }

        let text = core::str::from_utf8(&buf[..bytes_read]).ok()?;
        let mut loaded_mode = None;

        for line in text.lines() {
            let line = line.trim();
            if let Some(val) = line.strip_prefix("MODE=") {
                if let Ok(m) = val.parse::<u8>() {
                    loaded_mode = match m {
                        0 => Some(CapabilityMode::Basic),
                        1 => Some(CapabilityMode::Balanced),
                        2 => Some(CapabilityMode::Advanced),
                        _ => None,
                    };
                }
            } else if let Some(val) = line.strip_prefix("WALLPAPER=") {
                if let Ok(w) = val.parse::<u8>() {
                    if w <= 4 {
                        self.wallpaper_id = w;
                    }
                }
            }
        }

        if loaded_mode.is_some() {
            self.status_msg = "[V] Ajustes carregados do disco";
        }
        loaded_mode
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
        draw_string(buf, screen_w, screen_h, "--- Ajustes do Aladdin Desktop ---", win_x + 12, win_y + 30, self.accent_color, Some(COLOR_CARD));

        // 1. Secao de Modo Grafico (CapabilityMode)
        draw_string(buf, screen_w, screen_h, "Modo de Composicao Visual:", win_x + 12, win_y + 48, COLOR_TEXT_PRI, Some(COLOR_CARD));

        let btn_w = 124;
        let btn_h = 20;
        let b0_x = win_x + 12;
        let b1_x = b0_x + btn_w + 8;
        let b2_x = b1_x + btn_w + 8;
        let btn_y = win_y + 64;

        // Botao Basico (Eco)
        let is_m0 = mode == CapabilityMode::Basic;
        fill_rect(buf, screen_w, screen_h, b0_x, btn_y, btn_w, btn_h, if is_m0 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b0_x, btn_y, btn_w, btn_h, if is_m0 { COLOR_EMERALD } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "1. Eco (0% FX)", b0_x + 8, btn_y + 5, if is_m0 { COLOR_EMERALD } else { COLOR_TEXT_MUTED }, None);

        // Botao Equilibrado
        let is_m1 = mode == CapabilityMode::Balanced;
        fill_rect(buf, screen_w, screen_h, b1_x, btn_y, btn_w, btn_h, if is_m1 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b1_x, btn_y, btn_w, btn_h, if is_m1 { COLOR_CYAN } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "2. Equilibrado", b1_x + 8, btn_y + 5, if is_m1 { COLOR_CYAN } else { COLOR_TEXT_MUTED }, None);

        // Botao Avancado (Glow)
        let is_m2 = mode == CapabilityMode::Advanced;
        fill_rect(buf, screen_w, screen_h, b2_x, btn_y, btn_w, btn_h, if is_m2 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b2_x, btn_y, btn_w, btn_h, if is_m2 { COLOR_PURPLE } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "3. Glow Neon", b2_x + 12, btn_y + 5, if is_m2 { COLOR_PURPLE } else { COLOR_TEXT_MUTED }, None);

        // 2. Secao de Papel de Parede (Wallpaper)
        let wp_y = win_y + 90;
        draw_string(buf, screen_w, screen_h, "Papel de Parede (Wallpaper):", win_x + 12, wp_y, COLOR_TEXT_PRI, Some(COLOR_CARD));

        let wp_btn_y = wp_y + 16;
        let is_w0 = self.wallpaper_id == 0;
        fill_rect(buf, screen_w, screen_h, b0_x, wp_btn_y, btn_w, btn_h, if is_w0 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b0_x, wp_btn_y, btn_w, btn_h, if is_w0 { COLOR_CYAN } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "1. Cyber-Aurora", b0_x + 6, wp_btn_y + 5, if is_w0 { COLOR_CYAN } else { COLOR_TEXT_MUTED }, None);

        let is_w1 = self.wallpaper_id == 1;
        fill_rect(buf, screen_w, screen_h, b1_x, wp_btn_y, btn_w, btn_h, if is_w1 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b1_x, wp_btn_y, btn_w, btn_h, if is_w1 { COLOR_EMERALD } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "2. Midnight", b1_x + 14, wp_btn_y + 5, if is_w1 { COLOR_EMERALD } else { COLOR_TEXT_MUTED }, None);

        let is_w2 = self.wallpaper_id == 2;
        fill_rect(buf, screen_w, screen_h, b2_x, wp_btn_y, btn_w, btn_h, if is_w2 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, b2_x, wp_btn_y, btn_w, btn_h, if is_w2 { COLOR_PURPLE } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "3. Techno Grid", b2_x + 8, wp_btn_y + 5, if is_w2 { COLOR_PURPLE } else { COLOR_TEXT_MUTED }, None);

        // 3. Secao de Persistencia em Disco
        let pers_y = win_y + 134;
        let pers_w = win_w.saturating_sub(24);
        fill_rect(buf, screen_w, screen_h, win_x + 12, pers_y, pers_w, 44, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, pers_y, pers_w, 44, COLOR_BORDER);

        draw_string(buf, screen_w, screen_h, "Persistencia :", win_x + 20, pers_y + 6, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, self.status_msg, win_x + 130, pers_y + 6, COLOR_EMERALD, Some(COLOR_CARD_ALT));

        let btn_save_y = pers_y + 22;
        fill_rect(buf, screen_w, screen_h, win_x + 20, btn_save_y, 250, 18, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 20, btn_save_y, 250, 18, COLOR_CYAN);
        draw_string(buf, screen_w, screen_h, "[V] Salvar Preferencias no Disco", win_x + 28, btn_save_y + 3, COLOR_WHITE, Some(COLOR_BTN_BG));

        // 4. Secao de Hardware e Tela
        let hw_y = win_y + 184;
        fill_rect(buf, screen_w, screen_h, win_x + 12, hw_y, pers_w, 46, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, hw_y, pers_w, 46, COLOR_BORDER);

        draw_string(buf, screen_w, screen_h, "Resolucao : 1024x768x32 bpp (Limine GOP)", win_x + 20, hw_y + 6, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Cursor    : Rastreamento Rapido < 5 us (DirtyRect)", win_x + 20, hw_y + 20, COLOR_EMERALD, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Storage   : Perfil JSA: ", win_x + 20, hw_y + 34, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, sys.storage_profile_name(), win_x + 160, hw_y + 34, COLOR_CYAN, Some(COLOR_CARD_ALT));

        // 5. Secao do Firewall SPI JinnWall
        let fw_y = win_y + 236;
        fill_rect(buf, screen_w, screen_h, win_x + 12, fw_y, pers_w, 48, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, fw_y, pers_w, 48, COLOR_BORDER);

        let prof = sys.jinnwall_profile();
        let is_sec = prof == JinnWallProfile::PublicSecure;
        draw_string(buf, screen_w, screen_h, "JinnWall SPI: ", win_x + 20, fw_y + 6, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        let badge_color = if is_sec { COLOR_EMERALD } else { COLOR_CYAN };
        draw_string(buf, screen_w, screen_h, if is_sec { "[ PublicSecure ]" } else { "[ Development ]" }, win_x + 120, fw_y + 6, badge_color, Some(COLOR_CARD_ALT));

        let btn_fw_y = fw_y + 24;
        fill_rect(buf, screen_w, screen_h, win_x + 20, btn_fw_y, 200, 18, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 20, btn_fw_y, 200, 18, 0x0038_BDF8);
        draw_string(buf, screen_w, screen_h, "Alternar Perfil Firewall", win_x + 28, btn_fw_y + 3, COLOR_WHITE, Some(COLOR_BTN_BG));

        // 6. Botoes de Acao Rapida
        let act_y = win_y + 290;

        // Botao Ciclar Storage
        fill_rect(buf, screen_w, screen_h, win_x + 12, act_y, 140, 22, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, act_y, 140, 22, 0x0093_C5FD);
        draw_string(buf, screen_w, screen_h, "Ciclar JSA", win_x + 24, act_y + 5, COLOR_WHITE, Some(COLOR_BTN_BG));

        // Botao Reiniciar Maquina
        fill_rect(buf, screen_w, screen_h, win_x + 160, act_y, 140, 22, 0x002D_1215);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 160, act_y, 140, 22, COLOR_RUBY);
        draw_string(buf, screen_w, screen_h, "Reiniciar SO", win_x + 180, act_y + 5, 0x00F8_7171, Some(0x002D_1215));
    }
}
