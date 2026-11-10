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

        draw_label_and_num(buf, screen_w, screen_h, "Tempo Ativo    : ", stats.uptime_secs, " s (100 Hz APIC)", win_x + 12, win_y + 54, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Tarefas Ativas : ", stats.task_count, " processos", win_x + 12, win_y + 70, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Preempcoes APIC: ", stats.sched_ticks as usize, " ticks", win_x + 12, win_y + 86, COLOR_TEXT_PRI, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Memoria RAM    : ", stats.used_ram_mb, " MB / ", win_x + 12, win_y + 102, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", stats.total_ram_mb, " MB", win_x + 204, win_y + 102, COLOR_TEXT_PRI, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Frames Fisicos : ", stats.physical_frames_used, " / ", win_x + 12, win_y + 118, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", stats.physical_frames_total, " frames (4 KiB)", win_x + 204, win_y + 118, COLOR_TEXT_PRI, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Heap Dinamico  : ", stats.dynamic_pages_allocated.saturating_sub(stats.dynamic_pages_freed), " ativas (", win_x + 12, win_y + 134, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", stats.dynamic_pages_freed, " liberadas)", win_x + 204, win_y + 134, COLOR_TEXT_PRI, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "VFS / Arquivos : ", stats.vfs_open_fds, " FDs abertos | ", win_x + 12, win_y + 150, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "", stats.vfs_mounted_volumes, " vols", win_x + 230, win_y + 150, COLOR_TEXT_PRI, Some(COLOR_CARD));

        // Barra grafica de progresso de uso de RAM
        draw_progress_bar(
            buf,
            screen_w,
            screen_h,
            win_x + 12,
            win_y + 168,
            win_w.saturating_sub(24),
            12,
            stats.used_ram_mb,
            stats.total_ram_mb,
            COLOR_EMERALD,
            COLOR_CARD_ALT,
            COLOR_BORDER,
        );

        draw_string(buf, screen_w, screen_h, "--- Rede e Seguranca (JinnWall) ---", win_x + 12, win_y + 188, COLOR_CYAN, Some(COLOR_CARD));

        let has_ip = stats.net_ip != [0, 0, 0, 0];
        if has_ip {
            let mut line_buf = [0u8; 80];
            let mut pos = 0;
            let prefix = b"Rede IP  : ";
            line_buf[pos..pos + prefix.len()].copy_from_slice(prefix);
            pos += prefix.len();

            for (idx, &octet) in stats.net_ip.iter().enumerate() {
                if idx > 0 {
                    line_buf[pos] = b'.';
                    pos += 1;
                }
                if octet >= 100 {
                    line_buf[pos] = b'0' + (octet / 100);
                    pos += 1;
                    line_buf[pos] = b'0' + ((octet / 10) % 10);
                    pos += 1;
                    line_buf[pos] = b'0' + (octet % 10);
                    pos += 1;
                } else if octet >= 10 {
                    line_buf[pos] = b'0' + (octet / 10);
                    pos += 1;
                    line_buf[pos] = b'0' + (octet % 10);
                    pos += 1;
                } else {
                    line_buf[pos] = b'0' + octet;
                    pos += 1;
                }
            }

            let mid = b" (";
            line_buf[pos..pos + mid.len()].copy_from_slice(mid);
            pos += mid.len();

            let ad_bytes = stats.net_adapter.as_bytes();
            let ad_len = ad_bytes.len().min(line_buf.len() - pos - 2);
            line_buf[pos..pos + ad_len].copy_from_slice(&ad_bytes[..ad_len]);
            pos += ad_len;

            line_buf[pos] = b')';
            pos += 1;

            if let Ok(line_str) = core::str::from_utf8(&line_buf[..pos]) {
                draw_string(buf, screen_w, screen_h, line_str, win_x + 12, win_y + 206, COLOR_EMERALD, Some(COLOR_CARD));
            }
        } else {
            draw_string(buf, screen_w, screen_h, "Rede IP  : Inativa / Desconectada", win_x + 12, win_y + 206, COLOR_TEXT_MUTED, Some(COLOR_CARD));
        }

        draw_string(buf, screen_w, screen_h, "Status   : ", win_x + 12, win_y + 224, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_string(buf, screen_w, screen_h, stats.net_state_desc, win_x + 100, win_y + 224, if has_ip { COLOR_CYAN } else { COLOR_AMBER }, Some(COLOR_CARD));

        let is_sec = stats.jinnwall_profile == JinnWallProfile::PublicSecure;
        draw_string(buf, screen_w, screen_h, "Firewall : Perfil ", win_x + 12, win_y + 242, COLOR_TEXT_PRI, Some(COLOR_CARD));
        draw_string(buf, screen_w, screen_h, stats.jinnwall_profile.name(), win_x + 140, win_y + 242, if is_sec { COLOR_EMERALD } else { COLOR_CYAN }, Some(COLOR_CARD));

        draw_label_and_num(buf, screen_w, screen_h, "Conntrack: ", stats.jinnwall_active_conns, " fluxos ativos", win_x + 12, win_y + 260, COLOR_CYAN, Some(COLOR_CARD));
        draw_label_and_num(buf, screen_w, screen_h, "Defesas  : Bloqueios ", stats.jinnwall_drops as usize, " pacotes DROP", win_x + 12, win_y + 278, COLOR_PURPLE, Some(COLOR_CARD));
        draw_string(buf, screen_w, screen_h, "Seguranca: Ring 0 (Kernel) / Ring 3 (User ELF64)", win_x + 12, win_y + 296, COLOR_TEXT_MUTED, Some(COLOR_CARD));
    }
}
