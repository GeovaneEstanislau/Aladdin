//! Conky-style Telemetry HUD Card
//! Exibe status detalhado do microkernel no canto superior esquerdo com efeito de vidro fumê
//! e tipografia antialiased de alta definição com badges e suporte a acentuação portuguesa.

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::SystemStats;

pub const CONKY_WIDTH: usize = 256;
pub const CONKY_HEIGHT: usize = 234;

pub fn render_conky_hud(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    stats: &SystemStats,
    storage_profile: &str,
) {
    if screen_w < 600 || screen_h < 400 { return; }

    let x = 16;
    let y = 38;

    // 1. Sombra de elevação suave (Drop shadow difusa)
    draw_drop_shadow_rounded(buf, screen_w, screen_h, x, y, CONKY_WIDTH, CONKY_HEIGHT, 12, 8, COLOR_SHADOW);

    // 2. Fundo em vidro acrílico translúcido escuro (Frosted Glass suave com alpha 148)
    fill_acrylic_glass(buf, screen_w, screen_h, x, y, CONKY_WIDTH, CONKY_HEIGHT, 12, 0x000B_0F19, 148);
    draw_rounded_outline_alpha(buf, screen_w, screen_h, x, y, CONKY_WIDTH, CONKY_HEIGHT, 12, 0x0038_BDF8, 55);

    // 3. Cabeçalho: [JINN MICROKERNEL] [JAHA 1.2]
    let header_bg = 0x000B_0F19;
    draw_string_antialiased(buf, screen_w, screen_h, "• JINN MICROKERNEL", x + 12, y + 10, COLOR_EMERALD, Some(header_bg));
    draw_telemetry_badge(buf, screen_w, screen_h, "JAHA 1.2", x + 180, y + 8, COLOR_CYAN_NEON, 0x0013_2338, 0x0038_BDF8);
    draw_line(buf, screen_w, screen_h, (x + 12) as isize, (y + 26) as isize, (x + CONKY_WIDTH - 12) as isize, (y + 26) as isize, 0x001E_3A5F);

    // 4. Linhas de Telemetria Monospaçada com Acentuação Nativa
    let mut row_y = y + 32;
    let line_step = 14;

    // Arquitetura
    draw_string_telemetry(buf, screen_w, screen_h, "Arquitetura :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, "x86_64 no_std", x + 122, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Bootloader
    draw_string_telemetry(buf, screen_w, screen_h, "Bootloader  :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, "Limine Prot v2", x + 122, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Preempção
    draw_string_telemetry(buf, screen_w, screen_h, "Preempção   :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, "PIT 100 Hz(10ms)", x + 122, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // CPU Zero-Idle
    let total_c = stats.cpu_active_cycles + stats.cpu_idle_cycles;
    let cpu_pct = if total_c > 0 { ((stats.cpu_active_cycles * 100) / total_c) as usize } else { 0 };
    draw_string_telemetry(buf, screen_w, screen_h, "CPU Zero-Idle:", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", cpu_pct, "% (HLT)", x + 122, row_y, COLOR_EMERALD, Some(header_bg));
    row_y += line_step;

    // Syscalls Total
    draw_string_telemetry(buf, screen_w, screen_h, "Syscalls    :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.syscalls_total as usize, " ops", x + 122, row_y, COLOR_CYAN_NEON, Some(header_bg));
    row_y += line_step;

    // Ticks APIC
    draw_string_telemetry(buf, screen_w, screen_h, "Ticks APIC  :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.current_tick as usize, "", x + 122, row_y, COLOR_CYAN_NEON, Some(header_bg));
    row_y += line_step;

    // SMP Cores
    draw_string_telemetry(buf, screen_w, screen_h, "SMP Cores   :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, "4 On (LAPIC 0..3)", x + 122, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Memória Usável
    draw_string_telemetry(buf, screen_w, screen_h, "Memória     :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.used_ram_mb, "M/", x + 122, row_y, COLOR_TEXT_PRI, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, "M", x + 184, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Paginação
    draw_string_telemetry(buf, screen_w, screen_h, "Paginação   :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, "4-lvl PML4 W^X", x + 122, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // VFS Storage
    draw_string_telemetry(buf, screen_w, screen_h, "VFS Storage :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, storage_profile, x + 122, row_y, COLOR_AMBER, Some(header_bg));
    row_y += line_step;

    // Segurança
    draw_string_telemetry(buf, screen_w, screen_h, "Segurança   :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, "Ring 0 / Ring 3", x + 122, row_y, COLOR_EMERALD, Some(header_bg));
    row_y += line_step + 4;

    // Rodapé de Domínio
    draw_line(buf, screen_w, screen_h, (x + 12) as isize, (row_y - 2) as isize, (x + CONKY_WIDTH - 12) as isize, (row_y - 2) as isize, 0x001E_3A5F);
    draw_string_telemetry(buf, screen_w, screen_h, "Domínio :", x + 12, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string_telemetry(buf, screen_w, screen_h, "jinn-os.dev.br", x + 96, row_y, COLOR_CYAN_NEON, Some(header_bg));
}
