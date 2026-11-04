//! Centro de Controle Rápido (Quick Settings Popover) do Aladdin Desktop
//!
//! Fornece acesso instantâneo e fluído a modos gráficos, perfis de segurança,
//! telemetria do kernel e controles de energia.

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost};

pub const QUICK_SETTINGS_W: usize = 310;
pub const QUICK_SETTINGS_H: usize = 274;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickSettingsAction {
    None,
    SetMode(CapabilityMode),
    ToggleFirewall,
    CycleStorage,
    Reboot,
    ExitShell,
}

pub struct QuickSettingsPopover;

impl QuickSettingsPopover {
    pub const fn new() -> Self {
        Self
    }

    pub fn render(
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        popover_x: usize,
        popover_y: usize,
        mouse_x: usize,
        mouse_y: usize,
        mode: CapabilityMode,
        sys: &dyn SystemHost,
    ) {
        let stats = sys.stats();

        // 1. Sombra projetada para elevação fluída
        draw_drop_shadow(buf, screen_w, screen_h, popover_x, popover_y, QUICK_SETTINGS_W, QUICK_SETTINGS_H, 4, COLOR_SHADOW);

        // 2. Fundo acrílico escuro
        fill_rect(buf, screen_w, screen_h, popover_x, popover_y, QUICK_SETTINGS_W, QUICK_SETTINGS_H, COLOR_SURFACE_L1);
        draw_rect_outline(buf, screen_w, screen_h, popover_x, popover_y, QUICK_SETTINGS_W, QUICK_SETTINGS_H, COLOR_CYAN_NEON);

        // 3. Cabeçalho com linha sutil
        fill_rect(buf, screen_w, screen_h, popover_x + 1, popover_y + 1, QUICK_SETTINGS_W - 2, 28, COLOR_SURFACE_L0);
        draw_char(buf, screen_w, screen_h, b'*', popover_x + 12, popover_y + 8, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L0));
        draw_string(buf, screen_w, screen_h, "Centro de Controle Aladdin", popover_x + 24, popover_y + 8, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L0));
        draw_line(buf, screen_w, screen_h, popover_x as isize, (popover_y + 29) as isize, (popover_x + QUICK_SETTINGS_W) as isize, (popover_y + 29) as isize, COLOR_BORDER_SUBTLE);

        // 4. Seção 1: Modos de Desempenho Visual (FX)
        draw_string(buf, screen_w, screen_h, "Composicao e Efeitos Visuais:", popover_x + 12, popover_y + 36, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));

        let btn_w = 88;
        let btn_h = 22;
        let b0_x = popover_x + 12;
        let b1_x = b0_x + btn_w + 6;
        let b2_x = b1_x + btn_w + 6;
        let btn_y = popover_y + 52;

        let modes = [
            (b0_x, CapabilityMode::Basic, "1. Eco", COLOR_EMERALD),
            (b1_x, CapabilityMode::Balanced, "2. Bal", COLOR_CYAN_NEON),
            (b2_x, CapabilityMode::Advanced, "3. Glow", COLOR_PURPLE),
        ];

        for &(bx, m, label, ac) in &modes {
            let is_sel = mode == m;
            let is_hover = mouse_x >= bx && mouse_x < bx + btn_w && mouse_y >= btn_y && mouse_y < btn_y + btn_h;
            let bg = if is_sel { COLOR_SURFACE_L3 } else if is_hover { COLOR_SURFACE_HOVER } else { COLOR_SURFACE_L2 };
            let bd = if is_sel { ac } else if is_hover { COLOR_BORDER_ACT } else { COLOR_BORDER };
            let fg = if is_sel { ac } else if is_hover { COLOR_WHITE } else { COLOR_TEXT_SEC };

            fill_rect(buf, screen_w, screen_h, bx, btn_y, btn_w, btn_h, bg);
            draw_rect_outline(buf, screen_w, screen_h, bx, btn_y, btn_w, btn_h, bd);
            draw_string(buf, screen_w, screen_h, label, bx + 12, btn_y + 5, fg, Some(bg));
        }

        // 5. Seção 2: Telemetria Rápida de Memória
        let ram_card_y = popover_y + 84;
        fill_rect(buf, screen_w, screen_h, popover_x + 12, ram_card_y, QUICK_SETTINGS_W - 24, 48, COLOR_SURFACE_L2);
        draw_rect_outline(buf, screen_w, screen_h, popover_x + 12, ram_card_y, QUICK_SETTINGS_W - 24, 48, COLOR_BORDER_SUBTLE);

        draw_label_and_num(buf, screen_w, screen_h, "RAM em Uso: ", stats.used_ram_mb, " MB / ", popover_x + 20, ram_card_y + 6, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L2));
        draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, " MB", popover_x + 175, ram_card_y + 6, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L2));

        draw_progress_bar(
            buf,
            screen_w,
            screen_h,
            popover_x + 20,
            ram_card_y + 24,
            QUICK_SETTINGS_W - 40,
            12,
            stats.used_ram_mb,
            stats.total_ram_mb,
            COLOR_EMERALD,
            COLOR_SURFACE_L3,
            COLOR_BORDER,
        );

        // 6. Seção 3: JinnWall Firewall e Storage
        let sec_card_y = popover_y + 140;
        fill_rect(buf, screen_w, screen_h, popover_x + 12, sec_card_y, QUICK_SETTINGS_W - 24, 60, COLOR_SURFACE_L2);
        draw_rect_outline(buf, screen_w, screen_h, popover_x + 12, sec_card_y, QUICK_SETTINGS_W - 24, 60, COLOR_BORDER_SUBTLE);

        let prof = sys.jinnwall_profile();
        let is_sec = prof == JinnWallProfile::PublicSecure;
        draw_string(buf, screen_w, screen_h, "Firewall SPI: ", popover_x + 20, sec_card_y + 8, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L2));
        draw_string(buf, screen_w, screen_h, if is_sec { "[ PublicSecure ]" } else { "[ LocalTrusted ]" }, popover_x + 130, sec_card_y + 8, if is_sec { COLOR_EMERALD } else { COLOR_CYAN_NEON }, Some(COLOR_SURFACE_L2));

        // Botão Alternar Firewall
        let fw_btn_y = sec_card_y + 30;
        let is_fw_hover = mouse_x >= popover_x + 20 && mouse_x < popover_x + 150 && mouse_y >= fw_btn_y && mouse_y < fw_btn_y + 22;
        let fw_bg = if is_fw_hover { COLOR_BLUE_HOVER } else { COLOR_BLUE_PRIMARY };
        fill_rect(buf, screen_w, screen_h, popover_x + 20, fw_btn_y, 130, 22, fw_bg);
        draw_rect_outline(buf, screen_w, screen_h, popover_x + 20, fw_btn_y, 130, 22, COLOR_CYAN_NEON);
        draw_string(buf, screen_w, screen_h, "Mudar Perfil", popover_x + 32, fw_btn_y + 5, COLOR_WHITE, Some(fw_bg));

        // Botão Ciclar Storage JSA
        let is_jsa_hover = mouse_x >= popover_x + 160 && mouse_x < popover_x + 290 && mouse_y >= fw_btn_y && mouse_y < fw_btn_y + 22;
        let jsa_bg = if is_jsa_hover { COLOR_SURFACE_HOVER } else { COLOR_SURFACE_L3 };
        fill_rect(buf, screen_w, screen_h, popover_x + 160, fw_btn_y, 130, 22, jsa_bg);
        draw_rect_outline(buf, screen_w, screen_h, popover_x + 160, fw_btn_y, 130, 22, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Ciclar JSA", popover_x + 185, fw_btn_y + 5, COLOR_TEXT_PRI, Some(jsa_bg));

        // 7. Seção 4: Botões de Ação do Sistema
        let act_btn_y = popover_y + 210;
        let half_w = (QUICK_SETTINGS_W - 30) / 2;

        // Botão Reiniciar
        let r_btn_x = popover_x + 12;
        let is_r_hover = mouse_x >= r_btn_x && mouse_x < r_btn_x + half_w && mouse_y >= act_btn_y && mouse_y < act_btn_y + 26;
        let r_bg = if is_r_hover { 0x00D9_7706 } else { COLOR_SURFACE_L2 };
        fill_rect(buf, screen_w, screen_h, r_btn_x, act_btn_y, half_w, 26, r_bg);
        draw_rect_outline(buf, screen_w, screen_h, r_btn_x, act_btn_y, half_w, 26, COLOR_AMBER);
        draw_string(buf, screen_w, screen_h, "[R] Reiniciar", r_btn_x + 16, act_btn_y + 7, if is_r_hover { COLOR_WHITE } else { COLOR_AMBER }, Some(r_bg));

        // Botão Sair para Shell
        let s_btn_x = popover_x + 18 + half_w;
        let is_s_hover = mouse_x >= s_btn_x && mouse_x < s_btn_x + half_w && mouse_y >= act_btn_y && mouse_y < act_btn_y + 26;
        let s_bg = if is_s_hover { 0x00DC_2626 } else { COLOR_RUBY };
        fill_rect(buf, screen_w, screen_h, s_btn_x, act_btn_y, half_w, 26, s_bg);
        draw_rect_outline(buf, screen_w, screen_h, s_btn_x, act_btn_y, half_w, 26, 0x00F8_7171);
        draw_string(buf, screen_w, screen_h, "[X] Sair Shell", s_btn_x + 16, act_btn_y + 7, COLOR_WHITE, Some(s_bg));

        // Rodapé com versão
        draw_string(buf, screen_w, screen_h, "Aladdin Desktop 7.0 — Fluido e Leve", popover_x + 16, popover_y + 248, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));
    }

    pub fn handle_click(
        popover_x: usize,
        popover_y: usize,
        mx: usize,
        my: usize,
    ) -> QuickSettingsAction {
        if mx < popover_x || mx >= popover_x + QUICK_SETTINGS_W || my < popover_y || my >= popover_y + QUICK_SETTINGS_H {
            return QuickSettingsAction::None;
        }

        // 1. Modos FX
        let btn_w = 88;
        let btn_h = 22;
        let b0_x = popover_x + 12;
        let b1_x = b0_x + btn_w + 6;
        let b2_x = b1_x + btn_w + 6;
        let btn_y = popover_y + 52;

        if my >= btn_y && my < btn_y + btn_h {
            if mx >= b0_x && mx < b0_x + btn_w {
                return QuickSettingsAction::SetMode(CapabilityMode::Basic);
            }
            if mx >= b1_x && mx < b1_x + btn_w {
                return QuickSettingsAction::SetMode(CapabilityMode::Balanced);
            }
            if mx >= b2_x && mx < b2_x + btn_w {
                return QuickSettingsAction::SetMode(CapabilityMode::Advanced);
            }
        }

        // 2. Firewall e Storage
        let sec_card_y = popover_y + 140;
        let fw_btn_y = sec_card_y + 30;
        if my >= fw_btn_y && my < fw_btn_y + 22 {
            if mx >= popover_x + 20 && mx < popover_x + 150 {
                return QuickSettingsAction::ToggleFirewall;
            }
            if mx >= popover_x + 160 && mx < popover_x + 290 {
                return QuickSettingsAction::CycleStorage;
            }
        }

        // 3. Ações do Sistema
        let act_btn_y = popover_y + 210;
        let half_w = (QUICK_SETTINGS_W - 30) / 2;
        if my >= act_btn_y && my < act_btn_y + 26 {
            let r_btn_x = popover_x + 12;
            if mx >= r_btn_x && mx < r_btn_x + half_w {
                return QuickSettingsAction::Reboot;
            }
            let s_btn_x = popover_x + 18 + half_w;
            if mx >= s_btn_x && mx < s_btn_x + half_w {
                return QuickSettingsAction::ExitShell;
            }
        }

        QuickSettingsAction::None
    }
}
