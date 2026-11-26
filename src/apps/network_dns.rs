//! Aplicativo / Aba "Rede & DNS" — Tabela Local de DNS (Hosts + Cache LRU TTL) e Upstreams

use alloc::vec::Vec;
use crate::compositor::*;
use crate::contracts::system::{DnsServersConfig, DnsTableEntry, SystemHost};
use crate::desktop::notifications::NotificationCenter;
use crate::font::*;
use crate::ui::{draw_button, draw_text_input, ButtonStyle, WidgetState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsFocusField {
    None,
    PrimaryIp,
    SecondaryIp,
    FallbackIp,
    QueryHost,
    ModalHost,
    ModalIp,
}

struct InputField {
    buf: [u8; 64],
    len: usize,
}

impl InputField {
    const fn empty() -> Self {
        Self { buf: [0u8; 64], len: 0 }
    }

    fn set_str(&mut self, s: &str) {
        self.buf = [0u8; 64];
        let b = s.as_bytes();
        let l = b.len().min(63);
        self.buf[..l].copy_from_slice(&b[..l]);
        self.len = l;
    }

    fn set_ip(&mut self, ip: [u8; 4]) {
        self.buf = [0u8; 64];
        self.len = format_ip_into(&mut self.buf, ip);
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    fn push_byte(&mut self, b: u8) {
        if self.len < 63 && b >= 32 && b <= 126 {
            self.buf[self.len] = b;
            self.len += 1;
        }
    }

    fn pop_byte(&mut self) {
        if self.len > 0 {
            self.len -= 1;
            self.buf[self.len] = 0;
        }
    }
}

pub struct NetworkDnsApp {
    primary_in: InputField,
    secondary_in: InputField,
    fallback_in: InputField,
    query_in: InputField,
    modal_host_in: InputField,
    modal_ip_in: InputField,
    pub focus: DnsFocusField,
    pub add_host_modal: bool,
    pub entries: Vec<DnsTableEntry>,
    pub scroll_offset: usize,
    pub initialized: bool,
    status_buf: [u8; 80],
    status_len: usize,
    status_is_error: bool,
    last_probe_buf: [u8; 48],
    last_probe_len: usize,
    last_probe_ok: bool,
}

impl NetworkDnsApp {
    pub const fn new() -> Self {
        Self {
            primary_in: InputField::empty(),
            secondary_in: InputField::empty(),
            fallback_in: InputField::empty(),
            query_in: InputField::empty(),
            modal_host_in: InputField::empty(),
            modal_ip_in: InputField::empty(),
            focus: DnsFocusField::None,
            add_host_modal: false,
            entries: Vec::new(),
            scroll_offset: 0,
            initialized: false,
            status_buf: [0u8; 80],
            status_len: 0,
            status_is_error: false,
            last_probe_buf: [0u8; 48],
            last_probe_len: 0,
            last_probe_ok: true,
        }
    }

    pub fn is_text_focused(&self) -> bool {
        self.focus != DnsFocusField::None || self.add_host_modal
    }

    fn set_status(&mut self, msg: &str, is_error: bool) {
        self.status_buf = [0u8; 80];
        let l = msg.len().min(79);
        self.status_buf[..l].copy_from_slice(&msg.as_bytes()[..l]);
        self.status_len = l;
        self.status_is_error = is_error;
    }

    fn status_str(&self) -> &str {
        if self.status_len == 0 {
            "Pronto (/disk/hosts + Cache LRU Zero-Idle)"
        } else {
            core::str::from_utf8(&self.status_buf[..self.status_len]).unwrap_or("")
        }
    }

    fn set_probe_msg(&mut self, msg: &str, ok: bool) {
        self.last_probe_buf = [0u8; 48];
        let l = msg.len().min(47);
        self.last_probe_buf[..l].copy_from_slice(&msg.as_bytes()[..l]);
        self.last_probe_len = l;
        self.last_probe_ok = ok;
    }

    pub fn refresh(&mut self, sys: &dyn SystemHost) {
        let cfg = sys.dns_get_servers();
        if !self.initialized {
            self.query_in.set_str("example.com");
            self.initialized = true;
        }
        if self.focus != DnsFocusField::PrimaryIp {
            self.primary_in.set_ip(cfg.primary);
        }
        if self.focus != DnsFocusField::SecondaryIp {
            self.secondary_in.set_ip(cfg.secondary);
        }
        if self.focus != DnsFocusField::FallbackIp {
            self.fallback_in.set_ip(cfg.fallback);
        }

        if self.entries.len() < 64 {
            self.entries.resize(64, DnsTableEntry::empty());
        }
        let n = sys.dns_list_entries(&mut self.entries[..64]);
        self.entries.truncate(n);
        if self.scroll_offset >= n && n > 0 {
            self.scroll_offset = n.saturating_sub(1);
        } else if n == 0 {
            self.scroll_offset = 0;
        }
    }

    fn save_servers(&mut self, sys: &dyn SystemHost, notifs: &mut NotificationCenter, tick: u64) {
        let p = match parse_ipv4(self.primary_in.as_str()) {
            Some(ip) => ip,
            None => {
                self.set_status("IPv4 Primario invalido", true);
                return;
            }
        };
        let s = match parse_ipv4(self.secondary_in.as_str()) {
            Some(ip) => ip,
            None => {
                self.set_status("IPv4 Secundario invalido", true);
                return;
            }
        };
        let f = match parse_ipv4(self.fallback_in.as_str()) {
            Some(ip) => ip,
            None => {
                self.set_status("IPv4 Fallback invalido", true);
                return;
            }
        };
        match sys.dns_set_servers(DnsServersConfig { primary: p, secondary: s, fallback: f }) {
            Ok(()) => {
                self.focus = DnsFocusField::None;
                self.set_status("Servidores salvos em /disk/net.cfg", false);
                notifs.post(b'@', "DNS Config", "Servidores salvos em /disk/net.cfg", COLOR_EMERALD, 220, tick);
                self.refresh(sys);
            }
            Err(e) => self.set_status(e, true),
        }
    }

    fn run_probe(&mut self, sys: &dyn SystemHost, notifs: &mut NotificationCenter, tick: u64) {
        let host = self.query_in.as_str().trim();
        if host.is_empty() {
            self.set_status("Informe um dominio para testar", true);
            return;
        }
        match sys.dns_resolve_timed_ms(host) {
            Ok((ip, ms)) => {
                let mut msg = [0u8; 48];
                let mut pos = 0usize;
                pos += format_ip_into(&mut msg[pos..], ip);
                msg[pos..pos + 2].copy_from_slice(b" (");
                pos += 2;
                pos += format_num(ms as usize, &mut msg[pos..]);
                msg[pos..pos + 4].copy_from_slice(b" ms)");
                pos += 4;
                if let Ok(s) = core::str::from_utf8(&msg[..pos]) {
                    self.set_probe_msg(s, true);
                }
                self.set_status(if ms == 0 { "Cache HIT local (0 ms / 0 UDP)" } else { "Resolvido via upstream" }, false);
                notifs.post(b'@', "Teste DNS", if ms == 0 { "Cache Hit local (0 ms)" } else { "Resolucao DNS concluida" }, COLOR_CYAN_NEON, 200, tick);
                self.refresh(sys);
            }
            Err(e) => {
                self.set_probe_msg("Falha", false);
                self.set_status(e, true);
            }
        }
    }

    fn flush_cache(&mut self, sys: &dyn SystemHost, notifs: &mut NotificationCenter, tick: u64) {
        let cleared = sys.dns_flush_cache();
        let mut msg = [0u8; 64];
        let pfx = b"Cache limpo: ";
        msg[..pfx.len()].copy_from_slice(pfx);
        let mut pos = pfx.len();
        pos += format_num(cleared, &mut msg[pos..]);
        let sfx = b" removidos";
        msg[pos..pos + sfx.len()].copy_from_slice(sfx);
        pos += sfx.len();
        if let Ok(s) = core::str::from_utf8(&msg[..pos]) {
            self.set_status(s, false);
        }
        notifs.post(b'#', "Cache DNS", "Entradas dinamicas limpas", COLOR_AMBER, 200, tick);
        self.refresh(sys);
    }

    fn submit_modal_host(&mut self, sys: &dyn SystemHost, notifs: &mut NotificationCenter, tick: u64) {
        let host = self.modal_host_in.as_str().trim();
        let ip = match parse_ipv4(self.modal_ip_in.as_str().trim()) {
            Some(v) => v,
            None => {
                self.set_status("IPv4 invalido no dialogo + Host", true);
                return;
            }
        };
        if host.is_empty() {
            self.set_status("Informe o nome do host", true);
            return;
        }
        match sys.dns_add_host(host, ip) {
            Ok(()) => {
                self.add_host_modal = false;
                self.focus = DnsFocusField::None;
                self.set_status("Host gravado em /disk/hosts", false);
                notifs.post(b'+', "Hosts Estaticos", "Mapeamento salvo em /disk/hosts", COLOR_EMERALD, 220, tick);
                self.refresh(sys);
            }
            Err(e) => self.set_status(e, true),
        }
    }

    pub fn render(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        wx: usize,
        top_y: usize,
        ww: usize,
        body_h: usize,
        _tick: u64,
    ) {
        let cx = wx + 8;
        let cw = ww.saturating_sub(16);

        // ── Seção 1: Servidores DNS (/disk/net.cfg) ────────────────────────────
        fill_rect(buf, screen_w, screen_h, cx, top_y + 2, cw, 50, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, cx, top_y + 2, cw, 50, COLOR_BORDER);
        draw_string(buf, screen_w, screen_h, "Servidores DNS (/disk/net.cfg):", cx + 8, top_y + 7, COLOR_CYAN, Some(COLOR_CARD_ALT));
        draw_button(buf, screen_w, screen_h, cx + cw - 76, top_y + 5, 68, 17, "Salvar", ButtonStyle::Success, WidgetState::Normal);

        let ip_y = top_y + 26;
        draw_string_telemetry(buf, screen_w, screen_h, "1o:", cx + 8, ip_y + 5, COLOR_TEXT_SEC, Some(COLOR_CARD_ALT));
        draw_text_input(buf, screen_w, screen_h, cx + 32, ip_y, 120, 20, self.primary_in.as_str(), self.focus == DnsFocusField::PrimaryIp, self.primary_in.len);

        draw_string_telemetry(buf, screen_w, screen_h, "2o:", cx + 162, ip_y + 5, COLOR_TEXT_SEC, Some(COLOR_CARD_ALT));
        draw_text_input(buf, screen_w, screen_h, cx + 186, ip_y, 120, 20, self.secondary_in.as_str(), self.focus == DnsFocusField::SecondaryIp, self.secondary_in.len);

        draw_string_telemetry(buf, screen_w, screen_h, "Fbk:", cx + 314, ip_y + 5, COLOR_TEXT_SEC, Some(COLOR_CARD_ALT));
        draw_text_input(buf, screen_w, screen_h, cx + 346, ip_y, cw.saturating_sub(354), 20, self.fallback_in.as_str(), self.focus == DnsFocusField::FallbackIp, self.fallback_in.len);

        // ── Seção 2: Teste de Resolução e Controles ────────────────────────────
        let s2_y = top_y + 56;
        fill_rect(buf, screen_w, screen_h, cx, s2_y, cw, 56, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, cx, s2_y, cw, 56, COLOR_BORDER);

        draw_string_telemetry(buf, screen_w, screen_h, "Host:", cx + 8, s2_y + 9, COLOR_TEXT_PRI, Some(COLOR_CARD_ALT));
        draw_text_input(buf, screen_w, screen_h, cx + 48, s2_y + 4, 174, 20, self.query_in.as_str(), self.focus == DnsFocusField::QueryHost, self.query_in.len);
        draw_button(buf, screen_w, screen_h, cx + 228, s2_y + 4, 86, 20, "Testar Res.", ButtonStyle::Primary, WidgetState::Normal);

        if self.last_probe_len > 0 {
            let pmsg = core::str::from_utf8(&self.last_probe_buf[..self.last_probe_len]).unwrap_or("");
            let pcol = if self.last_probe_ok { COLOR_EMERALD } else { COLOR_RUBY };
            draw_string_telemetry(buf, screen_w, screen_h, pmsg, cx + 318, s2_y + 8, pcol, Some(COLOR_CARD_ALT));
        }

        let btn_row_y = s2_y + 29;
        draw_button(buf, screen_w, screen_h, cx + 8, btn_row_y, 148, 22, "+ Adicionar Host", ButtonStyle::Success, WidgetState::Normal);
        draw_button(buf, screen_w, screen_h, cx + 164, btn_row_y, 152, 22, "Limpar Cache DNS", ButtonStyle::Danger, WidgetState::Normal);
        draw_button(buf, screen_w, screen_h, cx + 324, btn_row_y, cw.saturating_sub(332), 22, "Atualizar Tabela", ButtonStyle::Secondary, WidgetState::Normal);

        // ── Seção 3: Tabela DNS Local (HOSTS vs CACHE) ─────────────────────────
        let tbl_y = s2_y + 60;
        let tbl_h = body_h.saturating_sub(60 + 56 + 26);
        fill_rect(buf, screen_w, screen_h, cx, tbl_y, cw, tbl_h, COLOR_CARD_ALT);
        draw_rect_outline(buf, screen_w, screen_h, cx, tbl_y, cw, tbl_h, COLOR_BORDER);

        fill_rect(buf, screen_w, screen_h, cx + 1, tbl_y + 1, cw - 2, 18, COLOR_SURFACE_L1);
        draw_string_telemetry(buf, screen_w, screen_h, "DOMINIO", cx + 8, tbl_y + 4, COLOR_CYAN, Some(COLOR_SURFACE_L1));
        draw_string_telemetry(buf, screen_w, screen_h, "IPv4", cx + 186, tbl_y + 4, COLOR_CYAN, Some(COLOR_SURFACE_L1));
        draw_string_telemetry(buf, screen_w, screen_h, "ORIG", cx + 314, tbl_y + 4, COLOR_CYAN, Some(COLOR_SURFACE_L1));
        draw_string_telemetry(buf, screen_w, screen_h, "TTL", cx + 368, tbl_y + 4, COLOR_CYAN, Some(COLOR_SURFACE_L1));
        draw_string_telemetry(buf, screen_w, screen_h, "HITS", cx + 422, tbl_y + 4, COLOR_CYAN, Some(COLOR_SURFACE_L1));

        draw_button(buf, screen_w, screen_h, cx + cw - 42, tbl_y + 2, 18, 15, "^", ButtonStyle::Secondary, WidgetState::Normal);
        draw_button(buf, screen_w, screen_h, cx + cw - 22, tbl_y + 2, 18, 15, "v", ButtonStyle::Secondary, WidgetState::Normal);

        let max_rows = (tbl_h.saturating_sub(24)) / 18;
        if self.entries.is_empty() {
            draw_string(buf, screen_w, screen_h, "(Nenhuma entrada ativa na tabela local)", cx + 16, tbl_y + 32, COLOR_TEXT_MUTED, Some(COLOR_CARD_ALT));
        } else {
            for row in 0..max_rows {
                let idx = self.scroll_offset + row;
                if idx >= self.entries.len() {
                    break;
                }
                let e = &self.entries[idx];
                let ry = tbl_y + 22 + row * 18;
                let row_bg = if row % 2 == 0 { COLOR_CARD_ALT } else { COLOR_SURFACE_L1 };
                fill_rect(buf, screen_w, screen_h, cx + 2, ry, cw - 4, 16, row_bg);

                let full_name = e.hostname_str();
                let disp_name = if full_name.len() > 21 { &full_name[..21] } else { full_name };
                draw_string_telemetry(buf, screen_w, screen_h, disp_name, cx + 8, ry + 3, COLOR_TEXT_PRI, Some(row_bg));

                let mut ip_b = [0u8; 16];
                let ip_l = format_ip_into(&mut ip_b, e.ipv4);
                let ip_s = core::str::from_utf8(&ip_b[..ip_l]).unwrap_or("");
                draw_string_telemetry(buf, screen_w, screen_h, ip_s, cx + 186, ry + 3, COLOR_CYAN, Some(row_bg));

                if e.is_static_host {
                    draw_string_telemetry(buf, screen_w, screen_h, "HOSTS", cx + 314, ry + 3, COLOR_EMERALD, Some(row_bg));
                    draw_string_telemetry(buf, screen_w, screen_h, "PERMA", cx + 368, ry + 3, COLOR_EMERALD, Some(row_bg));
                } else {
                    draw_string_telemetry(buf, screen_w, screen_h, "CACHE", cx + 314, ry + 3, COLOR_AMBER, Some(row_bg));
                    let mut ttl_b = [0u8; 16];
                    let mut tl = format_num(e.ttl_remaining_secs as usize, &mut ttl_b);
                    ttl_b[tl] = b's';
                    tl += 1;
                    let ttl_s = core::str::from_utf8(&ttl_b[..tl]).unwrap_or("");
                    draw_string_telemetry(buf, screen_w, screen_h, ttl_s, cx + 368, ry + 3, COLOR_TEXT_SEC, Some(row_bg));
                }

                let mut hit_b = [0u8; 12];
                let hl = format_num(e.hits as usize, &mut hit_b);
                let hit_s = core::str::from_utf8(&hit_b[..hl]).unwrap_or("0");
                let hcol = if e.hits > 0 { COLOR_EMERALD } else { COLOR_TEXT_MUTED };
                draw_string_telemetry(buf, screen_w, screen_h, hit_s, cx + 430, ry + 3, hcol, Some(row_bg));
            }
        }

        // ── Barra de Status Inferior ───────────────────────────────────────────
        let sb_y = top_y + body_h.saturating_sub(22);
        fill_rect(buf, screen_w, screen_h, wx + 2, sb_y, ww.saturating_sub(4), 20, COLOR_SURFACE_L1);
        let scol = if self.status_is_error { COLOR_RUBY } else { COLOR_EMERALD };
        draw_string_telemetry(buf, screen_w, screen_h, self.status_str(), cx + 4, sb_y + 5, scol, Some(COLOR_SURFACE_L1));

        // ── Modal: + Adicionar Host em /disk/hosts ────────────────────────────
        if self.add_host_modal {
            let mw = 340;
            let mh = 148;
            let mx = wx + (ww / 2).saturating_sub(mw / 2);
            let my = top_y + (body_h / 2).saturating_sub(mh / 2);

            fill_rect(buf, screen_w, screen_h, mx + 4, my + 4, mw, mh, 0x0005_0710);
            fill_rect(buf, screen_w, screen_h, mx, my, mw, mh, 0x001E_1B4B);
            draw_rect_outline(buf, screen_w, screen_h, mx, my, mw, mh, COLOR_CYAN_NEON);
            draw_string(buf, screen_w, screen_h, "Adicionar Host (/disk/hosts)", mx + 12, my + 10, COLOR_CYAN_NEON, Some(0x001E_1B4B));

            draw_string_telemetry(buf, screen_w, screen_h, "Dominio:", mx + 14, my + 38, COLOR_TEXT_PRI, Some(0x001E_1B4B));
            draw_text_input(buf, screen_w, screen_h, mx + 86, my + 32, mw - 102, 22, self.modal_host_in.as_str(), self.focus == DnsFocusField::ModalHost, self.modal_host_in.len);

            draw_string_telemetry(buf, screen_w, screen_h, "IPv4:", mx + 14, my + 68, COLOR_TEXT_PRI, Some(0x001E_1B4B));
            draw_text_input(buf, screen_w, screen_h, mx + 86, my + 62, mw - 102, 22, self.modal_ip_in.as_str(), self.focus == DnsFocusField::ModalIp, self.modal_ip_in.len);

            draw_button(buf, screen_w, screen_h, mx + 16, my + 102, 168, 28, "Gravar em /disk/hosts", ButtonStyle::Success, WidgetState::Normal);
            draw_button(buf, screen_w, screen_h, mx + 196, my + 102, 128, 28, "Cancelar", ButtonStyle::Secondary, WidgetState::Normal);
        }
    }

    pub fn handle_click(
        &mut self,
        mx: usize,
        my: usize,
        wx: usize,
        top_y: usize,
        ww: usize,
        body_h: usize,
        sys: &dyn SystemHost,
        notifs: &mut NotificationCenter,
        tick: u64,
    ) {
        if self.add_host_modal {
            let mw = 340;
            let mh = 148;
            let modal_x = wx + (ww / 2).saturating_sub(mw / 2);
            let modal_y = top_y + (body_h / 2).saturating_sub(mh / 2);

            if mx >= modal_x + 86 && mx <= modal_x + mw - 16 && my >= modal_y + 32 && my <= modal_y + 54 {
                self.focus = DnsFocusField::ModalHost;
            } else if mx >= modal_x + 86 && mx <= modal_x + mw - 16 && my >= modal_y + 62 && my <= modal_y + 84 {
                self.focus = DnsFocusField::ModalIp;
            } else if mx >= modal_x + 16 && mx <= modal_x + 184 && my >= modal_y + 102 && my <= modal_y + 130 {
                self.submit_modal_host(sys, notifs, tick);
            } else if mx >= modal_x + 196 && mx <= modal_x + 324 && my >= modal_y + 102 && my <= modal_y + 130 {
                self.add_host_modal = false;
                self.focus = DnsFocusField::None;
            }
            return;
        }

        let cx = wx + 8;
        let cw = ww.saturating_sub(16);

        // 1. Servidores DNS (/disk/net.cfg)
        if mx >= cx + cw - 76 && mx <= cx + cw - 8 && my >= top_y + 5 && my <= top_y + 22 {
            self.save_servers(sys, notifs, tick);
            return;
        }
        let ip_y = top_y + 26;
        if my >= ip_y && my <= ip_y + 20 {
            if mx >= cx + 32 && mx <= cx + 152 {
                self.focus = DnsFocusField::PrimaryIp;
                return;
            }
            if mx >= cx + 186 && mx <= cx + 306 {
                self.focus = DnsFocusField::SecondaryIp;
                return;
            }
            if mx >= cx + 346 && mx <= cx + cw - 8 {
                self.focus = DnsFocusField::FallbackIp;
                return;
            }
        }

        // 2. Testar Resolução & Botões de Ação
        let s2_y = top_y + 56;
        if mx >= cx + 48 && mx <= cx + 222 && my >= s2_y + 4 && my <= s2_y + 24 {
            self.focus = DnsFocusField::QueryHost;
            return;
        }
        if mx >= cx + 228 && mx <= cx + 314 && my >= s2_y + 4 && my <= s2_y + 24 {
            self.focus = DnsFocusField::None;
            self.run_probe(sys, notifs, tick);
            return;
        }
        let btn_row_y = s2_y + 29;
        if my >= btn_row_y && my <= btn_row_y + 22 {
            if mx >= cx + 8 && mx <= cx + 156 {
                self.add_host_modal = true;
                self.modal_host_in.set_str("meu.servico.local");
                self.modal_ip_in.set_str("127.0.0.1");
                self.focus = DnsFocusField::ModalHost;
                return;
            }
            if mx >= cx + 164 && mx <= cx + 316 {
                self.focus = DnsFocusField::None;
                self.flush_cache(sys, notifs, tick);
                return;
            }
            if mx >= cx + 324 && mx <= cx + cw - 8 {
                self.focus = DnsFocusField::None;
                self.refresh(sys);
                self.set_status("Tabela local DNS atualizada", false);
                return;
            }
        }

        // 3. Scroll da tabela
        let tbl_y = s2_y + 60;
        if my >= tbl_y && my <= tbl_y + 18 {
            if mx >= cx + cw - 42 && mx <= cx + cw - 24 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                return;
            }
            if mx >= cx + cw - 22 && mx <= cx + cw - 4 {
                if self.scroll_offset + 1 < self.entries.len() {
                    self.scroll_offset += 1;
                }
                return;
            }
        }

        self.focus = DnsFocusField::None;
    }

    pub fn handle_key(
        &mut self,
        key: u8,
        sys: &dyn SystemHost,
        notifs: &mut NotificationCenter,
        tick: u64,
    ) {
        if self.add_host_modal {
            match key {
                27 => {
                    self.add_host_modal = false;
                    self.focus = DnsFocusField::None;
                }
                9 => {
                    self.focus = if self.focus == DnsFocusField::ModalHost {
                        DnsFocusField::ModalIp
                    } else {
                        DnsFocusField::ModalHost
                    };
                }
                10 | 13 => self.submit_modal_host(sys, notifs, tick),
                8 | 127 => match self.focus {
                    DnsFocusField::ModalHost => self.modal_host_in.pop_byte(),
                    DnsFocusField::ModalIp => self.modal_ip_in.pop_byte(),
                    _ => {}
                },
                b if b >= 32 && b <= 126 => match self.focus {
                    DnsFocusField::ModalHost => self.modal_host_in.push_byte(b),
                    DnsFocusField::ModalIp => self.modal_ip_in.push_byte(b),
                    _ => {}
                },
                _ => {}
            }
            return;
        }

        if self.focus != DnsFocusField::None {
            match key {
                27 => self.focus = DnsFocusField::None,
                10 | 13 => match self.focus {
                    DnsFocusField::QueryHost => self.run_probe(sys, notifs, tick),
                    DnsFocusField::PrimaryIp | DnsFocusField::SecondaryIp | DnsFocusField::FallbackIp => {
                        self.save_servers(sys, notifs, tick);
                    }
                    _ => {}
                },
                8 | 127 => match self.focus {
                    DnsFocusField::PrimaryIp => self.primary_in.pop_byte(),
                    DnsFocusField::SecondaryIp => self.secondary_in.pop_byte(),
                    DnsFocusField::FallbackIp => self.fallback_in.pop_byte(),
                    DnsFocusField::QueryHost => self.query_in.pop_byte(),
                    _ => {}
                },
                b if b >= 32 && b <= 126 => match self.focus {
                    DnsFocusField::PrimaryIp => self.primary_in.push_byte(b),
                    DnsFocusField::SecondaryIp => self.secondary_in.push_byte(b),
                    DnsFocusField::FallbackIp => self.fallback_in.push_byte(b),
                    DnsFocusField::QueryHost => self.query_in.push_byte(b),
                    _ => {}
                },
                _ => {}
            }
            return;
        }

        match key {
            b'r' | b'R' => {
                self.refresh(sys);
                self.set_status("Tabela local DNS atualizada", false);
            }
            b't' | b'T' => self.run_probe(sys, notifs, tick),
            b'f' | b'F' => self.flush_cache(sys, notifs, tick),
            b'h' | b'H' => {
                self.add_host_modal = true;
                self.modal_host_in.set_str("meu.servico.local");
                self.modal_ip_in.set_str("127.0.0.1");
                self.focus = DnsFocusField::ModalHost;
            }
            b'j' => {
                if self.scroll_offset + 1 < self.entries.len() {
                    self.scroll_offset += 1;
                }
            }
            b'k' => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            _ => {}
        }
    }
}

fn format_ip_into(dst: &mut [u8], ip: [u8; 4]) -> usize {
    let mut pos = 0;
    for (i, &oct) in ip.iter().enumerate() {
        if i > 0 {
            dst[pos] = b'.';
            pos += 1;
        }
        pos += format_num(oct as usize, &mut dst[pos..]);
    }
    pos
}

fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let mut parts = [0u8; 4];
    let mut idx = 0usize;
    let mut val = 0u16;
    let mut digits = false;
    for &b in s.trim().as_bytes() {
        if b == b'.' {
            if !digits || val > 255 || idx >= 3 {
                return None;
            }
            parts[idx] = val as u8;
            idx += 1;
            val = 0;
            digits = false;
        } else if b.is_ascii_digit() {
            val = val * 10 + (b - b'0') as u16;
            digits = true;
        } else {
            return None;
        }
    }
    if !digits || val > 255 || idx != 3 {
        return None;
    }
    parts[3] = val as u8;
    Some(parts)
}
