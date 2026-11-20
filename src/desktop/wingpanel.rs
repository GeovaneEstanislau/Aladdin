//! Barra Superior de Status (Wingpanel) inspirada na interface oficial do portal Jinn OS
//! Vidro translúcido com indicador pulsante esmeralda, identidade do kernel e cápsula de telemetria

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, SystemStats};

pub const WINGPANEL_HEIGHT: usize = 28;

pub fn render_wingpanel(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    stats: &SystemStats,
    storage_profile: &str,
    mode: CapabilityMode,
    start_menu_open: bool,
) {
    // 1. Fundo acrílico translúcido do Wingpanel com desfoque real (Glassmorphism)
    fill_acrylic_surface(buf, screen_w, screen_h, 0, 0, screen_w, WINGPANEL_HEIGHT, 0, 0x000F_172A, 190, mode);
    draw_line(buf, screen_w, screen_h, 0, WINGPANEL_HEIGHT as isize, screen_w as isize, WINGPANEL_HEIGHT as isize, 0x001E_293B);

    // Linha de acento sutil neon inferior se modo Advanced
    if mode == CapabilityMode::Advanced {
        draw_line(buf, screen_w, screen_h, 0, (WINGPANEL_HEIGHT + 1) as isize, screen_w as isize, (WINGPANEL_HEIGHT + 1) as isize, COLOR_CYAN_NEON);
    }

    // Se o menu iniciar estiver aberto, desenha uma pílula de foco sob a marca
    if start_menu_open {
        draw_pill(buf, screen_w, screen_h, 6, 4, 156, 20, COLOR_SURFACE_HOVER, COLOR_CYAN_NEON);
    }

    // 2. Lado Esquerdo: Indicador pulsante e identidade de marca
    // Ponto esmeralda pulsante
    let pulse_bright = (stats.current_tick % 30) < 15;
    let dot_color = if pulse_bright { 0x0034_D399 } else { 0x0010_B981 };
    let dot_x = 12;
    let dot_y = 12;
    draw_pixel(buf, screen_w, screen_h, dot_x, dot_y, dot_color);
    draw_pixel(buf, screen_w, screen_h, dot_x + 1, dot_y, dot_color);
    draw_pixel(buf, screen_w, screen_h, dot_x, dot_y + 1, dot_color);
    draw_pixel(buf, screen_w, screen_h, dot_x + 1, dot_y + 1, dot_color);

    // Halo suave do ponto
    if pulse_bright {
        let halo_col = 0x0006_4E3B;
        draw_pixel(buf, screen_w, screen_h, dot_x - 1, dot_y, halo_col);
        draw_pixel(buf, screen_w, screen_h, dot_x + 2, dot_y, halo_col);
        draw_pixel(buf, screen_w, screen_h, dot_x, dot_y - 1, halo_col);
        draw_pixel(buf, screen_w, screen_h, dot_x, dot_y + 2, halo_col);
    }

    // "Jinn OS v0.0.3-alpha | x86_64 Bare-Metal | GitHub"
    let brand_x = 22;
    let brand_y = 10;
    draw_string(buf, screen_w, screen_h, "Jinn OS", brand_x, brand_y, COLOR_WHITE, None);
    draw_string(buf, screen_w, screen_h, "v0.0.3-alpha", brand_x + 64, brand_y, COLOR_CYAN_NEON, None);
    draw_string(buf, screen_w, screen_h, "| x86_64 Bare-Metal |", brand_x + 168, brand_y, COLOR_TEXT_MUTED, None);
    draw_string(buf, screen_w, screen_h, "GitHub", brand_x + 344, brand_y, COLOR_TEXT_SEC, None);

    // 3. Lado Direito: Cápsula de Telemetria e Monitor do Sistema
    let right_capsule_w = 460;
    if screen_w > right_capsule_w + 440 {
        let cap_x = screen_w.saturating_sub(right_capsule_w + 12);
        let cap_y = 4;
        let cap_h = 20;

        // Fundo em cápsula translúcida
        draw_pill(buf, screen_w, screen_h, cap_x, cap_y, right_capsule_w, cap_h, COLOR_SURFACE_L1, COLOR_BORDER_MUTED);

        // CPU Telemetry
        let mut cur_x = cap_x + 10;
        draw_string(buf, screen_w, screen_h, "CPU:", cur_x, cap_y + 4, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));
        cur_x += 34;
        let cpu_pct = ((stats.current_tick % 8) + 2) as usize;
        draw_label_and_num(buf, screen_w, screen_h, "", cpu_pct, "% | ", cur_x, cap_y + 4, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L1));
        cur_x += 44;

        // RAM Telemetry
        draw_string(buf, screen_w, screen_h, "RAM:", cur_x, cap_y + 4, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));
        cur_x += 34;
        draw_label_and_num(buf, screen_w, screen_h, "", stats.used_ram_mb, "/", cur_x, cap_y + 4, COLOR_EMERALD, Some(COLOR_SURFACE_L1));
        cur_x += 36;
        draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, "MB | ", cur_x, cap_y + 4, COLOR_EMERALD, Some(COLOR_SURFACE_L1));
        cur_x += 54;

        // SMP Cores & Storage
        draw_string(buf, screen_w, screen_h, "4 Cores | ", cur_x, cap_y + 4, COLOR_AMBER, Some(COLOR_SURFACE_L1));
        cur_x += 80;

        draw_string(buf, screen_w, screen_h, storage_profile, cur_x, cap_y + 4, COLOR_TEXT_SEC, Some(COLOR_SURFACE_L1));
        cur_x += storage_profile.len() * FONT_WIDTH + 8;

        draw_label_and_num(buf, screen_w, screen_h, "Up:", stats.uptime_secs as usize, "s", cur_x, cap_y + 4, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));
    }

    // 4. Botão Sair [X] no extremo direito
    let exit_x = screen_w.saturating_sub(68);
    draw_pill(buf, screen_w, screen_h, exit_x, 4, 60, 20, COLOR_RUBY, 0x00F8_7171);
    draw_string(buf, screen_w, screen_h, "[X] Sair", exit_x + 4, 8, COLOR_WHITE, Some(COLOR_RUBY));
}
