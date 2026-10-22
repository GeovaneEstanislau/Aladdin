//! Aplicativo Monitor de Sistema e Telemetria

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::SystemStats;

pub struct SystemMonitorApp;

impl SystemMonitorApp {
    pub const fn new() -> Self {
        Self
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
        stats: &SystemStats,
    ) {
        draw_string(buf, screen_w, screen_h, "--- Recursos do Kernel (Ring 0) ---", win_x + 12, win_y + 36, COLOR_CYAN, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Tempo Ativo    : ", stats.uptime_secs, " s (100 Hz APIC)", win_x + 12, win_y + 58, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Tarefas Ativas : ", stats.task_count, " processos", win_x + 12, win_y + 78, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Preempcoes APIC: ", stats.sched_ticks as usize, " ticks", win_x + 12, win_y + 98, COLOR_TEXT_PRI, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Memoria RAM    : ", stats.used_ram_mb, " MB / ", win_x + 12, win_y + 118, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, " MB", win_x + 204, win_y + 118, COLOR_TEXT_PRI, Some(COLOR_CARD));

        // Barra gráfica de progresso de uso de RAM
        draw_progress_bar(
            buf,
            screen_w,
            screen_h,
            win_x + 12,
            win_y + 138,
            win_w.saturating_sub(24),
            16,
            stats.used_ram_mb,
            stats.total_ram_mb,
            COLOR_EMERALD,
            COLOR_CARD_ALT,
            COLOR_BORDER,
        );

        draw_string(buf, screen_w, screen_h, "--- Rede e Seguranca (JinnWall) ---", win_x + 12, win_y + 164, COLOR_CYAN, Some(COLOR_CARD));
        
        let ip0 = stats.net_ip[0] as usize;
        let ip1 = stats.net_ip[1] as usize;
        let ip2 = stats.net_ip[2] as usize;
        let ip3 = stats.net_ip[3] as usize;
        draw_label_and_num(buf, screen_w, screen_h, "Rede IP  : ", ip0, ".", win_x + 12, win_y + 184, COLOR_EMERALD, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", ip1, ".", win_x + 110, win_y + 184, COLOR_EMERALD, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", ip2, ".", win_x + 130, win_y + 184, COLOR_EMERALD, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", ip3, " (VirtIO)", win_x + 150, win_y + 184, COLOR_EMERALD, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Firewall : Conntrack ", stats.jinnwall_active_conns, " fluxos ativos", win_x + 12, win_y + 204, COLOR_CYAN, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Defesas  : Bloqueios ", stats.jinnwall_drops as usize, " pacotes DROP", win_x + 12, win_y + 224, COLOR_PURPLE, Some(COLOR_CARD));
        draw_string(buf, screen_w, screen_h, "Seguranca: Ring 0 (Kernel) / Ring 3 (User ELF64)", win_x + 12, win_y + 244, COLOR_TEXT_MUTED, Some(COLOR_CARD));
    }
}
