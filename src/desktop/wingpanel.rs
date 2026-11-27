//! Barra Superior de Status (Wingpanel) inspirada na interface oficial do portal Jinn OS
//! Vidro translúcido com indicador pulsante esmeralda, identidade do kernel e cápsula de telemetria

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{CapabilityMode, SystemStats};

pub const WINGPANEL_HEIGHT: usize = 28;
pub const EXIT_BTN_W: usize = 72;

#[inline]
pub fn exit_btn_x(screen_w: usize) -> usize {
    screen_w.saturating_sub(EXIT_BTN_W + 8)
}

#[inline]
pub fn right_capsule_bounds(screen_w: usize) -> Option<(usize, usize)> {
    let cap_w = 468;
    let ex = exit_btn_x(screen_w);
    if ex > cap_w + 430 {
        Some((ex.saturating_sub(cap_w + 8), cap_w))
    } else {
        None
    }
}

pub fn render_wingpanel(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    stats: &SystemStats,
    storage_profile: &str,
    mode: CapabilityMode,
    start_menu_open: bool,
) {
    // 1. Fundo acrílico translúcido do Wingpanel com desfoque real (Glassmorphism suave com alpha 140)
    fill_acrylic_surface(buf, screen_w, screen_h, 0, 0, screen_w, WINGPANEL_HEIGHT, 0, 0x000F_172A, 140, mode);
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
    draw_string_antialiased(buf, screen_w, screen_h, "Jinn OS", brand_x, brand_y, COLOR_WHITE, None);
    draw_string_antialiased(buf, screen_w, screen_h, "v0.0.3-alpha", brand_x + 64, brand_y, COLOR_CYAN_NEON, None);
    draw_string_antialiased(buf, screen_w, screen_h, "| x86_64 Bare-Metal |", brand_x + 168, brand_y, COLOR_TEXT_MUTED, None);
    draw_string_antialiased(buf, screen_w, screen_h, "GitHub", brand_x + 344, brand_y, COLOR_TEXT_SEC, None);

    // 3. Lado Direito: Cápsula de Telemetria e Monitor do Sistema (desacoplada do botão [X] Sair)
    if let Some((cap_x, right_capsule_w)) = right_capsule_bounds(screen_w) {
        let cap_y = 4;
        let cap_h = 20;
        let cap_max_x = cap_x + right_capsule_w.saturating_sub(6);

        // Fundo em cápsula translúcida
        draw_pill(buf, screen_w, screen_h, cap_x, cap_y, right_capsule_w, cap_h, COLOR_SURFACE_L1, COLOR_BORDER_MUTED);

        let sep = |buf: &mut [u32], cur_x: usize| -> usize {
            let sx = (cur_x + 4) as isize;
            draw_line(buf, screen_w, screen_h, sx, (cap_y + 4) as isize, sx, (cap_y + cap_h - 4) as isize, COLOR_BORDER_MUTED);
            cur_x + 9
        };

        // CPU Telemetry (Consumo real Zero-Idle: ciclos ativos vs HLT)
        let mut cur_x = cap_x + 8;
        draw_string_telemetry(buf, screen_w, screen_h, "CPU:", cur_x, cap_y + 4, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));
        cur_x += text_width("CPU:");
        let total_cycles = stats.cpu_active_cycles + stats.cpu_idle_cycles;
        let cpu_pct = if total_cycles > 0 {
            ((stats.cpu_active_cycles * 100) / total_cycles) as usize
        } else {
            0
        };
        cur_x = draw_label_and_num(buf, screen_w, screen_h, "", cpu_pct, "%", cur_x, cap_y + 4, COLOR_CYAN_NEON, Some(COLOR_SURFACE_L1));
        cur_x = sep(buf, cur_x);

        // RAM Telemetry
        draw_string_telemetry(buf, screen_w, screen_h, "RAM:", cur_x, cap_y + 4, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));
        cur_x += text_width("RAM:");
        cur_x = draw_label_and_num(buf, screen_w, screen_h, "", stats.used_ram_mb, "/", cur_x, cap_y + 4, COLOR_EMERALD, Some(COLOR_SURFACE_L1));
        cur_x = draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, "MB", cur_x, cap_y + 4, COLOR_EMERALD, Some(COLOR_SURFACE_L1));
        cur_x = sep(buf, cur_x);

        // SMP Cores & Storage
        let cores_lbl = "4 Cores";
        draw_string_telemetry(buf, screen_w, screen_h, cores_lbl, cur_x, cap_y + 4, COLOR_AMBER, Some(COLOR_SURFACE_L1));
        cur_x += text_width(cores_lbl);
        cur_x = sep(buf, cur_x);

        let up_reserve = 72;
        if cur_x + text_width(storage_profile) + 9 + up_reserve <= cap_max_x {
            draw_string_telemetry(buf, screen_w, screen_h, storage_profile, cur_x, cap_y + 4, COLOR_TEXT_SEC, Some(COLOR_SURFACE_L1));
            cur_x += text_width(storage_profile);
            cur_x = sep(buf, cur_x);
        }

        if cur_x + 48 <= cap_max_x {
            draw_label_and_num(buf, screen_w, screen_h, "Up:", stats.uptime_secs as usize, "s", cur_x, cap_y + 4, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));
        }
    }

    // 4. Botão Sair [X] no extremo direito com margem isolada
    let exit_x = exit_btn_x(screen_w);
    draw_pill(buf, screen_w, screen_h, exit_x, 4, EXIT_BTN_W, 20, COLOR_RUBY, 0x00F8_7171);
    draw_string_antialiased(buf, screen_w, screen_h, "[X] Sair", exit_x + 4, 8, COLOR_WHITE, Some(COLOR_RUBY));
}
