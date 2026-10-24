//! Gerenciador de Sessão e Laço Principal do Aladdin Desktop

use crate::contracts::framebuffer::{DirtyRect, FramebufferDevice};
use crate::contracts::system::{CapabilityMode, SystemHost};
use crate::contracts::vfs::VfsHost;
use crate::contracts::process::ProcessHost;
use crate::compositor::*;
use crate::font::*;
use crate::wm::{WindowManager, Window};
use crate::apps::*;
use super::wingpanel::render_wingpanel;
use super::dock::render_dock;
use super::start_menu::*;

pub const CURSOR_W: usize = 11;
pub const CURSOR_H: usize = 16;
pub const CURSOR_BITMAP: [&str; 16] = [
    "#..........",
    "##.........",
    "#*#........",
    "#**#.......",
    "#***#......",
    "#****#.....",
    "#*****#....",
    "#******#...",
    "#*******#..",
    "#*****####.",
    "#**#**#....",
    "#*#.#**#...",
    "##..#**#...",
    "#....#**#..",
    ".....#**#..",
    "......##...",
];

pub struct AladdinSession {
    pub wm: WindowManager,
    pub term: TerminalApp,
    pub files: FileManagerApp,
    pub monitor: SystemMonitorApp,
    pub about: AboutApp,
    pub settings: SettingsApp,
    pub mode: CapabilityMode,
    pub start_menu_open: bool,
    pub should_exit: bool,
    pub spawn_count: usize,
    pub prev_buttons: u8,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub cursor_saved_bg: [u32; CURSOR_W * CURSOR_H],
    pub cursor_valid: bool,
    pub client_damaged: bool,
}

impl AladdinSession {
    pub fn new() -> Self {
        let mut session = Self {
            wm: WindowManager::new(),
            term: TerminalApp::new(),
            files: FileManagerApp::new(),
            monitor: SystemMonitorApp::new(),
            about: AboutApp::new(),
            settings: SettingsApp::new(),
            mode: CapabilityMode::Balanced,
            start_menu_open: false,
            should_exit: false,
            spawn_count: 0,
            prev_buttons: 0,
            cursor_x: 512,
            cursor_y: 384,
            cursor_saved_bg: [0; CURSOR_W * CURSOR_H],
            cursor_valid: false,
            client_damaged: false,
        };

        session.init_default_windows();
        session.term.init_defaults();
        session
    }

    pub fn create_client_window(
        &mut self,
        owner_pid: usize,
        title: &str,
        client_w: usize,
        client_h: usize,
    ) -> Result<usize, ()> {
        let count = self.wm.windows.iter().filter(|w| w.is_some()).count();
        let x = 80 + (count * 28);
        let y = 60 + (count * 24);
        let res = self.wm.create_client_window(owner_pid, title, x, y, client_w, client_h);
        if res.is_ok() {
            self.client_damaged = true;
        }
        res
    }

    pub fn update_client_buffer(
        &mut self,
        owner_pid: usize,
        win_id: usize,
        buffer: &[u32],
    ) -> Result<(), crate::wm::WindowOpError> {
        let res = self.wm.update_client_buffer(owner_pid, win_id, buffer);
        if res.is_ok() {
            self.client_damaged = true;
        }
        res
    }

    pub fn close_client_window(
        &mut self,
        owner_pid: usize,
        win_id: usize,
    ) -> Result<(), crate::wm::WindowOpError> {
        let res = self.wm.close_client_window(owner_pid, win_id);
        if res.is_ok() {
            self.client_damaged = true;
        }
        res
    }

    pub fn close_windows_for_pid(&mut self, pid: usize) {
        self.wm.close_all_for_pid(pid);
        self.client_damaged = true;
    }

    fn init_default_windows(&mut self) {
        // Janela 0: Monitor de Sistema (aberta)
        self.wm.windows[0] = Some(Window::new(0, "Jinn OS - Monitor de Sistema", 60, 60, 380, 270));

        // Janela 1: Terminal Interativo (aberta e com foco)
        self.wm.windows[1] = Some(Window::new(1, "Terminal Interativo (Ring 3 Ready)", 470, 80, 460, 240));

        // Janela 2: Sobre o Jinn OS (fechada)
        let mut w2 = Window::new(2, "Sobre o Jinn OS", 240, 330, 420, 200);
        w2.is_open = false;
        self.wm.windows[2] = Some(w2);

        // Janela 3: Gerenciador de Arquivos Aladdin (fechada)
        let mut w3 = Window::new(3, "Aladdin Files - Volume /disk (FAT32)", 160, 190, 440, 240);
        w3.is_open = false;
        self.wm.windows[3] = Some(w3);

        // Janela 4: Ajustes e Configurações (fechada)
        let mut w4 = Window::new(4, "Ajustes do Aladdin Desktop", 200, 150, 440, 276);
        w4.is_open = false;
        self.wm.windows[4] = Some(w4);

        // Terminal como janela inicial no topo do Z-order
        self.wm.order = [0, 2, 3, 4, 5, 6, 7, 1];
    }

    pub fn erase_cursor(&mut self, buf: &mut [u32], screen_w: usize, screen_h: usize) -> Option<DirtyRect> {
        if !self.cursor_valid { return None; }
        let cx = self.cursor_x;
        let cy = self.cursor_y;

        for y in 0..CURSOR_H {
            let py = cy + y;
            if py >= screen_h { break; }
            let row_start = py * screen_w;
            for x in 0..CURSOR_W {
                let px = cx + x;
                if px >= screen_w { break; }
                buf[row_start + px] = self.cursor_saved_bg[y * CURSOR_W + x];
            }
        }
        self.cursor_valid = false;
        Some(DirtyRect::new(cx, cy, CURSOR_W, CURSOR_H))
    }

    pub fn draw_cursor(&mut self, buf: &mut [u32], screen_w: usize, screen_h: usize, new_x: usize, new_y: usize) -> DirtyRect {
        let cx = new_x.min(screen_w.saturating_sub(CURSOR_W));
        let cy = new_y.min(screen_h.saturating_sub(CURSOR_H));

        self.cursor_x = cx;
        self.cursor_y = cy;

        // 1. Salva fundo
        for y in 0..CURSOR_H {
            let py = cy + y;
            if py >= screen_h { break; }
            let row_start = py * screen_w;
            for x in 0..CURSOR_W {
                let px = cx + x;
                if px >= screen_w { break; }
                self.cursor_saved_bg[y * CURSOR_W + x] = buf[row_start + px];
            }
        }

        // 2. Renderiza sprite do cursor
        for (y, row_str) in CURSOR_BITMAP.iter().enumerate() {
            let py = cy + y;
            if py >= screen_h { break; }
            let row_start = py * screen_w;

            for (x, ch) in row_str.bytes().enumerate() {
                let px = cx + x;
                if px >= screen_w { break; }
                match ch {
                    b'#' => buf[row_start + px] = COLOR_BLACK,
                    b'*' => buf[row_start + px] = COLOR_WHITE,
                    _ => {}
                }
            }
        }

        self.cursor_valid = true;
        DirtyRect::new(cx, cy, CURSOR_W, CURSOR_H)
    }

    pub fn render_scene(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        sys: &dyn SystemHost,
        mouse_x: usize,
        mouse_y: usize,
    ) {
        // 1. Fundo principal
        fill_rect(buf, screen_w, screen_h, 0, 0, screen_w, screen_h, COLOR_BG);

        // 2. Wingpanel superior
        let stats = sys.stats();
        render_wingpanel(buf, screen_w, screen_h, &stats, sys.storage_profile_name(), self.mode, self.start_menu_open);

        // 3. Marca d'água elegante
        draw_string(buf, screen_w, screen_h, "Aladdin Desktop 7.0 | Jinn Microkernel x86_64", screen_w.saturating_sub(370), screen_h.saturating_sub(16), COLOR_BORDER, None);

        // 4. Renderiza janelas em Z-order (do fundo para o topo)
        let top_idx = self.wm.top_window_id();
        for &win_idx in self.wm.order.iter() {
            if let Some(win) = self.wm.windows[win_idx] {
                if !win.is_open || win.is_minimized { continue; }
                let is_active = win_idx == top_idx;

                self.wm.render_window_chrome(buf, screen_w, screen_h, &win, is_active, self.mode);

                // Renderiza superfície cliente se for uma janela com buffer próprio (Ring 3 ou externa)
                if let Some(ref client_buf) = self.wm.client_buffers[win_idx] {
                    let clip = win.client_clip();
                    let client_x = win.x + 1;
                    let client_y = win.y + crate::wm::TITLE_BAR_HEIGHT + 1;
                    let client_w = win.client_width();
                    let client_h = win.client_height();

                    for cy in 0..client_h {
                        let py = client_y + cy;
                        if py < clip.min_y || py >= clip.max_y || py >= screen_h { continue; }
                        let row_start = py * screen_w;
                        let client_row = cy * client_w;
                        for cx in 0..client_w {
                            let px = client_x + cx;
                            if px < clip.min_x || px >= clip.max_x || px >= screen_w { continue; }
                            let p_idx = client_row + cx;
                            if p_idx < client_buf.len() {
                                buf[row_start + px] = client_buf[p_idx];
                            }
                        }
                    }
                } else {
                    match win.id {
                        0 => self.monitor.render(buf, screen_w, screen_h, win.x, win.y, win.width, win.height, &stats),
                        1 => self.term.render(buf, screen_w, screen_h, win.x, win.y, win.width, win.height, is_active, stats.current_tick, self.spawn_count),
                        2 => self.about.render(buf, screen_w, screen_h, win.x, win.y),
                        3 => self.files.render(buf, screen_w, screen_h, win.x, win.y, win.width, win.height, sys.storage_profile_name()),
                        4 => self.settings.render(buf, screen_w, screen_h, win.x, win.y, win.width, win.height, self.mode, sys),
                        _ => {}
                    }
                }
            }
        }

        // 5. Plank Dock inferior
        render_dock(buf, screen_w, screen_h, mouse_x, mouse_y, &self.wm);

        // 6. Menu Iniciar popup (se ativo)
        if self.start_menu_open {
            render_start_menu(buf, screen_w, screen_h, mouse_x, mouse_y);
        }
    }

    pub fn handle_mouse(
        &mut self,
        mx: usize,
        my: usize,
        buttons: u8,
        sys: &dyn SystemHost,
        vfs: &dyn VfsHost,
        proc_host: &mut dyn ProcessHost,
        screen_w: usize,
        screen_h: usize,
    ) -> (bool, Option<DirtyRect>) {
        let left_clicked = (buttons & 1) != 0;
        let prev_left_clicked = (self.prev_buttons & 1) != 0;
        self.prev_buttons = buttons;

        // Se soltou o botão esquerdo, encerra arrasto
        if !left_clicked {
            for win_opt in self.wm.windows.iter_mut() {
                if let Some(win) = win_opt {
                    win.is_dragging = false;
                }
            }
        }

        // Atualiza arrasto de janela ativo
        let mut dragging_any = false;
        let mut drag_damage: Option<DirtyRect> = None;
        for win_opt in self.wm.windows.iter_mut() {
            if let Some(win) = win_opt {
                if win.is_dragging {
                    let old_x = win.x;
                    let old_y = win.y;
                    let new_x = mx.saturating_sub(win.drag_offset_x);
                    let new_y = my.saturating_sub(win.drag_offset_y);
                    win.x = new_x.min(screen_w.saturating_sub(win.width));
                    win.y = new_y.min((screen_h.saturating_sub(54)).saturating_sub(win.height));
                    dragging_any = true;

                    let old_box = DirtyRect::new(old_x.saturating_sub(2), old_y.saturating_sub(2), win.width + 4, win.height + 4);
                    let new_box = DirtyRect::new(win.x.saturating_sub(2), win.y.saturating_sub(2), win.width + 4, win.height + 4);
                    drag_damage = Some(old_box.union(&new_box));
                }
            }
        }

        // Clique com botão esquerdo (transição de 0 para 1)
        if left_clicked && !prev_left_clicked && !dragging_any {
            // 1. Wingpanel superior (y: 0..26)
            if my < 26 {
                // Launcher [# Aladdin]
                if mx >= 8 && mx <= 122 {
                    self.start_menu_open = !self.start_menu_open;
                    return (false, None);
                }
                // RAM Pill
                let ram_x = screen_w.saturating_sub(424);
                if mx >= ram_x && mx <= ram_x + 120 {
                    if let Some(w) = self.wm.windows[0].as_mut() { w.is_open = true; w.is_minimized = false; }
                    self.wm.bring_to_front(0);
                    return (false, None);
                }
                // JSA Storage Profile
                let jsa_x = screen_w.saturating_sub(298);
                if mx >= jsa_x && mx <= jsa_x + 124 {
                    sys.cycle_storage_profile();
                    return (false, None);
                }
                // FX Capability Mode
                let fx_x = screen_w.saturating_sub(168);
                if mx >= fx_x && mx <= fx_x + 84 {
                    self.mode = match self.mode {
                        CapabilityMode::Basic => CapabilityMode::Balanced,
                        CapabilityMode::Balanced => CapabilityMode::Advanced,
                        CapabilityMode::Advanced => CapabilityMode::Basic,
                    };
                    return (false, None);
                }
                // Sair Button
                let exit_x = screen_w.saturating_sub(78);
                if mx >= exit_x && mx <= exit_x + 70 {
                    self.should_exit = true;
                    return (false, None);
                }
            }

            // 2. Menu Iniciar popup
            if self.start_menu_open {
                if mx >= MENU_X && mx <= MENU_X + MENU_W && my >= MENU_Y && my <= MENU_Y + MENU_H {
                    if my >= MENU_Y + 28 {
                        let item_idx = (my - (MENU_Y + 28)) / 26;
                        match item_idx {
                            0 => {
                                if let Some(w) = self.wm.windows[0].as_mut() { w.is_open = true; w.is_minimized = false; }
                                self.wm.bring_to_front(0);
                            }
                            1 => {
                                if let Some(w) = self.wm.windows[1].as_mut() { w.is_open = true; w.is_minimized = false; }
                                self.wm.bring_to_front(1);
                            }
                            2 => {
                                if let Some(w) = self.wm.windows[3].as_mut() { w.is_open = true; w.is_minimized = false; }
                                self.files.refresh_cache(vfs);
                                self.wm.bring_to_front(3);
                            }
                            3 => {
                                if let Ok(pid) = proc_host.spawn_app("/disk/ola.elf") {
                                    self.spawn_count += 1;
                                    self.term.push_line("Processo Ring 3 iniciado!", COLOR_EMERALD);
                                    let mut p_buf = [0u8; 16];
                                    let p_len = crate::compositor::format_num(pid, &mut p_buf);
                                    if let Ok(pid_str) = core::str::from_utf8(&p_buf[..p_len]) {
                                        self.term.push_line(pid_str, COLOR_CYAN);
                                    }
                                }
                            }
                            4 => {
                                if let Some(w) = self.wm.windows[2].as_mut() { w.is_open = true; w.is_minimized = false; }
                                self.wm.bring_to_front(2);
                            }
                            5 => sys.cycle_storage_profile(),
                            6 => sys.reboot(),
                            7 => self.should_exit = true,
                            _ => {}
                        }
                    }
                    self.start_menu_open = false;
                    return (false, None);
                } else {
                    self.start_menu_open = false;
                }
            }

            // 3. Plank Dock inferior (y >= screen_h - 54)
            let dock_x = (screen_w / 2).saturating_sub(crate::desktop::dock::DOCK_WIDTH / 2);
            let dock_y = screen_h.saturating_sub(crate::desktop::dock::DOCK_HEIGHT + 10);
            if my >= dock_y && my <= dock_y + crate::desktop::dock::DOCK_HEIGHT && mx >= dock_x && mx <= dock_x + crate::desktop::dock::DOCK_WIDTH {
                let slot_idx = (mx.saturating_sub(dock_x)) / 94;
                let top_id = self.wm.top_window_id();
                match slot_idx {
                    0 => {
                        if let Some(w) = self.wm.windows[0].as_mut() {
                            if w.is_open && !w.is_minimized && top_id == 0 {
                                w.is_minimized = true;
                            } else {
                                w.is_open = true;
                                w.is_minimized = false;
                                self.wm.bring_to_front(0);
                            }
                        }
                    }
                    1 => {
                        if let Some(w) = self.wm.windows[1].as_mut() {
                            if w.is_open && !w.is_minimized && top_id == 1 {
                                w.is_minimized = true;
                            } else {
                                w.is_open = true;
                                w.is_minimized = false;
                                self.wm.bring_to_front(1);
                            }
                        }
                    }
                    2 => {
                        if let Some(w) = self.wm.windows[3].as_mut() {
                            if w.is_open && !w.is_minimized && top_id == 3 {
                                w.is_minimized = true;
                            } else {
                                w.is_open = true;
                                w.is_minimized = false;
                                self.files.refresh_cache(vfs);
                                self.wm.bring_to_front(3);
                            }
                        }
                    }
                    3 => {
                        if let Ok(pid) = proc_host.spawn_app("/disk/ola.elf") {
                            self.spawn_count += 1;
                            self.term.push_line("Processo Ring 3 criado!", COLOR_EMERALD);
                            let mut p_buf = [0u8; 16];
                            let p_len = crate::compositor::format_num(pid, &mut p_buf);
                            if let Ok(pid_str) = core::str::from_utf8(&p_buf[..p_len]) {
                                self.term.push_line(pid_str, COLOR_CYAN);
                            }
                            if let Some(w) = self.wm.windows[1].as_mut() { w.is_open = true; w.is_minimized = false; }
                            self.wm.bring_to_front(1);
                        }
                    }
                    4 => {
                        // Slot 4: Ajustes (Win 4)
                        if let Some(w) = self.wm.windows[4].as_mut() {
                            if w.is_open && !w.is_minimized && top_id == 4 {
                                w.is_minimized = true;
                            } else {
                                w.is_open = true;
                                w.is_minimized = false;
                                self.wm.bring_to_front(4);
                            }
                        }
                    }
                    5 => {
                        // Slot 5: Sobre (Win 2)
                        if let Some(w) = self.wm.windows[2].as_mut() {
                            if w.is_open && !w.is_minimized && top_id == 2 {
                                w.is_minimized = true;
                            } else {
                                w.is_open = true;
                                w.is_minimized = false;
                                self.wm.bring_to_front(2);
                            }
                        }
                    }
                    _ => {}
                }
                return (false, None);
            }

            // 4. Interação com Janelas (ordem inversa de profundidade)
            if let Some(clicked_idx) = self.wm.find_window_at(mx, my) {
                self.wm.bring_to_front(clicked_idx);

                if let Some(win) = self.wm.windows[clicked_idx].as_mut() {
                    // Controles de janela
                    if win.is_in_close_btn(mx, my) {
                        let is_client = win.owner_pid.is_some();
                        win.is_open = false;
                        if is_client {
                            self.wm.close_window(clicked_idx);
                        }
                        return (false, None);
                    }
                    if win.is_in_min_btn(mx, my) {
                        win.is_minimized = true;
                        return (false, None);
                    }
                    if win.is_in_max_btn(mx, my) {
                        let damage = win.toggle_maximize(screen_w, screen_h, 26, 54);
                        return (false, Some(damage));
                    }

                    // Início de arrasto pela barra de título
                    if win.is_in_title_bar(mx, my) && !win.is_maximized {
                        win.is_dragging = true;
                        win.drag_offset_x = mx.saturating_sub(win.x);
                        win.drag_offset_y = my.saturating_sub(win.y);
                        return (true, None);
                    }

                    // Interações de corpo da janela
                    if win.id == 1 {
                        // Terminal: botão + Executar ola-mundo
                        let btn_y = win.y + win.height - 34;
                        if mx >= win.x + 8 && mx <= win.x + 228 && my >= btn_y && my <= btn_y + 24 {
                            if let Ok(pid) = proc_host.spawn_app("/disk/ola.elf") {
                                self.spawn_count += 1;
                                self.term.push_line("Spawned ola-mundo (U3) via Terminal!", COLOR_EMERALD);
                                let mut p_buf = [0u8; 16];
                                let p_len = crate::compositor::format_num(pid, &mut p_buf);
                                if let Ok(pid_str) = core::str::from_utf8(&p_buf[..p_len]) {
                                    self.term.push_line(pid_str, COLOR_CYAN);
                                }
                            }
                        }
                    } else if win.id == 3 {
                        // Gerenciador de Arquivos
                        let tab_y = win.y + 48;
                        if my >= tab_y && my <= tab_y + 18 {
                            if mx >= win.x + 12 && mx <= win.x + 126 {
                                self.files.active_volume = 0;
                                self.files.close_preview();
                                self.files.refresh_cache(vfs);
                            }
                            if mx >= win.x + 134 && mx <= win.x + 248 {
                                self.files.active_volume = 1;
                                self.files.close_preview();
                                self.files.refresh_cache(vfs);
                            }
                        }

                        if self.files.preview_active {
                            let p_x = win.x + 12;
                            let p_y = win.y + 70;
                            let p_w = win.width - 24;
                            let btn_c_x = p_x + p_w - 76 - 4;

                            // Botão Fechar Preview
                            if mx >= btn_c_x && mx <= btn_c_x + 76 && my >= p_y + 2 && my <= p_y + 18 {
                                self.files.close_preview();
                            }
                        } else {
                            // Botão [Atualizar]
                            let files_box_x = win.x + 12;
                            let files_box_y = win.y + 70;
                            let files_box_w = win.width - 24;
                            let btn_rf_x = files_box_x + files_box_w - 88 - 8;

                            if mx >= btn_rf_x && mx <= btn_rf_x + 88 && my >= files_box_y + 6 && my <= files_box_y + 24 {
                                self.files.refresh_cache(vfs);
                            }

                            // Botões de itens de arquivo
                            for i in 0..self.files.file_cache_count.min(3) {
                                let item_y = files_box_y + 32 + (i * 22);
                                let btn_x = files_box_x + files_box_w - 64 - 8;
                                let entry = self.files.file_cache[i];

                                if mx >= btn_x && mx <= btn_x + 64 && my >= item_y - 1 && my <= item_y + 15 {
                                    if let Ok(name_str) = core::str::from_utf8(&entry.name[..entry.name_len]) {
                                        if entry.is_elf {
                                            let mut path_buf = [0u8; 32];
                                            let pfx = if self.files.active_volume == 1 { "/ext2/" } else { "/disk/" };
                                            path_buf[..pfx.len()].copy_from_slice(pfx.as_bytes());
                                            path_buf[pfx.len()..(pfx.len() + entry.name_len)].copy_from_slice(&entry.name[..entry.name_len]);
                                            if let Ok(path_str) = core::str::from_utf8(&path_buf[..(pfx.len() + entry.name_len)]) {
                                                let _ = proc_host.spawn_app(path_str);
                                            }
                                        } else if entry.is_txt {
                                            self.files.open_preview(name_str, vfs);
                                        }
                                    }
                                }
                            }
                        }
                    } else if win.id == 4 {
                        // Ajustes e Configurações
                        let btn_y = win.y + 72;
                        if my >= btn_y && my <= btn_y + 22 {
                            let b0_x = win.x + 12;
                            let b1_x = b0_x + 108;
                            let b2_x = b1_x + 108;
                            // Eco
                            if mx >= b0_x && mx <= b0_x + 100 { self.mode = CapabilityMode::Basic; }
                            // Equilibrado
                            if mx >= b1_x && mx <= b1_x + 100 { self.mode = CapabilityMode::Balanced; }
                            // Glow Neon
                            if mx >= b2_x && mx <= b2_x + 100 { self.mode = CapabilityMode::Advanced; }
                        }

                        // Alternar Perfil Firewall JinnWall
                        let fw_btn_y = win.y + 166 + 40;
                        if my >= fw_btn_y && my <= fw_btn_y + 18 {
                            if mx >= win.x + 20 && mx <= win.x + 240 {
                                sys.toggle_jinnwall_profile();
                            }
                        }

                        // Ações rápidas
                        let act_y = win.y + 236;
                        if my >= act_y && my <= act_y + 24 {
                            // Ciclar JSA
                            if mx >= win.x + 12 && mx <= win.x + 152 {
                                sys.cycle_storage_profile();
                            }
                            // Reiniciar
                            if mx >= win.x + 160 && mx <= win.x + 300 {
                                sys.reboot();
                            }
                        }
                    }
                }
            }
        }

        (dragging_any, drag_damage)
    }

    pub fn handle_key(&mut self, ch: u8, proc_host: &mut dyn ProcessHost) {
        if ch == 0x1B { // ASCII ESC: sair para o console texto
            self.should_exit = true;
            return;
        }

        let top_idx = self.wm.top_window_id();
        if top_idx == 1 { // Terminal interativo
            self.term.handle_key(ch, proc_host);
        }
    }

    pub fn run(
        &mut self,
        fb: &mut dyn FramebufferDevice,
        sys: &dyn SystemHost,
        vfs: &dyn VfsHost,
        proc_host: &mut dyn ProcessHost,
        backbuffer: &mut [u32],
        poll_key: &mut dyn FnMut() -> Option<u8>,
        poll_mouse: &mut dyn FnMut() -> (usize, usize, u8),
    ) {
        let (screen_w, screen_h, _pitch) = fb.dimensions();
        self.should_exit = false;

        // Atualiza cache inicial de arquivos
        self.files.refresh_cache(vfs);

        let (mut last_mx, mut last_my, mut last_btn) = poll_mouse();
        let mut last_ticks = sys.stats().current_tick;

        // Render inicial
        let _ = self.erase_cursor(backbuffer, screen_w, screen_h);
        self.render_scene(backbuffer, screen_w, screen_h, sys, last_mx, last_my);
        let _ = self.draw_cursor(backbuffer, screen_w, screen_h, last_mx, last_my);
        fb.swap_full(backbuffer);

        loop {
            // 1. Processa entrada do teclado
            let mut key_pressed = false;
            while let Some(ch) = poll_key() {
                self.handle_key(ch, proc_host);
                key_pressed = true;
                if self.should_exit { break; }
            }
            if self.should_exit { break; }

            // 2. Lê mouse e telemetria
            let (mx, my, buttons) = poll_mouse();
            let current_ticks = sys.stats().current_tick;

            let (dragging_any, _drag_damage) = self.handle_mouse(mx, my, buttons, sys, vfs, proc_host, screen_w, screen_h);

            let mouse_moved = (mx != last_mx) || (my != last_my);
            let button_changed = buttons != last_btn;
            let time_changed = current_ticks.saturating_sub(last_ticks) >= 50; // Atualiza a cada 500ms

            let need_scene_redraw = button_changed || key_pressed || dragging_any || time_changed || self.client_damaged;

            if need_scene_redraw {
                self.client_damaged = false;
                let _ = self.erase_cursor(backbuffer, screen_w, screen_h);
                self.render_scene(backbuffer, screen_w, screen_h, sys, mx, my);
                let _ = self.draw_cursor(backbuffer, screen_w, screen_h, mx, my);
                fb.swap_full(backbuffer);

                last_mx = mx;
                last_my = my;
                last_btn = buttons;
                last_ticks = current_ticks;
            } else if mouse_moved {
                // Caminho Rápido de Cursor (< 5 µs com DirtyRect)
                let old_box = self.erase_cursor(backbuffer, screen_w, screen_h);
                let new_box = self.draw_cursor(backbuffer, screen_w, screen_h, mx, my);
                let damage = match old_box {
                    Some(ob) => ob.union(&new_box),
                    None => new_box,
                };
                fb.swap_rect(&damage, backbuffer);

                last_mx = mx;
                last_my = my;
            } else {
                // Repouso: cede ciclos de CPU
                sys.yield_cpu();
            }
        }
    }
}
