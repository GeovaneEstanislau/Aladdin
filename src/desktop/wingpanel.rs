//! Barra Superior de Status (Wingpanel) inspirada no elementary OS 7 com acentos Garuda Dr460nized

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, SystemStats};

pub const WINGPANEL_HEIGHT: usize = 26;

pub fn render_wingpanel(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    stats: &SystemStats,
    storage_profile: &str,
    mode: CapabilityMode,
    start_menu_open: bool,
) {
    // 1. Fundo translúcido do Wingpanel
    fill_rect(buf, screen_w, screen_h, 0, 0, screen_w, WINGPANEL_HEIGHT, COLOR_TOPPANEL);
    draw_line(buf, screen_w, screen_h, 0, WINGPANEL_HEIGHT as isize, screen_w as isize, WINGPANEL_HEIGHT as isize, COLOR_BORDER);

    // Linha de acento neon inferior no modo Advanced (Garuda Dr460nized)
    if mode == CapabilityMode::Advanced {
        draw_line(buf, screen_w, screen_h, 0, (WINGPANEL_HEIGHT + 1) as isize, screen_w as isize, (WINGPANEL_HEIGHT + 1) as isize, COLOR_CYAN_NEON);
    }

    // 2. Launcher Pill [Aladdin]
    let launcher_bg = if start_menu_open { COLOR_SURFACE_HOVER } else { COLOR_SURFACE_L1 };
    let launcher_bd = if start_menu_open { COLOR_CYAN_NEON } else { COLOR_BORDER_MUTED };
    draw_pill(buf, screen_w, screen_h, 8, 3, 92, 20, launcher_bg, launcher_bd);
    draw_string(buf, screen_w, screen_h, "Aladdin", 26, 5, COLOR_TEXT_PRI, Some(launcher_bg));

    // 3. Center Pill: Relógio / Uptime & APIC Timer (Proporção Áurea elementary OS)
    let center_x = (screen_w / 2).saturating_sub(76);
    draw_pill(buf, screen_w, screen_h, center_x, 3, 152, 20, COLOR_SURFACE_L1, COLOR_BORDER_MUTED);
    draw_label_and_num(buf, screen_w, screen_h, "Up: ", stats.uptime_secs, "s | 100Hz", center_x + 8, 5, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L1));

    // 4. Right Pill 1: RAM Telemetry
    let ram_x = screen_w.saturating_sub(424);
    draw_pill(buf, screen_w, screen_h, ram_x, 3, 120, 20, COLOR_SURFACE_L1, COLOR_BORDER_MUTED);
    draw_label_and_num(buf, screen_w, screen_h, "RAM: ", stats.used_ram_mb, "/", ram_x + 8, 5, COLOR_EMERALD, Some(COLOR_SURFACE_L1));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, "M", ram_x + 72, 5, COLOR_EMERALD, Some(COLOR_SURFACE_L1));

    // 5. Right Pill 2: JSA Storage Profile
    let jsa_x = screen_w.saturating_sub(298);
    draw_pill(buf, screen_w, screen_h, jsa_x, 3, 124, 20, COLOR_SURFACE_L1, COLOR_BORDER_MUTED);
    draw_string(buf, screen_w, screen_h, storage_profile, jsa_x + 8, 5, COLOR_TEXT_SEC, Some(COLOR_SURFACE_L1));

    // 6. Right Pill 3: FX Capability Mode
    let fx_x = screen_w.saturating_sub(168);
    let (mode_name, mode_fg) = match mode {
        CapabilityMode::Basic => ("FX: Eco", COLOR_TEXT_MUTED),
        CapabilityMode::Balanced => ("FX: Bal", COLOR_CYAN_NEON),
        CapabilityMode::Advanced => ("FX: Glow", COLOR_PURPLE),
    };
    draw_pill(buf, screen_w, screen_h, fx_x, 3, 84, 20, COLOR_SURFACE_L1, COLOR_BORDER_MUTED);
    draw_string(buf, screen_w, screen_h, mode_name, fx_x + 8, 5, mode_fg, Some(COLOR_SURFACE_L1));

    // 7. Right Pill 4: Sair Button [X]
    let exit_x = screen_w.saturating_sub(78);
    draw_pill(buf, screen_w, screen_h, exit_x, 3, 70, 20, COLOR_RUBY, 0x00F8_7171);
    draw_string(buf, screen_w, screen_h, "[X] Sair", exit_x + 6, 5, COLOR_WHITE, Some(COLOR_RUBY));
}
