//! Aplicativo Monitor de Sistema e Telemetria

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::{JinnWallProfile, SystemStats};

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

        draw_label_and_num(buf, screen_w, screen_h, "Tempo Ativo    : ", stats.uptime_secs, " s (100 Hz APIC)", win_x + 12, win_y + 56, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Tarefas Ativas : ", stats.task_count, " processos", win_x + 12, win_y + 74, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Preempcoes APIC: ", stats.sched_ticks as usize, " ticks", win_x + 12, win_y + 92, COLOR_TEXT_PRI, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Memoria RAM    : ", stats.used_ram_mb, " MB / ", win_x + 12, win_y + 110, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, " MB", win_x + 204, win_y + 110, COLOR_TEXT_PRI, Some(COLOR_CARD));

        // Barra grafica de progresso de uso de RAM
        draw_progress_bar(
            buf,
            screen_w,
            screen_h,
            win_x + 12,
            win_y + 128,
            win_w.saturating_sub(24),
            14,
            stats.used_ram_mb,
            stats.total_ram_mb,
            COLOR_EMERALD,
            COLOR_CARD_ALT,
            COLOR_BORDER,
        );

        draw_string(buf, screen_w, screen_h, "--- Rede e Seguranca (JinnWall) ---", win_x + 12, win_y + 152, COLOR_CYAN, Some(COLOR_CARD));

        let ip0 = stats.net_ip[0] as usize;
        let ip1 = stats.net_ip[1] as usize;
        let ip2 = stats.net_ip[2] as usize;
        let ip3 = stats.net_ip[3] as usize;

        let has_ip = stats.net_ip != [0, 0, 0, 0];
        if has_ip {
            draw_label_and_num(buf, screen_w, screen_h, "Rede IP  : ", ip0, ".", win_x + 12, win_y + 170, COLOR_EMERALD, Some(COLOR_CARD));
            draw_label_and_num(buf, screen_w, screen_h, "", ip1, ".", win_x + 110, win_y + 170, COLOR_EMERALD, Some(COLOR_CARD));
            draw_label_and_num(buf, screen_w, screen_h, "", ip2, ".", win_x + 130, win_y + 170, COLOR_EMERALD, Some(COLOR_CARD));
            draw_label_and_num(buf, screen_w, screen_h, "", ip3, " (", win_x + 150, win_y + 170, COLOR_EMERALD, Some(COLOR_CARD));
            draw_string(buf, screen_w, screen_h, stats.net_adapter, win_x + 195, win_y + 170, COLOR_EMERALD, Some(COLOR_CARD));
            draw_string(buf, screen_w, screen_h, ")", win_x + 195 + stats.net_adapter.len() * 8, win_y + 170, COLOR_EMERALD, Some(COLOR_CARD));
        } else {
            draw_string(buf, screen_w, screen_h, "Rede IP  : Inativa / Desconectada", win_x + 12, win_y + 170, COLOR_TEXT_MUTED, Some(COLOR_CARD));
        }

        draw_string(buf, screen_w, screen_h, "Status   : ", win_x + 12, win_y + 188, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_string(buf, screen_w, screen_h, stats.net_state_desc, win_x + 100, win_y + 188, if has_ip { COLOR_CYAN } else { COLOR_AMBER }, Some(COLOR_CARD));

        let is_sec = stats.jinnwall_profile == JinnWallProfile::PublicSecure;
        draw_string(buf, screen_w, screen_h, "Firewall : Perfil ", win_x + 12, win_y + 206, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_string(buf, screen_w, screen_h, stats.jinnwall_profile.name(), win_x + 140, win_y + 206, if is_sec { COLOR_EMERALD } else { COLOR_CYAN }, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Conntrack: ", stats.jinnwall_active_conns, " fluxos ativos", win_x + 12, win_y + 224, COLOR_CYAN, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Defesas  : Bloqueios ", stats.jinnwall_drops as usize, " pacotes DROP", win_x + 12, win_y + 242, COLOR_PURPLE, Some(COLOR_CARD));
        draw_string(buf, screen_w, screen_h, "Seguranca: Ring 0 (Kernel) / Ring 3 (User ELF64)", win_x + 12, win_y + 260, COLOR_TEXT_MUTED, Some(COLOR_CARD));
    }
}
