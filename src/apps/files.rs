//! Gerenciador de Arquivos Aladdin (Aladdin File Manager 0.1.0)
//!
//! Interface moderna e minimalista para navegação em sistemas de arquivos, gerenciamento de
//! múltiplos volumes (FAT32, Ext2), operações completas de arquivos (criar, renomear, copiar,
//! excluir), inspeção de propriedades, Quick Look e execução de binários ELF.

use crate::compositor::*;
use crate::font::*;
use crate::contracts::vfs::VfsHost;
use crate::contracts::storage::StorageHost;
use crate::contracts::process::ProcessHost;

pub const MAX_FILES: usize = 16;
pub const MAX_PATH_BUF: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FileDialogKind {
    None,
    ConfirmDelete,
    NewFolder,
    NewFile,
    Rename,
    Preview,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileSortOrder {
    NameAsc,
    Type,
}

#[derive(Clone, Copy)]
pub struct CachedFileEntry {
    pub name: [u8; 32],
    pub name_len: usize,
    pub is_dir: bool,
    pub is_elf: bool,
    pub is_txt: bool,
    pub size: usize,
}

impl CachedFileEntry {
    pub const fn empty() -> Self {
        Self {
            name: [0; 32],
            name_len: 0,
            is_dir: false,
            is_elf: false,
            is_txt: false,
            size: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        if self.name_len == 0 {
            ""
        } else {
            core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
        }
    }
}

pub struct FileManagerApp {
    pub active_volume: usize, // 0 = /disk (FAT32, RW), 1 = /ext2 (Ext2, RO)
    pub current_path: [u8; MAX_PATH_BUF],
    pub current_path_len: usize,
    pub file_cache: [CachedFileEntry; MAX_FILES],
    pub file_cache_count: usize,
    pub file_cache_valid: bool,
    pub selected_index: Option<usize>,
    pub sort_order: FileSortOrder,

    // Diálogos e Interações Modais
    pub dialog_kind: FileDialogKind,
    pub input_buffer: [u8; 32],
    pub input_len: usize,
    pub status_message: [u8; 64],
    pub status_len: usize,
    pub status_is_error: bool,

    // Quick Look Preview
    pub preview_filename: [u8; 32],
    pub preview_filename_len: usize,
    pub preview_content: [u8; 256],
    pub preview_content_len: usize,
}

impl FileManagerApp {
    pub const fn new() -> Self {
        let mut app = Self {
            active_volume: 0,
            current_path: [0; MAX_PATH_BUF],
            current_path_len: 5,
            file_cache: [CachedFileEntry::empty(); MAX_FILES],
            file_cache_count: 0,
            file_cache_valid: false,
            selected_index: None,
            sort_order: FileSortOrder::NameAsc,
            dialog_kind: FileDialogKind::None,
            input_buffer: [0; 32],
            input_len: 0,
            status_message: [0; 64],
            status_len: 0,
            status_is_error: false,
            preview_filename: [0; 32],
            preview_filename_len: 0,
            preview_content: [0; 256],
            preview_content_len: 0,
        };
        app.current_path[0] = b'/';
        app.current_path[1] = b'd';
        app.current_path[2] = b'i';
        app.current_path[3] = b's';
        app.current_path[4] = b'k';
        app
    }

    pub fn current_path_str(&self) -> &str {
        if self.current_path_len == 0 {
            "/"
        } else {
            core::str::from_utf8(&self.current_path[..self.current_path_len]).unwrap_or("/")
        }
    }

    pub fn set_status(&mut self, msg: &str, is_err: bool) {
        let len = msg.len().min(63);
        self.status_message = [0; 64];
        self.status_message[..len].copy_from_slice(&msg.as_bytes()[..len]);
        self.status_len = len;
        self.status_is_error = is_err;
    }

    pub fn status_str(&self) -> &str {
        if self.status_len == 0 {
            "Pronto"
        } else {
            core::str::from_utf8(&self.status_message[..self.status_len]).unwrap_or("Pronto")
        }
    }

    pub fn is_read_only(&self) -> bool {
        self.active_volume == 1 || self.current_path_str().starts_with("/ext2")
    }

    pub fn refresh_cache(&mut self, vfs: &dyn VfsHost) {
        self.file_cache_count = 0;
        self.selected_index = None;
        let mut count = 0;
        let cache_ptr = self.file_cache.as_mut_ptr();
        let target = self.current_path_str();

        vfs.readdir(target, &mut |name, is_dir, is_elf, is_txt| {
            if count < MAX_FILES && name != "." && name != ".." {
                let n_len = name.len().min(31);
                unsafe {
                    let slot = &mut *cache_ptr.add(count);
                    slot.name = [0; 32];
                    slot.name[..n_len].copy_from_slice(&name.as_bytes()[..n_len]);
                    slot.name_len = n_len;
                    slot.is_dir = is_dir;
                    slot.is_elf = is_elf;
                    slot.is_txt = is_txt;
                    slot.size = 0;
                }
                count += 1;
            }
        });

        self.file_cache_count = count;
        self.file_cache_valid = true;
    }

    pub fn select_volume(&mut self, vol_idx: usize, vfs: &dyn VfsHost) {
        self.active_volume = vol_idx;
        self.close_dialog();
        self.selected_index = None;

        let root_path = if vol_idx == 1 { "/ext2" } else { "/disk" };
        let b = root_path.as_bytes();
        self.current_path = [0; MAX_PATH_BUF];
        self.current_path[..b.len()].copy_from_slice(b);
        self.current_path_len = b.len();

        self.refresh_cache(vfs);
        self.set_status(if vol_idx == 1 { "Volume /ext2 selecionado (Ext2, Somente Leitura)" } else { "Volume /disk selecionado (FAT32, Leitura/Escrita)" }, false);
    }

    pub fn navigate_up(&mut self, vfs: &dyn VfsHost) {
        let cur = self.current_path_str();
        if cur == "/disk" || cur == "/ext2" || cur == "/" {
            return;
        }

        if let Some(last_slash) = cur.rfind('/') {
            let new_len = if last_slash == 0 { 1 } else { last_slash };
            self.current_path_len = new_len;
            self.refresh_cache(vfs);
            self.set_status("Navegou para o diretorio pai", false);
        }
    }

    pub fn navigate_into(&mut self, dirname: &str, vfs: &dyn VfsHost) {
        let cur_len = self.current_path_len;
        let d_bytes = dirname.as_bytes();
        if cur_len + 1 + d_bytes.len() >= MAX_PATH_BUF {
            self.set_status("Caminho excede o tamanho maximo suportado", true);
            return;
        }

        if self.current_path[cur_len - 1] != b'/' {
            self.current_path[cur_len] = b'/';
            self.current_path[(cur_len + 1)..(cur_len + 1 + d_bytes.len())].copy_from_slice(d_bytes);
            self.current_path_len = cur_len + 1 + d_bytes.len();
        } else {
            self.current_path[cur_len..(cur_len + d_bytes.len())].copy_from_slice(d_bytes);
            self.current_path_len = cur_len + d_bytes.len();
        }

        self.refresh_cache(vfs);
        self.set_status("Diretorio aberto", false);
    }

    pub fn create_folder(&mut self, name: &str, vfs: &dyn VfsHost) -> Result<(), &'static str> {
        if self.is_read_only() {
            self.set_status("[Erro] Volume somente-leitura. Criacao negada.", true);
            return Err("Volume somente-leitura");
        }

        let clean = name.trim();
        if clean.is_empty() || clean.len() > 11 {
            self.set_status("[Erro] Nome de pasta invalido (maximo 11 caracteres 8.3)", true);
            return Err("Nome invalido");
        }

        let mut path_buf = [0u8; MAX_PATH_BUF];
        let cur = self.current_path_str();
        let cur_b = cur.as_bytes();
        path_buf[..cur_b.len()].copy_from_slice(cur_b);
        let mut plen = cur_b.len();
        if path_buf[plen - 1] != b'/' {
            path_buf[plen] = b'/';
            plen += 1;
        }
        path_buf[plen..plen + clean.len()].copy_from_slice(clean.as_bytes());
        plen += clean.len();

        let path_str = core::str::from_utf8(&path_buf[..plen]).map_err(|_| "Erro UTF-8")?;
        vfs.create_dir(path_str).map_err(|_| "Falha ao criar pasta no VFS")?;

        self.refresh_cache(vfs);
        self.set_status("[OK] Pasta criada com sucesso no VFS", false);
        Ok(())
    }

    pub fn create_file(&mut self, name: &str, vfs: &dyn VfsHost) -> Result<(), &'static str> {
        if self.is_read_only() {
            self.set_status("[Erro] Volume somente-leitura. Criacao negada.", true);
            return Err("Volume somente-leitura");
        }

        let clean = name.trim();
        if clean.is_empty() || clean.len() > 12 {
            self.set_status("[Erro] Nome de arquivo invalido (maximo 12 caracteres 8.3)", true);
            return Err("Nome invalido");
        }

        let mut path_buf = [0u8; MAX_PATH_BUF];
        let cur = self.current_path_str();
        let cur_b = cur.as_bytes();
        path_buf[..cur_b.len()].copy_from_slice(cur_b);
        let mut plen = cur_b.len();
        if path_buf[plen - 1] != b'/' {
            path_buf[plen] = b'/';
            plen += 1;
        }
        path_buf[plen..plen + clean.len()].copy_from_slice(clean.as_bytes());
        plen += clean.len();

        let path_str = core::str::from_utf8(&path_buf[..plen]).map_err(|_| "Erro UTF-8")?;
        vfs.create_file(path_str).map_err(|_| "Falha ao criar arquivo no VFS")?;

        self.refresh_cache(vfs);
        self.set_status("[OK] Arquivo criado com sucesso no VFS", false);
        Ok(())
    }

    pub fn copy_selected(&mut self, vfs: &dyn VfsHost) -> Result<(), &'static str> {
        if self.is_read_only() {
            self.set_status("[Erro] Volume somente-leitura. Copia negada.", true);
            return Err("Volume somente-leitura");
        }

        let sel_idx = self.selected_index.ok_or("Nenhum arquivo selecionado para copia")?;
        if sel_idx >= self.file_cache_count {
            return Err("Selecao invalida");
        }

        let entry = self.file_cache[sel_idx];
        if entry.is_dir {
            self.set_status("[Erro] Copia de diretorios inteiros nao suportada", true);
            return Err("Diretorio nao suportado");
        }

        let name = entry.name_str();
        let cur = self.current_path_str();

        let mut src_buf = [0u8; MAX_PATH_BUF];
        let mut src_len = cur.len();
        src_buf[..src_len].copy_from_slice(cur.as_bytes());
        if src_buf[src_len - 1] != b'/' {
            src_buf[src_len] = b'/';
            src_len += 1;
        }
        src_buf[src_len..src_len + name.len()].copy_from_slice(name.as_bytes());
        src_len += name.len();
        let src_path = core::str::from_utf8(&src_buf[..src_len]).map_err(|_| "Erro UTF-8")?;

        let mut dst_buf = [0u8; MAX_PATH_BUF];
        let mut dst_len = cur.len();
        dst_buf[..dst_len].copy_from_slice(cur.as_bytes());
        if dst_buf[dst_len - 1] != b'/' {
            dst_buf[dst_len] = b'/';
            dst_len += 1;
        }
        let prefix = b"C_";
        dst_buf[dst_len..dst_len + prefix.len()].copy_from_slice(prefix);
        dst_len += prefix.len();
        let copy_name_len = name.len().min(8);
        dst_buf[dst_len..dst_len + copy_name_len].copy_from_slice(&name.as_bytes()[..copy_name_len]);
        dst_len += copy_name_len;
        let dst_path = core::str::from_utf8(&dst_buf[..dst_len]).map_err(|_| "Erro UTF-8")?;

        vfs.copy_file(src_path, dst_path).map_err(|_| "Falha ao copiar arquivo no VFS")?;

        self.refresh_cache(vfs);
        self.set_status("[OK] Copia criada com sucesso no VFS", false);
        Ok(())
    }

    pub fn rename_selected(&mut self, new_name: &str, vfs: &dyn VfsHost) -> Result<(), &'static str> {
        if self.is_read_only() {
            self.set_status("[Erro] Volume somente-leitura. Renomeacao negada.", true);
            return Err("Volume somente-leitura");
        }

        let sel_idx = self.selected_index.ok_or("Nenhum arquivo selecionado")?;
        if sel_idx >= self.file_cache_count {
            return Err("Selecao invalida");
        }

        let clean = new_name.trim();
        if clean.is_empty() || clean.len() > 12 {
            self.set_status("[Erro] Novo nome invalido", true);
            return Err("Nome invalido");
        }

        let old_name = self.file_cache[sel_idx].name_str();
        let cur = self.current_path_str();

        let mut old_buf = [0u8; MAX_PATH_BUF];
        let mut old_len = cur.len();
        old_buf[..old_len].copy_from_slice(cur.as_bytes());
        if old_buf[old_len - 1] != b'/' { old_buf[old_len] = b'/'; old_len += 1; }
        old_buf[old_len..old_len + old_name.len()].copy_from_slice(old_name.as_bytes());
        old_len += old_name.len();
        let old_path = core::str::from_utf8(&old_buf[..old_len]).map_err(|_| "Erro UTF-8")?;

        let mut new_buf = [0u8; MAX_PATH_BUF];
        let mut new_len = cur.len();
        new_buf[..new_len].copy_from_slice(cur.as_bytes());
        if new_buf[new_len - 1] != b'/' { new_buf[new_len] = b'/'; new_len += 1; }
        new_buf[new_len..new_len + clean.len()].copy_from_slice(clean.as_bytes());
        new_len += clean.len();
        let new_path = core::str::from_utf8(&new_buf[..new_len]).map_err(|_| "Erro UTF-8")?;

        vfs.rename_file(old_path, new_path).map_err(|_| "Falha ao renomear arquivo no VFS")?;

        self.refresh_cache(vfs);
        self.set_status("[OK] Arquivo renomeado com sucesso no VFS", false);
        Ok(())
    }

    pub fn delete_selected(&mut self, vfs: &dyn VfsHost) -> Result<(), &'static str> {
        if self.is_read_only() {
            self.set_status("[Erro] Volume somente-leitura. Exclusao negada.", true);
            return Err("Volume somente-leitura");
        }

        let sel_idx = self.selected_index.ok_or("Nenhum arquivo selecionado")?;
        if sel_idx >= self.file_cache_count {
            return Err("Selecao invalida");
        }

        let name = self.file_cache[sel_idx].name_str();
        let cur = self.current_path_str();

        let mut path_buf = [0u8; MAX_PATH_BUF];
        let mut plen = cur.len();
        path_buf[..plen].copy_from_slice(cur.as_bytes());
        if path_buf[plen - 1] != b'/' { path_buf[plen] = b'/'; plen += 1; }
        path_buf[plen..plen + name.len()].copy_from_slice(name.as_bytes());
        plen += name.len();
        let path_str = core::str::from_utf8(&path_buf[..plen]).map_err(|_| "Erro UTF-8")?;

        vfs.remove_file(path_str).map_err(|_| "Falha ao excluir arquivo no VFS")?;

        self.refresh_cache(vfs);
        self.set_status("[OK] Arquivo excluido com sucesso do VFS", false);
        Ok(())
    }

    pub fn open_preview(&mut self, name: &str, vfs: &dyn VfsHost) {
        let n_len = name.len().min(31);
        self.preview_filename = [0; 32];
        self.preview_filename[..n_len].copy_from_slice(&name.as_bytes()[..n_len]);
        self.preview_filename_len = n_len;

        let cur = self.current_path_str();
        let mut path_buf = [0u8; MAX_PATH_BUF];
        let mut plen = cur.len();
        path_buf[..plen].copy_from_slice(cur.as_bytes());
        if path_buf[plen - 1] != b'/' { path_buf[plen] = b'/'; plen += 1; }
        path_buf[plen..plen + n_len].copy_from_slice(&name.as_bytes()[..n_len]);
        plen += n_len;

        if let Ok(path_str) = core::str::from_utf8(&path_buf[..plen]) {
            if let Ok(read_bytes) = vfs.read_file(path_str, &mut self.preview_content) {
                self.preview_content_len = read_bytes.min(256);
                self.dialog_kind = FileDialogKind::Preview;
                return;
            }
        }
        self.preview_content_len = 0;
        self.dialog_kind = FileDialogKind::Preview;
    }

    pub fn open_dialog(&mut self, kind: FileDialogKind) {
        self.dialog_kind = kind;
        self.input_len = 0;
        self.input_buffer = [0; 32];
    }

    pub fn close_dialog(&mut self) {
        self.dialog_kind = FileDialogKind::None;
        self.input_len = 0;
        self.input_buffer = [0; 32];
    }

    pub fn handle_key(&mut self, key: u8, vfs: &dyn VfsHost) {
        if self.dialog_kind == FileDialogKind::None {
            return;
        }

        match key {
            27 => {
                // ESC
                self.close_dialog();
            }
            10 | 13 => {
                // Enter: Confirma operação do diálogo
                let mut in_buf = [0u8; 32];
                let in_len = self.input_len;
                in_buf[..in_len].copy_from_slice(&self.input_buffer[..in_len]);

                match self.dialog_kind {
                    FileDialogKind::ConfirmDelete => {
                        let _ = self.delete_selected(vfs);
                        self.close_dialog();
                    }
                    FileDialogKind::NewFolder => {
                        let name = if in_len > 0 { core::str::from_utf8(&in_buf[..in_len]).unwrap_or("PASTA") } else { "PASTA" };
                        let _ = self.create_folder(name, vfs);
                        self.close_dialog();
                    }
                    FileDialogKind::NewFile => {
                        let name = if in_len > 0 { core::str::from_utf8(&in_buf[..in_len]).unwrap_or("NOVO.TXT") } else { "NOVO.TXT" };
                        let _ = self.create_file(name, vfs);
                        self.close_dialog();
                    }
                    FileDialogKind::Rename => {
                        let name = if in_len > 0 { core::str::from_utf8(&in_buf[..in_len]).unwrap_or("NOVO") } else { "NOVO" };
                        let _ = self.rename_selected(name, vfs);
                        self.close_dialog();
                    }
                    FileDialogKind::Preview => {
                        self.close_dialog();
                    }
                    FileDialogKind::None => {}
                }
            }
            8 => {
                // Backspace
                if self.input_len > 0 {
                    self.input_len -= 1;
                    self.input_buffer[self.input_len] = 0;
                }
            }
            b if (b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-') && self.input_len < 16 => {
                self.input_buffer[self.input_len] = b.to_ascii_uppercase();
                self.input_len += 1;
            }
            _ => {}
        }
    }

    pub fn handle_mouse_click(
        &mut self,
        mx: usize,
        my: usize,
        wx: usize,
        wy: usize,
        ww: usize,
        wh: usize,
        vfs: &dyn VfsHost,
        proc_host: &mut dyn ProcessHost,
    ) -> bool {
        // Se houver diálogo modal aberto, processa cliques nos botões do diálogo
        if self.dialog_kind != FileDialogKind::None {
            let dw = 320;
            let dh = 140;
            let dx = wx + (ww.saturating_sub(dw)) / 2;
            let dy = wy + (wh.saturating_sub(dh)) / 2;

            let btn_y = dy + dh - 32;
            let b0_x = dx + 20;
            let b0_w = 120;
            let b1_x = dx + dw - 140;
            let b1_w = 120;

            if my >= btn_y && my <= btn_y + 24 {
                // Botão Primário do Diálogo (Confirmar / Criar / Renomear)
                if mx >= b0_x && mx <= b0_x + b0_w {
                    let mut in_buf = [0u8; 32];
                    let in_len = self.input_len;
                    in_buf[..in_len].copy_from_slice(&self.input_buffer[..in_len]);

                    match self.dialog_kind {
                        FileDialogKind::ConfirmDelete => {
                            let _ = self.delete_selected(vfs);
                        }
                        FileDialogKind::NewFolder => {
                            let name = if in_len > 0 { core::str::from_utf8(&in_buf[..in_len]).unwrap_or("PASTA") } else { "PASTA" };
                            let _ = self.create_folder(name, vfs);
                        }
                        FileDialogKind::NewFile => {
                            let name = if in_len > 0 { core::str::from_utf8(&in_buf[..in_len]).unwrap_or("NOVO.TXT") } else { "NOVO.TXT" };
                            let _ = self.create_file(name, vfs);
                        }
                        FileDialogKind::Rename => {
                            let name = if in_len > 0 { core::str::from_utf8(&in_buf[..in_len]).unwrap_or("NOVO") } else { "NOVO" };
                            let _ = self.rename_selected(name, vfs);
                        }
                        FileDialogKind::Preview => {}
                        FileDialogKind::None => {}
                    }
                    self.close_dialog();
                    return true;
                }

                // Botão Secundário do Diálogo (Cancelar / Fechar)
                if mx >= b1_x && mx <= b1_x + b1_w {
                    self.close_dialog();
                    return true;
                }
            }

            // Clique fora do diálogo ou no botão fechar superior
            let close_x = dx + dw - 24;
            if mx >= close_x && mx <= close_x + 18 && my >= dy + 6 && my <= dy + 22 {
                self.close_dialog();
                return true;
            }

            return true;
        }

        // 1. Breadcrumb / Voltar (wy + 28 a wy + 48)
        let bc_y = wy + 28;
        if my >= bc_y && my <= bc_y + 20 {
            // Botão [< Voltar]
            if mx >= wx + 12 && mx <= wx + 80 {
                self.navigate_up(vfs);
                return true;
            }
        }

        // 2. Toolbar de Operações (wy + 52 a wy + 72)
        let tb_y = wy + 52;
        if my >= tb_y && my <= tb_y + 20 {
            let mut bx = wx + 12;

            // [+ Pasta]
            if mx >= bx && mx <= bx + 70 {
                self.dialog_kind = FileDialogKind::NewFolder;
                let default_name = b"PASTA";
                self.input_buffer[..default_name.len()].copy_from_slice(default_name);
                self.input_len = default_name.len();
                return true;
            }
            bx += 76;

            // [+ Arquivo]
            if mx >= bx && mx <= bx + 80 {
                self.dialog_kind = FileDialogKind::NewFile;
                let default_name = b"NOVO.TXT";
                self.input_buffer[..default_name.len()].copy_from_slice(default_name);
                self.input_len = default_name.len();
                return true;
            }
            bx += 86;

            // [Copiar]
            if mx >= bx && mx <= bx + 64 {
                let _ = self.copy_selected(vfs);
                return true;
            }
            bx += 70;

            // [Renomear]
            if mx >= bx && mx <= bx + 76 {
                if let Some(idx) = self.selected_index {
                    if idx < self.file_cache_count {
                        self.dialog_kind = FileDialogKind::Rename;
                        let name = self.file_cache[idx].name_str();
                        self.input_buffer[..name.len()].copy_from_slice(name.as_bytes());
                        self.input_len = name.len();
                    }
                } else {
                    self.set_status("Selecione um arquivo para renomear", true);
                }
                return true;
            }
            bx += 82;

            // [Excluir]
            if mx >= bx && mx <= bx + 68 {
                if self.selected_index.is_some() {
                    self.dialog_kind = FileDialogKind::ConfirmDelete;
                } else {
                    self.set_status("Selecione um arquivo para excluir", true);
                }
                return true;
            }
            bx += 74;

            // [Atualizar]
            if mx >= bx && mx <= bx + 74 {
                self.refresh_cache(vfs);
                self.set_status("Listagem de arquivos atualizada", false);
                return true;
            }
        }

        // 3. Barra Lateral de Volumes (wx + 10 a wx + 124, wy + 78 a wy + wh - 36)
        let sb_x = wx + 10;
        let sb_w = 114;
        let main_y = wy + 78;

        if mx >= sb_x && mx <= sb_x + sb_w && my >= main_y {
            // Item 0: /disk
            let item0_y = main_y + 24;
            if my >= item0_y && my <= item0_y + 22 {
                self.select_volume(0, vfs);
                return true;
            }

            // Item 1: /ext2
            let item1_y = item0_y + 26;
            if my >= item1_y && my <= item1_y + 22 {
                self.select_volume(1, vfs);
                return true;
            }
        }

        // 4. Área de Listagem de Arquivos (wx + 130 a wx + ww - 10)
        let list_x = wx + 130;
        let list_w = ww.saturating_sub(140);
        if mx >= list_x && mx <= list_x + list_w && my >= main_y && my <= wy + wh - 40 {
            let row_h = 20;
            for i in 0..self.file_cache_count {
                let row_y = main_y + 4 + (i * row_h);
                if my >= row_y && my <= row_y + row_h {
                    if self.selected_index == Some(i) {
                        // Segundo clique na mesma entrada: Executa ação (Navega se pasta, Preview se txt, Spawn se ELF)
                        let entry = self.file_cache[i];
                        let name_str = entry.name_str();
                        if entry.is_dir {
                            self.navigate_into(name_str, vfs);
                        } else if entry.is_elf {
                            let cur = self.current_path_str();
                            let mut path_buf = [0u8; MAX_PATH_BUF];
                            let mut plen = cur.len();
                            path_buf[..plen].copy_from_slice(cur.as_bytes());
                            if path_buf[plen - 1] != b'/' { path_buf[plen] = b'/'; plen += 1; }
                            path_buf[plen..plen + entry.name_len].copy_from_slice(&entry.name[..entry.name_len]);
                            plen += entry.name_len;
                            if let Ok(p_str) = core::str::from_utf8(&path_buf[..plen]) {
                                let _ = proc_host.spawn_app(p_str);
                                self.set_status("Aplicativo iniciado", false);
                            }
                        } else if entry.is_txt {
                            self.open_preview(name_str, vfs);
                        }
                    } else {
                        let is_dir = self.file_cache[i].is_dir;
                        self.selected_index = Some(i);
                        if is_dir {
                            self.set_status("Diretorio selecionado", false);
                        } else {
                            self.set_status("Arquivo selecionado", false);
                        }
                    }
                    return true;
                }
            }
        }

        false
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
        // Fundo da janela do gerenciador de arquivos
        fill_rect(buf, screen_w, screen_h, wx, wy, ww, wh, COLOR_CARD);

        // 1. Barra de Navegação e Breadcrumb (wy + 28 a wy + 50)
        let bc_y = wy + 28;
        fill_rect(buf, screen_w, screen_h, wx + 8, bc_y, ww - 16, 22, 0x000F_172A);
        draw_rect_outline(buf, screen_w, screen_h, wx + 8, bc_y, ww - 16, 22, COLOR_BORDER_SUBTLE);

        // Botão [< Voltar]
        let btn_back_w = 64;
        let is_at_root = self.current_path_str() == "/disk" || self.current_path_str() == "/ext2";
        let back_bg = if is_at_root { 0x001E_293B } else { 0x0025_63EB };
        let back_fg = if is_at_root { COLOR_TEXT_MUTED } else { COLOR_WHITE };
        fill_rect(buf, screen_w, screen_h, wx + 10, bc_y + 2, btn_back_w, 18, back_bg);
        draw_string(buf, screen_w, screen_h, "< Voltar", wx + 14, bc_y + 4, back_fg, Some(back_bg));

        // Breadcrumb
        draw_string(buf, screen_w, screen_h, "Local:", wx + 82, bc_y + 4, COLOR_CYAN, Some(0x000F_172A));
        draw_string(buf, screen_w, screen_h, self.current_path_str(), wx + 128, bc_y + 4, COLOR_TEXT_PRI, Some(0x000F_172A));

        // 2. Barra de Ferramentas / Toolbar (wy + 52 a wy + 74)
        let tb_y = wy + 52;
        fill_rect(buf, screen_w, screen_h, wx + 8, tb_y, ww - 16, 22, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, wx + 8, tb_y, ww - 16, 22, COLOR_BORDER);

        let mut bx = wx + 12;
        let is_ro = self.is_read_only();

        // Botão [+ Pasta]
        let bg_f = if is_ro { COLOR_CARD } else { 0x001E_293B };
        let bd_f = if is_ro { COLOR_BORDER } else { COLOR_EMERALD };
        let fg_f = if is_ro { COLOR_TEXT_MUTED } else { COLOR_EMERALD };
        fill_rect(buf, screen_w, screen_h, bx, tb_y + 2, 70, 18, bg_f);
        draw_rect_outline(buf, screen_w, screen_h, bx, tb_y + 2, 70, 18, bd_f);
        draw_string(buf, screen_w, screen_h, "+ Pasta", bx + 6, tb_y + 4, fg_f, Some(bg_f));
        bx += 76;

        // Botão [+ Arquivo]
        fill_rect(buf, screen_w, screen_h, bx, tb_y + 2, 80, 18, bg_f);
        draw_rect_outline(buf, screen_w, screen_h, bx, tb_y + 2, 80, 18, bd_f);
        draw_string(buf, screen_w, screen_h, "+ Arquivo", bx + 5, tb_y + 4, fg_f, Some(bg_f));
        bx += 86;

        // Botão [Copiar]
        let has_sel = self.selected_index.is_some();
        let bg_c = if has_sel && !is_ro { 0x001E_293B } else { COLOR_CARD };
        let fg_c = if has_sel && !is_ro { COLOR_CYAN } else { COLOR_TEXT_MUTED };
        fill_rect(buf, screen_w, screen_h, bx, tb_y + 2, 64, 18, bg_c);
        draw_rect_outline(buf, screen_w, screen_h, bx, tb_y + 2, 64, 18, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Copiar", bx + 8, tb_y + 4, fg_c, Some(bg_c));
        bx += 70;

        // Botão [Renomear]
        let fg_r = if has_sel && !is_ro { COLOR_AMBER } else { COLOR_TEXT_MUTED };
        fill_rect(buf, screen_w, screen_h, bx, tb_y + 2, 76, 18, bg_c);
        draw_rect_outline(buf, screen_w, screen_h, bx, tb_y + 2, 76, 18, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Renomear", bx + 6, tb_y + 4, fg_r, Some(bg_c));
        bx += 82;

        // Botão [Excluir]
        let fg_d = if has_sel && !is_ro { COLOR_RUBY } else { COLOR_TEXT_MUTED };
        fill_rect(buf, screen_w, screen_h, bx, tb_y + 2, 68, 18, bg_c);
        draw_rect_outline(buf, screen_w, screen_h, bx, tb_y + 2, 68, 18, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Excluir", bx + 8, tb_y + 4, fg_d, Some(bg_c));
        bx += 74;

        // Botão [Atualizar]
        fill_rect(buf, screen_w, screen_h, bx, tb_y + 2, 74, 18, 0x001E_293B);
        draw_rect_outline(buf, screen_w, screen_h, bx, tb_y + 2, 74, 18, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Atualizar", bx + 6, tb_y + 4, COLOR_TEXT_PRI, Some(0x001E_293B));

        // 3. Barra Lateral de Volumes e Locais (wx + 8 a wx + 124)
        let main_y = wy + 76;
        let main_h = wh.saturating_sub(112);
        let sb_x = wx + 8;
        let sb_w = 116;

        fill_rect(buf, screen_w, screen_h, sb_x, main_y, sb_w, main_h, COLOR_SURFACE_L1);
        draw_rect_outline(buf, screen_w, screen_h, sb_x, main_y, sb_w, main_h, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "VOLUMES", sb_x + 8, main_y + 6, COLOR_TEXT_MUTED, Some(COLOR_SURFACE_L1));

        // Item Volume 0: /disk
        let is_v0 = self.active_volume == 0;
        let bg_v0 = if is_v0 { 0x001E_293B } else { COLOR_SURFACE_L1 };
        let bd_v0 = if is_v0 { COLOR_CYAN } else { COLOR_BORDER_SUBTLE };
        let fg_v0 = if is_v0 { COLOR_CYAN } else { COLOR_TEXT_PRI };
        fill_rect(buf, screen_w, screen_h, sb_x + 4, main_y + 24, sb_w - 8, 22, bg_v0);
        draw_rect_outline(buf, screen_w, screen_h, sb_x + 4, main_y + 24, sb_w - 8, 22, bd_v0);
        draw_string(buf, screen_w, screen_h, "[D] /disk (RW)", sb_x + 8, main_y + 28, fg_v0, Some(bg_v0));

        // Item Volume 1: /ext2
        let is_v1 = self.active_volume == 1;
        let bg_v1 = if is_v1 { 0x001E_293B } else { COLOR_SURFACE_L1 };
        let bd_v1 = if is_v1 { COLOR_CYAN } else { COLOR_BORDER_SUBTLE };
        let fg_v1 = if is_v1 { COLOR_CYAN } else { COLOR_TEXT_PRI };
        fill_rect(buf, screen_w, screen_h, sb_x + 4, main_y + 50, sb_w - 8, 22, bg_v1);
        draw_rect_outline(buf, screen_w, screen_h, sb_x + 4, main_y + 50, sb_w - 8, 22, bd_v1);
        draw_string(buf, screen_w, screen_h, "[E] /ext2 (RO)", sb_x + 8, main_y + 54, fg_v1, Some(bg_v1));

        // 4. Área Principal de Listagem de Arquivos (wx + 128 a wx + ww - 8)
        let list_x = wx + 128;
        let list_w = ww.saturating_sub(136);
        fill_rect(buf, screen_w, screen_h, list_x, main_y, list_w, main_h, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, list_x, main_y, list_w, main_h, COLOR_BORDER);

        if self.file_cache_count == 0 {
            draw_string(buf, screen_w, screen_h, "(Diretorio vazio)", list_x + 20, main_y + 30, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        } else {
            let row_h = 20;
            for i in 0..self.file_cache_count.min(MAX_FILES) {
                let entry = &self.file_cache[i];
                let row_y = main_y + 4 + (i * row_h);
                let is_sel = self.selected_index == Some(i);

                let r_bg = if is_sel { 0x001E_293B } else { COLOR_CARD_ALT };
                fill_rect(buf, screen_w, screen_h, list_x + 2, row_y, list_w - 4, row_h - 2, r_bg);
                if is_sel {
                    draw_rect_outline(buf, screen_w, screen_h, list_x + 2, row_y, list_w - 4, row_h - 2, COLOR_BORDER_ACT);
                }

                let (tag, tag_fg) = if entry.is_dir {
                    ("[DIR] ", COLOR_EMERALD)
                } else if entry.is_elf {
                    ("[ELF] ", COLOR_CYAN)
                } else if entry.is_txt {
                    ("[TXT] ", 0x0093_C5FD)
                } else {
                    ("[ARQ] ", COLOR_TEXT_MUTED)
                };

                draw_string(buf, screen_w, screen_h, tag, list_x + 6, row_y + 2, tag_fg, Some(r_bg));
                draw_string(buf, screen_w, screen_h, entry.name_str(), list_x + 50, row_y + 2, COLOR_TEXT_PRI, Some(r_bg));

                // Ação rápida à direita
                let act_x = list_x + list_w - 60;
                if entry.is_dir {
                    draw_string(buf, screen_w, screen_h, "Abrir >", act_x, row_y + 2, COLOR_EMERALD, Some(r_bg));
                } else if entry.is_elf {
                    draw_string(buf, screen_w, screen_h, "> Rodar", act_x, row_y + 2, COLOR_WHITE, Some(r_bg));
                } else if entry.is_txt {
                    draw_string(buf, screen_w, screen_h, "Ver VFS", act_x, row_y + 2, COLOR_CYAN, Some(r_bg));
                }
            }
        }

        // 5. Barra Inferior de Status e Capacidade do Volume (wy + wh - 34 a wy + wh - 4)
        let bar_y = wy + wh - 34;
        fill_rect(buf, screen_w, screen_h, wx + 8, bar_y, ww - 16, 28, 0x000F_172A);
        draw_rect_outline(buf, screen_w, screen_h, wx + 8, bar_y, ww - 16, 28, COLOR_BORDER);

        // Linha 1: Capacidade do Volume ativo via StorageHost
        let vol_opt = storage.get_volume(self.active_volume);
        if let Some(vol) = vol_opt {
            let mut info_buf = [0u8; 80];
            let mut ilen = 0;
            let pfx = vol.mount_point_str();
            let fs_name = vol.fs_type_str();

            for b in b"Volume: " { info_buf[ilen] = *b; ilen += 1; }
            for b in pfx.as_bytes() { info_buf[ilen] = *b; ilen += 1; }
            for b in b" [" { info_buf[ilen] = *b; ilen += 1; }
            for b in fs_name.as_bytes() { info_buf[ilen] = *b; ilen += 1; }
            for b in b"] | " { info_buf[ilen] = *b; ilen += 1; }

            let mut nb = [0u8; 16];
            let nl1 = format_num(vol.total_mb as usize, &mut nb);
            for b in &nb[..nl1] { info_buf[ilen] = *b; ilen += 1; }
            for b in b" MB total (" { info_buf[ilen] = *b; ilen += 1; }

            let nl2 = format_num(vol.free_mb as usize, &mut nb);
            for b in &nb[..nl2] { info_buf[ilen] = *b; ilen += 1; }
            for b in b" MB livre) | " { info_buf[ilen] = *b; ilen += 1; }

            let mode_str = if self.is_read_only() { "Somente Leitura" } else { "Leitura e Escrita" };
            for b in mode_str.as_bytes() { info_buf[ilen] = *b; ilen += 1; }

            if let Ok(istr) = core::str::from_utf8(&info_buf[..ilen]) {
                draw_string(buf, screen_w, screen_h, istr, wx + 14, bar_y + 3, COLOR_TEXT_MUTED, Some(0x000F_172A));
            }
        }

        // Linha 2: Mensagem de Status da Operação
        let status_color = if self.status_is_error { COLOR_RUBY } else { COLOR_EMERALD };
        draw_string(buf, screen_w, screen_h, self.status_str(), wx + 14, bar_y + 16, status_color, Some(0x000F_172A));

        // 6. Renderização de Diálogos Modais (se ativos)
        if self.dialog_kind != FileDialogKind::None {
            self.render_dialog(buf, screen_w, screen_h, wx, wy, ww, wh);
        }
    }

    fn render_dialog(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        wx: usize,
        wy: usize,
        ww: usize,
        wh: usize,
    ) {
        let dw = 340;
        let dh = if self.dialog_kind == FileDialogKind::Preview { 180 } else { 120 };
        let dx = wx + (ww.saturating_sub(dw)) / 2;
        let dy = wy + (wh.saturating_sub(dh)) / 2;

        // Fundo do diálogo e borda acentuada
        fill_rect(buf, screen_w, screen_h, dx, dy, dw, dh, 0x000F_172A);
        draw_rect_outline(buf, screen_w, screen_h, dx, dy, dw, dh, COLOR_BORDER_ACT);

        // Barra superior do diálogo
        fill_rect(buf, screen_w, screen_h, dx + 1, dy + 1, dw - 2, 22, 0x001E_293B);
        let title = match self.dialog_kind {
            FileDialogKind::ConfirmDelete => "Confirmar Exclusao",
            FileDialogKind::NewFolder => "Criar Nova Pasta",
            FileDialogKind::NewFile => "Criar Novo Arquivo",
            FileDialogKind::Rename => "Renomear Item",
            FileDialogKind::Preview => "Quick Look — Visualizador VFS",
            FileDialogKind::None => "",
        };
        draw_string(buf, screen_w, screen_h, title, dx + 8, dy + 4, COLOR_CYAN, Some(0x001E_293B));

        // Botão X de fechar
        draw_string(buf, screen_w, screen_h, "[X]", dx + dw - 24, dy + 4, COLOR_RUBY, Some(0x001E_293B));

        let btn_y = dy + dh - 28;
        let b0_x = dx + 16;
        let b0_w = 140;
        let b1_x = dx + dw - 156;
        let b1_w = 140;

        match self.dialog_kind {
            FileDialogKind::ConfirmDelete => {
                let name = if let Some(i) = self.selected_index {
                    if i < self.file_cache_count { self.file_cache[i].name_str() } else { "item" }
                } else { "item" };

                draw_string(buf, screen_w, screen_h, "Excluir permanentemente este item?", dx + 16, dy + 36, COLOR_TEXT_PRI, Some(0x000F_172A));
                draw_string(buf, screen_w, screen_h, name, dx + 16, dy + 56, COLOR_RUBY, Some(0x000F_172A));

                // [ Sim, Excluir ]
                fill_rect(buf, screen_w, screen_h, b0_x, btn_y, b0_w, 20, 0x002D_1215);
                draw_rect_outline(buf, screen_w, screen_h, b0_x, btn_y, b0_w, 20, COLOR_RUBY);
                draw_string(buf, screen_w, screen_h, "Sim, Excluir", b0_x + 24, btn_y + 3, COLOR_RUBY, Some(0x002D_1215));

                // [ Cancelar ]
                fill_rect(buf, screen_w, screen_h, b1_x, btn_y, b1_w, 20, 0x001E_293B);
                draw_rect_outline(buf, screen_w, screen_h, b1_x, btn_y, b1_w, 20, COLOR_BORDER);
                draw_string(buf, screen_w, screen_h, "Cancelar", b1_x + 36, btn_y + 3, COLOR_TEXT_PRI, Some(0x001E_293B));
            }
            FileDialogKind::NewFolder | FileDialogKind::NewFile | FileDialogKind::Rename => {
                let prompt = match self.dialog_kind {
                    FileDialogKind::NewFolder => "Nome da pasta (ex: APPS):",
                    FileDialogKind::NewFile => "Nome do arquivo (ex: NOTAS.TXT):",
                    FileDialogKind::Rename => "Novo nome:",
                    _ => "",
                };
                draw_string(buf, screen_w, screen_h, prompt, dx + 16, dy + 32, COLOR_TEXT_PRI, Some(0x000F_172A));

                // Campo de entrada de texto
                let input_w = dw - 32;
                fill_rect(buf, screen_w, screen_h, dx + 16, dy + 50, input_w, 22, 0x001E_293B);
                draw_rect_outline(buf, screen_w, screen_h, dx + 16, dy + 50, input_w, 22, COLOR_BORDER_ACT);

                let typed = core::str::from_utf8(&self.input_buffer[..self.input_len]).unwrap_or("");
                draw_string(buf, screen_w, screen_h, typed, dx + 22, dy + 54, COLOR_WHITE, Some(0x001E_293B));

                // [ Confirmar ]
                let confirm_text = match self.dialog_kind {
                    FileDialogKind::NewFolder => "Criar Pasta",
                    FileDialogKind::NewFile => "Criar Arquivo",
                    FileDialogKind::Rename => "Renomear",
                    _ => "OK",
                };
                fill_rect(buf, screen_w, screen_h, b0_x, btn_y, b0_w, 20, 0x0025_63EB);
                draw_rect_outline(buf, screen_w, screen_h, b0_x, btn_y, b0_w, 20, COLOR_CYAN);
                draw_string(buf, screen_w, screen_h, confirm_text, b0_x + 28, btn_y + 3, COLOR_WHITE, Some(0x0025_63EB));

                // [ Cancelar ]
                fill_rect(buf, screen_w, screen_h, b1_x, btn_y, b1_w, 20, 0x001E_293B);
                draw_rect_outline(buf, screen_w, screen_h, b1_x, btn_y, b1_w, 20, COLOR_BORDER);
                draw_string(buf, screen_w, screen_h, "Cancelar", b1_x + 36, btn_y + 3, COLOR_TEXT_PRI, Some(0x001E_293B));
            }
            FileDialogKind::Preview => {
                let name = core::str::from_utf8(&self.preview_filename[..self.preview_filename_len]).unwrap_or("arquivo");
                draw_string(buf, screen_w, screen_h, name, dx + 16, dy + 30, COLOR_CYAN, Some(0x000F_172A));

                // Caixa de texto com conteúdo lido
                let tb_w = dw - 32;
                let tb_h = 88;
                fill_rect(buf, screen_w, screen_h, dx + 16, dy + 48, tb_w, tb_h, 0x001E_293B);
                draw_rect_outline(buf, screen_w, screen_h, dx + 16, dy + 48, tb_w, tb_h, COLOR_BORDER);

                let c_len = self.preview_content_len.min(256);
                let mut line_y = dy + 52;
                let mut line_start = 0;
                let mut rendered = 0;
                for i in 0..c_len {
                    let b = self.preview_content[i];
                    let is_break = b == b'\n' || (i - line_start) >= 36 || i == c_len - 1;
                    if is_break {
                        let end_i = if b == b'\n' { i } else { i + 1 };
                        if end_i > line_start && rendered < 4 {
                            if let Ok(lstr) = core::str::from_utf8(&self.preview_content[line_start..end_i]) {
                                draw_string(buf, screen_w, screen_h, lstr, dx + 22, line_y, COLOR_TEXT_PRI, Some(0x001E_293B));
                                line_y += 18;
                                rendered += 1;
                            }
                        }
                        line_start = i + 1;
                    }
                }

                // [ Fechar ]
                let fc_x = dx + (dw - 100) / 2;
                fill_rect(buf, screen_w, screen_h, fc_x, btn_y, 100, 20, 0x001E_293B);
                draw_rect_outline(buf, screen_w, screen_h, fc_x, btn_y, 100, 20, COLOR_BORDER);
                draw_string(buf, screen_w, screen_h, "Fechar", fc_x + 28, btn_y + 3, COLOR_WHITE, Some(0x001E_293B));
            }
            FileDialogKind::None => {}
        }
    }
}
