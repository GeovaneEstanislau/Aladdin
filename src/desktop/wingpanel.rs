//! Barra Superior de Status (Wingpanel) inspirada no elementary OS 7

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
    // Fundo da barra superior
    fill_rect(buf, screen_w, screen_h, 0, 0, screen_w, WINGPANEL_HEIGHT, COLOR_TOPPANEL);
    draw_line(buf, screen_w, screen_h, 0, WINGPANEL_HEIGHT as isize, screen_w as isize, WINGPANEL_HEIGHT as isize, COLOR_BORDER);

    // 1. Launcher Pill [# Aladdin]
    let launcher_bg = if start_menu_open { COLOR_CARD_ALT } else { COLOR_TOPPANEL };
    let launcher_bd = if start_menu_open { COLOR_CYAN } else { COLOR_BORDER };
    fill_rect(buf, screen_w, screen_h, 8, 3, 114, 20, launcher_bg);
    draw_rect_outline(buf, screen_w, screen_h, 8, 3, 114, 20, launcher_bd);
    draw_string(buf, screen_w, screen_h, "[#] Aladdin", 16, 5, COLOR_CYAN, Some(launcher_bg));

    // 2. Center Pill: Uptime & APIC Timer
    let center_x = (screen_w / 2).saturating_sub(72);
    fill_rect(buf, screen_w, screen_h, center_x, 3, 144, 20, COLOR_CARD_ALT);
    draw_rect_outline(buf, screen_w, screen_h, center_x, 3, 144, 20, COLOR_BORDER);
    draw_label_and_num(buf, screen_w, screen_h, "Up: ", stats.uptime_secs, "s | 100Hz", center_x + 8, 5, COLOR_CYAN, Some(COLOR_CARD_ALT));

    // 3. Right Pill 1: RAM Telemetry
    let ram_x = screen_w.saturating_sub(424);
    fill_rect(buf, screen_w, screen_h, ram_x, 3, 120, 20, COLOR_CARD_ALT);
    draw_rect_outline(buf, screen_w, screen_h, ram_x, 3, 120, 20, COLOR_BORDER);
    draw_label_and_num(buf, screen_w, screen_h, "RAM: ", stats.used_ram_mb, "/", ram_x + 8, 5, COLOR_EMERALD, Some(COLOR_CARD_ALT));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, "M", ram_x + 72, 5, COLOR_EMERALD, Some(COLOR_CARD_ALT));

    // 4. Right Pill 2: JSA Storage Profile
    let jsa_x = screen_w.saturating_sub(298);
    fill_rect(buf, screen_w, screen_h, jsa_x, 3, 124, 20, COLOR_CARD_ALT);
    draw_rect_outline(buf, screen_w, screen_h, jsa_x, 3, 124, 20, COLOR_BORDER);
    draw_string(buf, screen_w, screen_h, storage_profile, jsa_x + 8, 5, COLOR_CYAN, Some(COLOR_CARD_ALT));

    // 5. Right Pill 3: FX Capability Mode
    let fx_x = screen_w.saturating_sub(168);
    let (mode_name, mode_fg) = match mode {
        CapabilityMode::Basic => ("FX: Eco", COLOR_TEXT_MUTED),
        CapabilityMode::Balanced => ("FX: Bal", COLOR_CYAN),
        CapabilityMode::Advanced => ("FX: Glow", COLOR_PURPLE),
    };
    fill_rect(buf, screen_w, screen_h, fx_x, 3, 84, 20, COLOR_CARD_ALT);
    draw_rect_outline(buf, screen_w, screen_h, fx_x, 3, 84, 20, COLOR_BORDER);
    draw_string(buf, screen_w, screen_h, mode_name, fx_x + 8, 5, mode_fg, Some(COLOR_CARD_ALT));

    // 6. Right Pill 4: Sair Button [X]
    let exit_x = screen_w.saturating_sub(78);
    fill_rect(buf, screen_w, screen_h, exit_x, 3, 70, 20, COLOR_RUBY);
    draw_rect_outline(buf, screen_w, screen_h, exit_x, 3, 70, 20, 0x00F8_7171);
    draw_string(buf, screen_w, screen_h, "[X] Sair", exit_x + 6, 5, COLOR_WHITE, Some(COLOR_RUBY));
}
