//! Aladdin Disk Manager — Gerenciador Gráfico de Discos, Partições e Volumes

use crate::compositor::*;
use crate::font::*;
use crate::contracts::storage::{StorageHost, DiskBus};

pub struct DiskManagerApp {
    pub selected_disk: usize,
    pub selected_volume: usize,
    pub confirm_dialog_open: bool,
    pub action_volume_id: Option<usize>,
    pub status_message: [u8; 64],
    pub status_message_len: usize,
    pub status_is_error: bool,
}

impl DiskManagerApp {
    pub const fn new() -> Self {
        Self {
            selected_disk: 0,
            selected_volume: 0,
            confirm_dialog_open: false,
            action_volume_id: None,
            status_message: [0; 64],
            status_message_len: 0,
            status_is_error: false,
        }
    }

    pub fn set_status(&mut self, msg: &str, is_error: bool) {
        let len = msg.len().min(63);
        self.status_message = [0; 64];
        self.status_message[..len].copy_from_slice(&msg.as_bytes()[..len]);
        self.status_message_len = len;
        self.status_is_error = is_error;
    }

    pub fn status_str(&self) -> &str {
        if self.status_message_len == 0 {
            "Pronto"
        } else {
            core::str::from_utf8(&self.status_message[..self.status_message_len]).unwrap_or("Pronto")
        }
    }

    pub fn render(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        wx: usize,
        wy: usize,
        ww: usize,
        wh: usize,
        storage: &dyn StorageHost,
    ) {
        // Fundo da janela do gerenciador de discos
        fill_rect(buf, screen_w, screen_h, wx, wy, ww, wh, COLOR_CARD);

        // Cabeçalho da aplicação
        let header_bg = 0x000F_172A;
        fill_rect(buf, screen_w, screen_h, wx, wy, ww, 26, header_bg);
        draw_string(
            buf, screen_w, screen_h,
            "Aladdin Disk Manager — Dispositivos de Armazenamento",
            wx + 10, wy + 6,
            COLOR_CYAN,
            Some(header_bg),
        );

        // Painel 1: Discos Físicos Detectados (Topo: wy + 32 a wy + 115)
        let disk_panel_y = wy + 32;
        fill_rect(buf, screen_w, screen_h, wx + 8, disk_panel_y, ww - 16, 80, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, wx + 8, disk_panel_y, ww - 16, 80, COLOR_BORDER);

        draw_string(buf, screen_w, screen_h, "Discos Fisicos Detectados:", wx + 14, disk_panel_y + 4, COLOR_EMERALD, Some(COLOR_CARD_ALT));

        let disk_count = storage.disk_count();
        if disk_count == 0 {
            draw_string(buf, screen_w, screen_h, "Nenhum disco detectado", wx + 14, disk_panel_y + 24, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        } else {
            for i in 0..disk_count.min(3) {
                if let Some(disk) = storage.get_disk(i) {
                    let dy = disk_panel_y + 22 + (i * 18);
                    let is_sel = self.selected_disk == i;
                    let row_bg = if is_sel { COLOR_CARD } else { COLOR_CARD_ALT };
                    fill_rect(buf, screen_w, screen_h, wx + 12, dy - 2, ww - 24, 16, row_bg);

                    let bus_tag = match disk.bus {
                        DiskBus::IdeAta => "[IDE]",
                        DiskBus::AhciSata => "[SATA]",
                        DiskBus::NvmePci => "[NVMe]",
                        DiskBus::VirtioBlk => "[VIRTIO]",
                        DiskBus::RamDisk => "[RAM]",
                        DiskBus::Unknown => "[RAW]",
                    };
                    draw_string(buf, screen_w, screen_h, bus_tag, wx + 14, dy, COLOR_AMBER, Some(row_bg));
                    draw_string(buf, screen_w, screen_h, disk.name_str(), wx + 70, dy, COLOR_TEXT_PRI, Some(row_bg));
                    draw_string(buf, screen_w, screen_h, disk.model_str(), wx + 180, dy, COLOR_TEXT_MUTED, Some(row_bg));

                    // Capacidade
                    let mut cap_buf = [0u8; 16];
                    let clen = format_num(disk.capacity_mb as usize, &mut cap_buf);
                    if let Ok(c_str) = core::str::from_utf8(&cap_buf[..clen]) {
                        draw_string(buf, screen_w, screen_h, c_str, wx + ww - 90, dy, COLOR_CYAN, Some(row_bg));
                        draw_string(buf, screen_w, screen_h, "MB", wx + ww - 40, dy, COLOR_TEXT_MUTED, Some(row_bg));
                    }
                }
            }
        }

        // Painel 2: Partições & Volumes (Centro: wy + 118 a wy + 215)
        let vol_panel_y = wy + 118;
        fill_rect(buf, screen_w, screen_h, wx + 8, vol_panel_y, ww - 16, 95, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, wx + 8, vol_panel_y, ww - 16, 95, COLOR_BORDER);

        draw_string(buf, screen_w, screen_h, "Volumes e Sistemas de Arquivos:", wx + 14, vol_panel_y + 4, COLOR_CYAN, Some(COLOR_CARD_ALT));

        let vol_count = storage.volume_count();
        if vol_count == 0 {
            draw_string(buf, screen_w, screen_h, "Nenhum volume mapeado", wx + 14, vol_panel_y + 24, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        } else {
            for v_idx in 0..vol_count.min(3) {
                if let Some(vol) = storage.get_volume(v_idx) {
                    let vy = vol_panel_y + 22 + (v_idx * 23);
                    let is_sel = self.selected_volume == v_idx;
                    let row_bg = if is_sel { COLOR_CARD } else { COLOR_CARD_ALT };
                    fill_rect(buf, screen_w, screen_h, wx + 12, vy - 2, ww - 24, 21, row_bg);

                    // Ponto de montagem ou Rótulo
                    draw_string(buf, screen_w, screen_h, vol.mount_point_str(), wx + 14, vy + 2, COLOR_TEXT_PRI, Some(row_bg));
                    draw_string(buf, screen_w, screen_h, vol.fs_type_str(), wx + 75, vy + 2, COLOR_AMBER, Some(row_bg));

                    // Badge de Raiz / Status
                    if vol.is_root {
                        fill_rect(buf, screen_w, screen_h, wx + 130, vy + 1, 64, 14, 0x0016_4E63);
                        draw_string(buf, screen_w, screen_h, "ROOT", wx + 145, vy + 3, COLOR_CYAN, Some(0x0016_4E63));
                    } else if vol.is_mounted {
                        fill_rect(buf, screen_w, screen_h, wx + 130, vy + 1, 64, 14, 0x0006_4E3B);
                        draw_string(buf, screen_w, screen_h, "MONTADO", wx + 135, vy + 3, COLOR_EMERALD, Some(0x0006_4E3B));
                    } else {
                        fill_rect(buf, screen_w, screen_h, wx + 130, vy + 1, 64, 14, 0x0033_4155);
                        draw_string(buf, screen_w, screen_h, "DESMONTADO", wx + 132, vy + 3, COLOR_TEXT_MUTED, Some(0x0033_4155));
                    }

                    // Barra visual de uso
                    let bar_x = wx + 205;
                    let bar_w = ww.saturating_sub(310);
                    let bar_y = vy + 5;
                    fill_rect(buf, screen_w, screen_h, bar_x, bar_y, bar_w, 8, 0x000F_172A);
                    let used_ratio = if vol.total_mb > 0 {
                        (vol.used_mb * bar_w as u64) / vol.total_mb
                    } else {
                        0
                    } as usize;
                    let fill_w = used_ratio.min(bar_w);
                    if fill_w > 0 {
                        fill_rect(buf, screen_w, screen_h, bar_x, bar_y, fill_w, 8, COLOR_EMERALD);
                    }
                    draw_rect_outline(buf, screen_w, screen_h, bar_x, bar_y, bar_w, 8, COLOR_BORDER);

                    // Tamanho
                    let mut sz_buf = [0u8; 16];
                    let sz_len = format_num(vol.total_mb as usize, &mut sz_buf);
                    if let Ok(sz_str) = core::str::from_utf8(&sz_buf[..sz_len]) {
                        draw_string(buf, screen_w, screen_h, sz_str, wx + ww - 85, vy + 2, COLOR_TEXT_MUTED, Some(row_bg));
                        draw_string(buf, screen_w, screen_h, "MB", wx + ww - 40, vy + 2, COLOR_TEXT_MUTED, Some(row_bg));
                    }
                }
            }
        }

        // Painel 3: Ações e Controles (wy + 220 a wy + 265)
        let act_y = wy + 222;
        let btn_mount_bg = 0x0006_5F46;
        fill_rect(buf, screen_w, screen_h, wx + 14, act_y, 88, 26, btn_mount_bg);
        draw_rect_outline(buf, screen_w, screen_h, wx + 14, act_y, 88, 26, COLOR_EMERALD);
        draw_string(buf, screen_w, screen_h, "Montar", wx + 34, act_y + 7, COLOR_TEXT_PRI, Some(btn_mount_bg));

        let btn_unmount_bg = 0x0085_4D0E;
        fill_rect(buf, screen_w, screen_h, wx + 112, act_y, 96, 26, btn_unmount_bg);
        draw_rect_outline(buf, screen_w, screen_h, wx + 112, act_y, 96, 26, COLOR_AMBER);
        draw_string(buf, screen_w, screen_h, "Desmontar", wx + 126, act_y + 7, COLOR_TEXT_PRI, Some(btn_unmount_bg));

        let btn_fmt_bg = 0x007F_1D1D;
        fill_rect(buf, screen_w, screen_h, wx + 218, act_y, 96, 26, btn_fmt_bg);
        draw_rect_outline(buf, screen_w, screen_h, wx + 218, act_y, 96, 26, COLOR_RUBY);
        draw_string(buf, screen_w, screen_h, "Formatar", wx + 236, act_y + 7, COLOR_TEXT_PRI, Some(btn_fmt_bg));

        // Barra de Status Inferior (wy + wh - 22)
        let sb_y = wy + wh - 22;
        fill_rect(buf, screen_w, screen_h, wx, sb_y, ww, 22, 0x000B_0F19);
        fill_rect(buf, screen_w, screen_h, wx, sb_y, ww, 1, COLOR_BORDER);
        let st_color = if self.status_is_error { COLOR_RUBY } else { COLOR_CYAN };
        draw_string(buf, screen_w, screen_h, "Status:", wx + 10, sb_y + 5, COLOR_TEXT_MUTED, Some(0x000B_0F19));
        draw_string(buf, screen_w, screen_h, self.status_str(), wx + 65, sb_y + 5, st_color, Some(0x000B_0F19));

        // Diálogo de Confirmação Explícita de Ação Destrutiva (Modal)
        if self.confirm_dialog_open {
            let dw = 360;
            let dh = 130;
            let dx = wx + (ww / 2).saturating_sub(dw / 2);
            let dy = wy + (wh / 2).saturating_sub(dh / 2);

            // Sombra e fundo escuro do modal
            fill_rect(buf, screen_w, screen_h, dx + 4, dy + 4, dw, dh, 0x0005_0710);
            fill_rect(buf, screen_w, screen_h, dx, dy, dw, dh, 0x001E_1B4B);
            draw_rect_outline(buf, screen_w, screen_h, dx, dy, dw, dh, COLOR_RUBY);

            draw_string(buf, screen_w, screen_h, "AVISO: ACAO DESTRUTIVA", dx + 12, dy + 12, COLOR_RUBY, Some(0x001E_1B4B));
            draw_string(
                buf, screen_w, screen_h,
                "Deseja formatar este volume?",
                dx + 12, dy + 36,
                COLOR_TEXT_PRI, Some(0x001E_1B4B),
            );
            draw_string(
                buf, screen_w, screen_h,
                "Todos os arquivos serao permanentemente apagados.",
                dx + 12, dy + 54,
                COLOR_AMBER, Some(0x001E_1B4B),
            );

            // Botão Confirmar
            let c_bg = COLOR_RUBY;
            fill_rect(buf, screen_w, screen_h, dx + 20, dy + 88, 140, 26, c_bg);
            draw_string(buf, screen_w, screen_h, "Confirmar Formatar", dx + 26, dy + 94, COLOR_TEXT_PRI, Some(c_bg));

            // Botão Cancelar
            let cn_bg = COLOR_CARD_ALT;
            fill_rect(buf, screen_w, screen_h, dx + 180, dy + 88, 140, 26, cn_bg);
            draw_string(buf, screen_w, screen_h, "Cancelar", dx + 225, dy + 94, COLOR_TEXT_PRI, Some(cn_bg));
        }
    }

    pub fn handle_click(
        &mut self,
        rel_x: usize,
        rel_y: usize,
        ww: usize,
        wh: usize,
        storage: &mut dyn StorageHost,
    ) {
        // Se o modal de confirmação estiver aberto, intercepta os cliques
        if self.confirm_dialog_open {
            let dw = 360;
            let dh = 130;
            let dx = (ww / 2).saturating_sub(dw / 2);
            let dy = (wh / 2).saturating_sub(dh / 2);

            // Botão Confirmar Formatar: dx + 20 .. dx + 160, dy + 88 .. dy + 114
            if rel_x >= dx + 20 && rel_x <= dx + 160 && rel_y >= dy + 88 && rel_y <= dy + 114 {
                if let Some(vol_id) = self.action_volume_id {
                    match storage.format_volume(vol_id, "FAT32") {
                        Ok(()) => self.set_status("Volume formatado com sucesso (FAT32).", false),
                        Err(err) => self.set_status(err, true),
                    }
                }
                self.confirm_dialog_open = false;
                self.action_volume_id = None;
                return;
            }

            // Botão Cancelar: dx + 180 .. dx + 320, dy + 88 .. dy + 114
            if rel_x >= dx + 180 && rel_x <= dx + 320 && rel_y >= dy + 88 && rel_y <= dy + 114 {
                self.set_status("Formatacao cancelada pelo usuario.", false);
                self.confirm_dialog_open = false;
                self.action_volume_id = None;
                return;
            }

            return; // Bloqueia cliques fora do modal enquanto ele estiver aberto
        }

        // Seleção de disco (wy + 54 a wy + 112)
        if rel_y >= 54 && rel_y < 112 {
            let idx = (rel_y - 54) / 18;
            if idx < storage.disk_count() {
                self.selected_disk = idx;
                self.set_status("Disco selecionado.", false);
            }
            return;
        }

        // Seleção de volume (wy + 140 a wy + 210)
        if rel_y >= 140 && rel_y < 210 {
            let v_idx = (rel_y - 140) / 23;
            if v_idx < storage.volume_count() {
                self.selected_volume = v_idx;
                self.set_status("Volume selecionado.", false);
            }
            return;
        }

        // Ações de botões (wy + 222 a wy + 248)
        if rel_y >= 222 && rel_y <= 248 {
            let target_vol_id = self.selected_volume;

            // Botão Montar (wx + 14 a wx + 102)
            if rel_x >= 14 && rel_x <= 102 {
                match storage.mount_volume(target_vol_id, "/mnt") {
                    Ok(()) => self.set_status("Volume montado com sucesso.", false),
                    Err(e) => self.set_status(e, true),
                }
                return;
            }

            // Botão Desmontar (wx + 112 a wx + 208)
            if rel_x >= 112 && rel_x <= 208 {
                if let Some(vol) = storage.get_volume(target_vol_id) {
                    if vol.is_root {
                        self.set_status("Bloqueado: Particao raiz ativa '/' nao pode ser desmontada.", true);
                        return;
                    }
                }
                match storage.unmount_volume(target_vol_id) {
                    Ok(()) => self.set_status("Volume desmontado com sucesso.", false),
                    Err(e) => self.set_status(e, true),
                }
                return;
            }

            // Botão Formatar (wx + 218 a wx + 314)
            if rel_x >= 218 && rel_x <= 314 {
                if let Some(vol) = storage.get_volume(target_vol_id) {
                    if vol.is_root {
                        self.set_status("Bloqueado: Proibido formatar a particao raiz do sistema.", true);
                        return;
                    }
                }
                self.action_volume_id = Some(target_vol_id);
                self.confirm_dialog_open = true;
                return;
            }
        }
    }
}
