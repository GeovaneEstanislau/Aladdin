//! Conky-style Telemetry HUD Card
//! Exibe status detalhado do microkernel no canto superior esquerdo com efeito de vidro fumê

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::SystemStats;

pub const CONKY_WIDTH: usize = 240;
pub const CONKY_HEIGHT: usize = 192;

pub fn render_conky_hud(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    stats: &SystemStats,
    storage_profile: &str,
) {
    if screen_w < 600 || screen_h < 400 { return; }

    let x = 20;
    let y = 44;

    // 1. Sombra de elevação suave (Drop shadow)
    draw_drop_shadow_rounded(buf, screen_w, screen_h, x, y, CONKY_WIDTH, CONKY_HEIGHT, 10, 4, COLOR_SHADOW);

    // 2. Fundo em vidro acrílico translúcido escuro (Frosted Glass)
    fill_acrylic_glass(buf, screen_w, screen_h, x, y, CONKY_WIDTH, CONKY_HEIGHT, 10, 0x000B_0F19, 175);
    draw_rounded_outline(buf, screen_w, screen_h, x, y, CONKY_WIDTH, CONKY_HEIGHT, 10, 0x001E_3A5F);

    // 3. Cabeçalho: --- JINN MICROKERNEL --- JAHA 1.2
    let header_bg = 0x000B_0F19;
    draw_string(buf, screen_w, screen_h, "--- JINN MICROKERNEL ---", x + 10, y + 10, COLOR_EMERALD, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "JAHA 1.2", x + 172, y + 10, COLOR_TEXT_MUTED, Some(header_bg));
    draw_line(buf, screen_w, screen_h, (x + 10) as isize, (y + 22) as isize, (x + CONKY_WIDTH - 10) as isize, (y + 22) as isize, 0x001E_3A5F);

    // 4. Linhas de Telemetria Monospaçada
    let mut row_y = y + 28;
    let line_step = 13;

    // Arquitetura
    draw_string(buf, screen_w, screen_h, "Arquitetura :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "x86_64 no_std", x + 116, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Bootloader
    draw_string(buf, screen_w, screen_h, "Bootloader  :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "Limine Prot v2", x + 116, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Preempção
    draw_string(buf, screen_w, screen_h, "Preempcao   :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "PIT 100 Hz(10ms)", x + 116, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Ticks APIC
    draw_string(buf, screen_w, screen_h, "Ticks APIC  :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.current_tick as usize, "", x + 116, row_y, COLOR_CYAN_NEON, Some(header_bg));
    row_y += line_step;

    // SMP Cores
    draw_string(buf, screen_w, screen_h, "SMP Cores   :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "4 On (LAPIC 0..3)", x + 116, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Memória Usável
    draw_string(buf, screen_w, screen_h, "Mem Usavel  :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.used_ram_mb, "M/", x + 116, row_y, COLOR_TEXT_PRI, Some(header_bg));
    draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, "M", x + 176, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // Paginação
    draw_string(buf, screen_w, screen_h, "Paginacao   :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "4-lvl PML4 W^X", x + 116, row_y, COLOR_TEXT_PRI, Some(header_bg));
    row_y += line_step;

    // VFS Storage
    draw_string(buf, screen_w, screen_h, "VFS Storage :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, storage_profile, x + 116, row_y, COLOR_AMBER, Some(header_bg));
    row_y += line_step;

    // Segurança
    draw_string(buf, screen_w, screen_h, "Seguranca   :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "Ring 0/Ring 3", x + 116, row_y, COLOR_EMERALD, Some(header_bg));
    row_y += line_step + 4;

    // Rodapé de Domínio
    draw_line(buf, screen_w, screen_h, (x + 10) as isize, (row_y - 2) as isize, (x + CONKY_WIDTH - 10) as isize, (row_y - 2) as isize, 0x001E_3A5F);
    draw_string(buf, screen_w, screen_h, "Dominio :", x + 10, row_y, COLOR_TEXT_MUTED, Some(header_bg));
    draw_string(buf, screen_w, screen_h, "jinn-os.dev.br", x + 90, row_y, COLOR_CYAN_NEON, Some(header_bg));
}
