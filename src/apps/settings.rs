//! Aplicativo de Configuracoes e Personalizacao do Aladdin (Settings App)

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost};
use crate::contracts::vfs::VfsHost;

pub struct SettingsApp {
    pub accent_color: u32,
    pub wallpaper_id: u8,
    pub wallpaper_audio_enabled: bool,
    pub wallpaper_audio_volume: u8,
    pub status_msg: &'static str,
}

impl SettingsApp {
    pub const fn new() -> Self {
        Self {
            accent_color: COLOR_CYAN,
            wallpaper_id: 0,
            wallpaper_audio_enabled: false,
            wallpaper_audio_volume: 40,
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

        let mut buf = [0u8; 192];
        let mut len = 0;

        let header = b"# ALADDIN_CONFIG_V1\n";
        buf[..header.len()].copy_from_slice(header);
        len += header.len();

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
        if self.wallpaper_id < 10 {
            buf[len] = b'0' + self.wallpaper_id;
            len += 1;
        } else {
            buf[len] = b'1';
            buf[len + 1] = b'0' + (self.wallpaper_id - 10);
            len += 2;
        }
        buf[len] = b'\n';
        len += 1;

        let prefix_aud = b"WALLPAPER_AUDIO=";
        buf[len..len + prefix_aud.len()].copy_from_slice(prefix_aud);
        len += prefix_aud.len();
        buf[len] = if self.wallpaper_audio_enabled { b'1' } else { b'0' };
        len += 1;
        buf[len] = b'\n';
        len += 1;

        let prefix_vol = b"WALLPAPER_VOL=";
        buf[len..len + prefix_vol.len()].copy_from_slice(prefix_vol);
        len += prefix_vol.len();
        let vol_tens = self.wallpaper_audio_volume / 10;
        let vol_ones = self.wallpaper_audio_volume % 10;
        buf[len] = b'0' + vol_tens;
        buf[len + 1] = b'0' + vol_ones;
        len += 2;
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
        let mut buf = [0u8; 192];
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
                    if w <= 11 {
                        self.wallpaper_id = w;
                    }
                }
            } else if let Some(val) = line.strip_prefix("WALLPAPER_AUDIO=") {
                if let Ok(a) = val.parse::<u8>() {
                    self.wallpaper_audio_enabled = a != 0;
                }
            } else if let Some(val) = line.strip_prefix("WALLPAPER_VOL=") {
                if let Ok(v) = val.parse::<u8>() {
                    self.wallpaper_audio_volume = v.min(100);
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
        draw_string(buf, screen_w, screen_h, "Papel de Parede (Estaticos, Dominio Publico & Videos):", win_x + 12, wp_y, COLOR_TEXT_PRI, Some(COLOR_CARD));

        // Linha 1: Estaticos Procedurais (IDs 0..4)
        let row1_y = wp_y + 16;
        let w_s5 = 80;
        let p0_x = win_x + 12;
        let p1_x = p0_x + w_s5 + 6;
        let p2_x = p1_x + w_s5 + 6;
        let p3_x = p2_x + w_s5 + 6;
        let p4_x = p3_x + w_s5 + 6;

        let is_w0 = self.wallpaper_id == 0;
        fill_rect(buf, screen_w, screen_h, p0_x, row1_y, w_s5, 18, if is_w0 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, p0_x, row1_y, w_s5, 18, if is_w0 { COLOR_CYAN } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "1. Aurora", p0_x + 6, row1_y + 4, if is_w0 { COLOR_CYAN } else { COLOR_TEXT_MUTED }, None);

        let is_w1 = self.wallpaper_id == 1;
        fill_rect(buf, screen_w, screen_h, p1_x, row1_y, w_s5, 18, if is_w1 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, p1_x, row1_y, w_s5, 18, if is_w1 { COLOR_EMERALD } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "2. Midnite", p1_x + 6, row1_y + 4, if is_w1 { COLOR_EMERALD } else { COLOR_TEXT_MUTED }, None);

        let is_w2 = self.wallpaper_id == 2;
        fill_rect(buf, screen_w, screen_h, p2_x, row1_y, w_s5, 18, if is_w2 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, p2_x, row1_y, w_s5, 18, if is_w2 { COLOR_PURPLE } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "3. Techno", p2_x + 6, row1_y + 4, if is_w2 { COLOR_PURPLE } else { COLOR_TEXT_MUTED }, None);

        let is_w3 = self.wallpaper_id == 3;
        fill_rect(buf, screen_w, screen_h, p3_x, row1_y, w_s5, 18, if is_w3 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, p3_x, row1_y, w_s5, 18, if is_w3 { COLOR_AMBER } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "4. Solaris", p3_x + 6, row1_y + 4, if is_w3 { COLOR_AMBER } else { COLOR_TEXT_MUTED }, None);

        let is_w4 = self.wallpaper_id == 4;
        fill_rect(buf, screen_w, screen_h, p4_x, row1_y, w_s5, 18, if is_w4 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, p4_x, row1_y, w_s5, 18, if is_w4 { COLOR_CYAN_NEON } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "5. Obsid", p4_x + 8, row1_y + 4, if is_w4 { COLOR_CYAN_NEON } else { COLOR_TEXT_MUTED }, None);

        // Linha 2: Dominio Publico (CC0 / NASA) (IDs 5..7)
        let row2_y = row1_y + 22;
        let w_pub = 135;
        let pb0_x = win_x + 12;
        let pb1_x = pb0_x + w_pub + 8;
        let pb2_x = pb1_x + w_pub + 8;

        let is_w5 = self.wallpaper_id == 5;
        fill_rect(buf, screen_w, screen_h, pb0_x, row2_y, w_pub, 18, if is_w5 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, pb0_x, row2_y, w_pub, 18, if is_w5 { COLOR_AMBER } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "6. NASA Nebula (CC0)", pb0_x + 4, row2_y + 4, if is_w5 { COLOR_AMBER } else { COLOR_TEXT_MUTED }, None);

        let is_w6 = self.wallpaper_id == 6;
        fill_rect(buf, screen_w, screen_h, pb1_x, row2_y, w_pub, 18, if is_w6 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, pb1_x, row2_y, w_pub, 18, if is_w6 { COLOR_RUBY } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "7. Mt Fuji (CC0)", pb1_x + 6, row2_y + 4, if is_w6 { 0x00F4_3F5E } else { COLOR_TEXT_MUTED }, None);

        let is_w7 = self.wallpaper_id == 7;
        fill_rect(buf, screen_w, screen_h, pb2_x, row2_y, w_pub, 18, if is_w7 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, pb2_x, row2_y, w_pub, 18, if is_w7 { COLOR_CYAN } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "8. Deep Space (CC0)", pb2_x + 4, row2_y + 4, if is_w7 { COLOR_CYAN } else { COLOR_TEXT_MUTED }, None);

        // Linha 3: Videos Dinamicos / Motion (IDs 8..11)
        let row3_y = row2_y + 22;
        let w_mot = 100;
        let m0_x = win_x + 12;
        let m1_x = m0_x + w_mot + 6;
        let m2_x = m1_x + w_mot + 6;
        let m3_x = m2_x + w_mot + 6;

        let is_w8 = self.wallpaper_id == 8;
        fill_rect(buf, screen_w, screen_h, m0_x, row3_y, w_mot, 18, if is_w8 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, m0_x, row3_y, w_mot, 18, if is_w8 { COLOR_EMERALD } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "9. Matrix Rain*", m0_x + 4, row3_y + 4, if is_w8 { COLOR_EMERALD } else { COLOR_TEXT_MUTED }, None);

        let is_w9 = self.wallpaper_id == 9;
        fill_rect(buf, screen_w, screen_h, m1_x, row3_y, w_mot, 18, if is_w9 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, m1_x, row3_y, w_mot, 18, if is_w9 { COLOR_CYAN_NEON } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "10. Warp 3D*", m1_x + 6, row3_y + 4, if is_w9 { COLOR_CYAN_NEON } else { COLOR_TEXT_MUTED }, None);

        let is_w10 = self.wallpaper_id == 10;
        fill_rect(buf, screen_w, screen_h, m2_x, row3_y, w_mot, 18, if is_w10 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, m2_x, row3_y, w_mot, 18, if is_w10 { COLOR_PURPLE } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "11. Sun Pulse*", m2_x + 4, row3_y + 4, if is_w10 { COLOR_PURPLE } else { COLOR_TEXT_MUTED }, None);

        let is_w11 = self.wallpaper_id == 11;
        fill_rect(buf, screen_w, screen_h, m3_x, row3_y, w_mot, 18, if is_w11 { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, m3_x, row3_y, w_mot, 18, if is_w11 { COLOR_CYAN } else { COLOR_BORDER });
        draw_string(buf, screen_w, screen_h, "12. Aurora Flw*", m3_x + 4, row3_y + 4, if is_w11 { COLOR_CYAN } else { COLOR_TEXT_MUTED }, None);

        // Linha 4: Controles de Audio do Wallpaper
        let row4_y = row3_y + 22;
        let aud_btn_w = 230;
        fill_rect(buf, screen_w, screen_h, win_x + 12, row4_y, aud_btn_w, 18, if self.wallpaper_audio_enabled { COLOR_CARD_ALT } else { 0x0011_1827 });
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, row4_y, aud_btn_w, 18, if self.wallpaper_audio_enabled { COLOR_EMERALD } else { COLOR_BORDER });
        if self.wallpaper_audio_enabled {
            draw_string(buf, screen_w, screen_h, "[x] Audio do Wallpaper: ATIVO", win_x + 18, row4_y + 4, COLOR_EMERALD, None);
        } else {
            draw_string(buf, screen_w, screen_h, "[ ] Audio do Wallpaper: MUDO", win_x + 18, row4_y + 4, COLOR_TEXT_MUTED, None);
        }

        // Botoes de Volume: [ - ] 40% [ + ]
        let vol_x = win_x + 12 + aud_btn_w + 12;
        draw_string(buf, screen_w, screen_h, "Vol:", vol_x, row4_y + 4, COLOR_TEXT_PRI, Some(COLOR_CARD));

        let btn_down_x = vol_x + 36;
        fill_rect(buf, screen_w, screen_h, btn_down_x, row4_y, 22, 18, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, btn_down_x, row4_y, 22, 18, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "-", btn_down_x + 8, row4_y + 4, COLOR_WHITE, None);

        let vol_num_x = btn_down_x + 28;
        let mut v_str = [b'0'; 4];
        let v_tens = b'0' + (self.wallpaper_audio_volume / 10);
        let v_ones = b'0' + (self.wallpaper_audio_volume % 10);
        v_str[0] = v_tens;
        v_str[1] = v_ones;
        v_str[2] = b'%';
        v_str[3] = 0;
        let v_slice = core::str::from_utf8(&v_str[..3]).unwrap_or("40%");
        draw_string(buf, screen_w, screen_h, v_slice, vol_num_x, row4_y + 4, COLOR_CYAN, Some(COLOR_CARD));

        let btn_up_x = vol_num_x + 32;
        fill_rect(buf, screen_w, screen_h, btn_up_x, row4_y, 22, 18, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, btn_up_x, row4_y, 22, 18, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "+", btn_up_x + 7, row4_y + 4, COLOR_WHITE, None);

        // 3. Secao de Persistencia em Disco
        let pers_y = row4_y + 24;
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
        let hw_y = pers_y + 50;
        fill_rect(buf, screen_w, screen_h, win_x + 12, hw_y, pers_w, 46, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 12, hw_y, pers_w, 46, COLOR_BORDER);

        draw_string(buf, screen_w, screen_h, "Resolucao : 1024x768x32 bpp (Limine GOP)", win_x + 20, hw_y + 6, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Cursor    : Rastreamento Rapido < 5 us (DirtyRect)", win_x + 20, hw_y + 20, COLOR_EMERALD, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, "Storage   : Perfil JSA: ", win_x + 20, hw_y + 34, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        draw_string(buf, screen_w, screen_h, sys.storage_profile_name(), win_x + 160, hw_y + 34, COLOR_CYAN, Some(COLOR_CARD_ALT));

        // 5. Secao do Firewall SPI JinnWall
        let fw_y = hw_y + 52;
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
        let act_y = fw_y + 54;

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
