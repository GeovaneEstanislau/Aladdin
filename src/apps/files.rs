//! Gerenciador de Arquivos Aladdin (Navegação Multi-Volume e Quick Look)

use crate::compositor::*;
use crate::font::*;
use crate::contracts::vfs::VfsHost;

#[derive(Clone, Copy)]
pub struct CachedFileEntry {
    pub name: [u8; 16],
    pub name_len: usize,
    pub is_elf: bool,
    pub is_txt: bool,
}

impl CachedFileEntry {
    pub const fn empty() -> Self {
        Self {
            name: [0; 16],
            name_len: 0,
            is_elf: false,
            is_txt: false,
        }
    }
}

pub struct FileManagerApp {
    pub active_volume: u8, // 0 = /disk (FAT32, RW), 1 = /ext2 (Ext2, RO)
    pub file_cache: [CachedFileEntry; 6],
    pub file_cache_count: usize,
    pub file_cache_valid: bool,
    pub preview_active: bool,
    pub preview_filename: [u8; 16],
    pub preview_filename_len: usize,
    pub preview_content: [u8; 256],
    pub preview_content_len: usize,
}

impl FileManagerApp {
    pub const fn new() -> Self {
        Self {
            active_volume: 0,
            file_cache: [CachedFileEntry::empty(); 6],
            file_cache_count: 0,
            file_cache_valid: false,
            preview_active: false,
            preview_filename: [0; 16],
            preview_filename_len: 0,
            preview_content: [0; 256],
            preview_content_len: 0,
        }
    }

    pub fn refresh_cache(&mut self, vfs: &dyn VfsHost) {
        self.file_cache_count = 0;
        let mut count = 0;
        let cache_ptr = self.file_cache.as_mut_ptr();
        let target = if self.active_volume == 1 { "/ext2" } else { "/disk" };

        vfs.readdir(target, &mut |name, _is_dir, is_elf, is_txt| {
            if count < 6 {
                let n_len = name.len().min(15);
                unsafe {
                    let slot = &mut *cache_ptr.add(count);
                    slot.name[..n_len].copy_from_slice(&name.as_bytes()[..n_len]);
                    slot.name_len = n_len;
                    slot.is_elf = is_elf;
                    slot.is_txt = is_txt;
                }
                count += 1;
            }
        });

        self.file_cache_count = count;
        self.file_cache_valid = true;
    }

    pub fn open_preview(&mut self, name: &str, vfs: &dyn VfsHost) {
        let n_len = name.len().min(15);
        self.preview_filename[..n_len].copy_from_slice(&name.as_bytes()[..n_len]);
        self.preview_filename_len = n_len;

        // Monta caminho
        let mut path_buf = [0u8; 32];
        let pfx = if self.active_volume == 1 { "/ext2/" } else { "/disk/" };
        let pfx_b = pfx.as_bytes();
        path_buf[..pfx_b.len()].copy_from_slice(pfx_b);
        path_buf[pfx_b.len()..(pfx_b.len() + n_len)].copy_from_slice(&name.as_bytes()[..n_len]);
        let total_path_len = pfx_b.len() + n_len;

        if let Ok(path_str) = core::str::from_utf8(&path_buf[..total_path_len]) {
            if let Ok(read_bytes) = vfs.read_file(path_str, &mut self.preview_content) {
                self.preview_content_len = read_bytes.min(256);
                self.preview_active = true;
                return;
            }
        }
        self.preview_content_len = 0;
        self.preview_active = true;
    }

    pub fn close_preview(&mut self) {
        self.preview_active = false;
        self.preview_filename_len = 0;
        self.preview_content_len = 0;
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
        storage_profile: &str,
    ) {
        draw_string(buf, screen_w, screen_h, "--- Armazenamento Jinn / VFS ---", win_x + 12, win_y + 32, COLOR_CYAN, Some(COLOR_CARD));

        // Abas seletoras de volume: /disk (FAT32) e /ext2 (Ext2)
        let tab_d_w = 114;
        let tab_e_w = 114;
        let tab_d_x = win_x + 12;
        let tab_e_x = tab_d_x + tab_d_w + 8;
        let tab_y = win_y + 48;
        let tab_h = 18;

        let is_v0 = self.active_volume == 0;
        let bg_d = if is_v0 { 0x001E_293B } else { COLOR_CARD };
        let bd_d = if is_v0 { COLOR_CYAN } else { COLOR_BORDER };
        let fg_d = if is_v0 { COLOR_CYAN } else { COLOR_TEXT_MUTED };
        fill_rect(buf, screen_w, screen_h, tab_d_x, tab_y, tab_d_w, tab_h, bg_d);
        draw_rect_outline(buf, screen_w, screen_h, tab_d_x, tab_y, tab_d_w, tab_h, bd_d);
        draw_string(buf, screen_w, screen_h, "/disk (FAT32)", tab_d_x + 6, tab_y + 3, fg_d, Some(bg_d));

        let is_v1 = self.active_volume == 1;
        let bg_e = if is_v1 { 0x001E_293B } else { COLOR_CARD };
        let bd_e = if is_v1 { COLOR_CYAN } else { COLOR_BORDER };
        let fg_e = if is_v1 { COLOR_CYAN } else { COLOR_TEXT_MUTED };
        fill_rect(buf, screen_w, screen_h, tab_e_x, tab_y, tab_e_w, tab_h, bg_e);
        draw_rect_outline(buf, screen_w, screen_h, tab_e_x, tab_y, tab_e_w, tab_h, bd_e);
        draw_string(buf, screen_w, screen_h, "/ext2 (Ext2)", tab_e_x + 8, tab_y + 3, fg_e, Some(bg_e));

        if self.preview_active {
            // Painel Quick Look (File Preview)
            let p_x = win_x + 12;
            let p_y = win_y + 70;
            let p_w = win_w - 24;
            let p_h = 106;

            fill_rect(buf, screen_w, screen_h, p_x, p_y, p_w, p_h, 0x000F_172A);
            draw_rect_outline(buf, screen_w, screen_h, p_x, p_y, p_w, p_h, COLOR_BORDER_ACT);

            // Barra superior do preview
            fill_rect(buf, screen_w, screen_h, p_x + 1, p_y + 1, p_w - 2, 20, 0x001E_293B);
            let pfx_str = if self.active_volume == 1 { "Preview: /ext2/" } else { "Preview: /disk/" };
            draw_string(buf, screen_w, screen_h, pfx_str, p_x + 6, p_y + 3, COLOR_CYAN, Some(0x001E_293B));
            let fn_len = self.preview_filename_len.min(16);
            if let Ok(fn_str) = core::str::from_utf8(&self.preview_filename[..fn_len]) {
                draw_string(buf, screen_w, screen_h, fn_str, p_x + 126, p_y + 3, COLOR_WHITE, Some(0x001E_293B));
            }

            let btn_c_w = 76;
            let btn_c_x = p_x + p_w - btn_c_w - 4;

            // Botão Excluir no Preview (apenas no volume RW /disk)
            if self.active_volume == 0 {
                let btn_dp_w = 64;
                let btn_dp_x = btn_c_x - btn_dp_w - 4;
                fill_rect(buf, screen_w, screen_h, btn_dp_x, p_y + 2, btn_dp_w, 16, 0x002D_1215);
                draw_rect_outline(buf, screen_w, screen_h, btn_dp_x, p_y + 2, btn_dp_w, 16, COLOR_RUBY);
                draw_string(buf, screen_w, screen_h, "Excluir", btn_dp_x + 5, p_y + 3, 0x00F8_7171, Some(0x002D_1215));
            }

            // Botão Fechar Preview
            fill_rect(buf, screen_w, screen_h, btn_c_x, p_y + 2, btn_c_w, 16, COLOR_RUBY);
            draw_string(buf, screen_w, screen_h, "X Fechar", btn_c_x + 6, p_y + 3, COLOR_WHITE, Some(COLOR_RUBY));

            // Renderiza linhas de texto do preview
            let c_len = self.preview_content_len.min(256);
            let mut line_y = p_y + 24;
            let mut line_start = 0;
            let mut rendered_lines = 0;
            for i in 0..c_len {
                let b = self.preview_content[i];
                let is_break = b == b'\n' || (i - line_start) >= 46 || i == c_len - 1;
                if is_break {
                    let end_i = if b == b'\n' { i } else { i + 1 };
                    if end_i > line_start && rendered_lines < 4 {
                        if let Ok(line_str) = core::str::from_utf8(&self.preview_content[line_start..end_i]) {
                            draw_string(buf, screen_w, screen_h, line_str, p_x + 8, line_y, COLOR_TEXT_PRI, Some(0x000F_172A));
                            line_y += 18;
                            rendered_lines += 1;
                        }
                    }
                    line_start = i + 1;
                }
            }

            let mode_lbl = if self.active_volume == 1 { "Modo: Ext2 Read-Only | UTF-8" } else { "Modo: FAT32 Read-Write | UTF-8" };
            draw_string(buf, screen_w, screen_h, mode_lbl, win_x + 12, win_y + 182, COLOR_TEXT_MUTED, Some(COLOR_CARD));
        } else {
            // Card de listagem de arquivos
            let files_box_x = win_x + 12;
            let files_box_y = win_y + 70;
            let files_box_w = win_w - 24;
            let files_box_h = 104;
            fill_rect(buf, screen_w, screen_h, files_box_x, files_box_y, files_box_w, files_box_h, COLOR_CARD_ALT);
            draw_rect_outline(buf, screen_w, screen_h, files_box_x, files_box_y, files_box_w, files_box_h, COLOR_BORDER);

            let f_title = if self.active_volume == 1 { "Arquivos em /ext2 (RO):" } else { "Arquivos em /disk (RW):" };
            draw_string(buf, screen_w, screen_h, f_title, files_box_x + 8, files_box_y + 8, COLOR_CYAN, Some(COLOR_CARD_ALT));

            let btn_rf_w = 88;
            let btn_rf_x = files_box_x + files_box_w - btn_rf_w - 8;

            if self.active_volume == 0 {
                // Botão [+ Novo]
                let btn_new_w = 60;
                let btn_new_x = btn_rf_x - btn_new_w - 6;
                fill_rect(buf, screen_w, screen_h, btn_new_x, files_box_y + 6, btn_new_w, 18, 0x001E_293B);
                draw_rect_outline(buf, screen_w, screen_h, btn_new_x, files_box_y + 6, btn_new_w, 18, COLOR_EMERALD);
                draw_string(buf, screen_w, screen_h, "+ Novo", btn_new_x + 6, files_box_y + 8, COLOR_EMERALD, Some(0x001E_293B));
            } else {
                let ro_w = 90;
                let ro_x = btn_rf_x - ro_w - 6;
                draw_string(buf, screen_w, screen_h, "(Somente Leitura)", ro_x, files_box_y + 8, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
            }

            // Botão [Atualizar]
            fill_rect(buf, screen_w, screen_h, btn_rf_x, files_box_y + 6, btn_rf_w, 18, 0x001E_293B);
            draw_rect_outline(buf, screen_w, screen_h, btn_rf_x, files_box_y + 6, btn_rf_w, 18, COLOR_BORDER);
            draw_string(buf, screen_w, screen_h, "[Atualizar]", btn_rf_x + 6, files_box_y + 8, COLOR_TEXT_PRI, Some(0x001E_293B));

            if self.file_cache_count == 0 {
                draw_string(buf, screen_w, screen_h, "(Nenhum arquivo detectado neste volume)", files_box_x + 12, files_box_y + 40, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
            } else {
                for i in 0..self.file_cache_count.min(3) {
                    let entry = &self.file_cache[i];
                    let item_y = files_box_y + 32 + (i * 22);
                    let tag = if entry.is_elf { "[ELF] " } else if entry.is_txt { "[TXT] " } else { "[ARQ] " };
                    let tag_color = if entry.is_elf { COLOR_EMERALD } else { COLOR_CYAN };
                    draw_string(buf, screen_w, screen_h, tag, files_box_x + 8, item_y, tag_color, Some(COLOR_CARD_ALT));
                    if let Ok(name_str) = core::str::from_utf8(&entry.name[..entry.name_len]) {
                        draw_string(buf, screen_w, screen_h, name_str, files_box_x + 52, item_y, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
                    }

                    // Botões de ação do item
                    let btn_w = 64;
                    let btn_h = 16;
                    let btn_x = files_box_x + files_box_w - btn_w - 8;

                    if entry.is_elf {
                        fill_rect(buf, screen_w, screen_h, btn_x, item_y - 1, btn_w, btn_h, COLOR_BTN_BG);
                        draw_rect_outline(buf, screen_w, screen_h, btn_x, item_y - 1, btn_w, btn_h, 0x0093_C5FD);
                        draw_string(buf, screen_w, screen_h, "> Rodar", btn_x + 6, item_y, COLOR_WHITE, Some(COLOR_BTN_BG));
                    } else if entry.is_txt {
                        // Botão [Ler VFS] (Quick Look)
                        fill_rect(buf, screen_w, screen_h, btn_x, item_y - 1, btn_w, btn_h, 0x001E_293B);
                        draw_rect_outline(buf, screen_w, screen_h, btn_x, item_y - 1, btn_w, btn_h, COLOR_BORDER);
                        draw_string(buf, screen_w, screen_h, "Ler VFS", btn_x + 5, item_y, COLOR_CYAN, Some(0x001E_293B));

                        // Botão [Del] (apenas no volume RW /disk)
                        if self.active_volume == 0 {
                            let btn_del_w = 40;
                            let btn_del_x = btn_x - btn_del_w - 6;
                            fill_rect(buf, screen_w, screen_h, btn_del_x, item_y - 1, btn_del_w, btn_h, 0x002D_1215);
                            draw_rect_outline(buf, screen_w, screen_h, btn_del_x, item_y - 1, btn_del_w, btn_h, COLOR_RUBY);
                            draw_string(buf, screen_w, screen_h, "Del", btn_del_x + 8, item_y, 0x00F8_7171, Some(0x002D_1215));
                        }
                    }
                }
            }

            draw_string(buf, screen_w, screen_h, storage_profile, win_x + 12, win_y + 184, COLOR_CYAN, Some(COLOR_CARD));
            draw_string(buf, screen_w, screen_h, "Politica: LRU Cache Ativo | Dual Partition MBR", win_x + 12, win_y + 204, COLOR_TEXT_MUTED, Some(COLOR_CARD));
        }
    }
}
