//! Aladdin Software & Update Manager — Gerenciador de Aplicativos e Atualizações Ring 3
//!
//! Implementa a arquitetura de pacotes e atualização "Offline-on-Shutdown":
//! - Permite navegação e inspeção de serviços e pacotes do ecossistema Jinn.
//! - Download/verificação em background com baixa prioridade (Zero-Idle).
//! - Instalação real com display apagado no desligamento (zero lentidão durante o uso).

use crate::compositor::*;
use crate::font::*;
use crate::contracts::system::SystemHost;

#[derive(Debug, Clone, Copy)]
pub struct PackageEntry {
    pub name: &'static str,
    pub category: &'static str,
    pub installed_ver: &'static str,
    pub target_ver: &'static str,
    pub is_staged: bool,
    pub size_kb: usize,
}

pub const PACKAGES: [PackageEntry; 4] = [
    PackageEntry {
        name: "jhcs-daemon (JHCS Heterogeneous Compute)",
        category: "Computacao Vetorial / SIMD AVX2",
        installed_ver: "v0.1.0",
        target_ver: "v0.2.1",
        is_staged: true,
        size_kb: 1420,
    },
    PackageEntry {
        name: "aladdin-server (Compositor Desktop)",
        category: "Interface Grafica Ring 3",
        installed_ver: "v7.1.0",
        target_ver: "v7.1.0",
        is_staged: false,
        size_kb: 2150,
    },
    PackageEntry {
        name: "jinn-browser (JBE Web Engine)",
        category: "Navegador Web / Renderizacao",
        installed_ver: "v1.1.0",
        target_ver: "v1.2.0",
        is_staged: true,
        size_kb: 3840,
    },
    PackageEntry {
        name: "jinn-microkernel (Core & Microcode)",
        category: "Nucleo & Drivers Bare-Metal",
        installed_ver: "v0.0.3",
        target_ver: "v0.0.4",
        is_staged: true,
        size_kb: 890,
    },
];

pub struct SoftwareManagerApp {
    pub selected_idx: usize,
    pub check_in_progress: bool,
    pub last_checked_tick: u64,
}

impl SoftwareManagerApp {
    pub const fn new() -> Self {
        Self {
            selected_idx: 0,
            check_in_progress: false,
            last_checked_tick: 0,
        }
    }

    pub fn render(
        &self,
        buf: &mut [u32],
        screen_w: usize,
        screen_h: usize,
        win_x: usize,
        win_y: usize,
        win_w: usize,
        win_h: usize,
        _sys: &dyn SystemHost,
    ) {
        if win_w < 380 || win_h < 260 { return; }

        let client_x = win_x + 12;
        let client_y = win_y + 32;
        let client_w = win_w.saturating_sub(24);

        // 1. Banner Informativo Superior (Estratégia Offline-on-Shutdown)
        fill_rounded_rect(buf, screen_w, screen_h, client_x, client_y, client_w, 42, 6, COLOR_SURFACE_L1);
        draw_rounded_outline(buf, screen_w, screen_h, client_x, client_y, client_w, 42, 6, COLOR_BORDER_SUBTLE);

        draw_char(buf, screen_w, screen_h, b'*', client_x + 12, client_y + 8, COLOR_EMERALD, Some(COLOR_SURFACE_L1));
        draw_string(buf, screen_w, screen_h, "Atualizacoes Preparadas para Desligamento Seguro", client_x + 24, client_y + 8, COLOR_TEXT_PRI, Some(COLOR_SURFACE_L1));
        draw_string(
            buf,
            screen_w,
            screen_h,
            "Pacotes baixados em 2o plano. Instalacao ocorre ao desligar c/ tela apagada.",
            client_x + 12,
            client_y + 24,
            COLOR_TEXT_MUTED,
            Some(COLOR_SURFACE_L1),
        );

        // 2. Lista de Pacotes e Componentes
        let list_y = client_y + 50;
        let card_h = 36;
        let card_spacing = 42;

        for (idx, pkg) in PACKAGES.iter().enumerate() {
            let row_y = list_y + (idx * card_spacing);
            if row_y + card_h > win_y + win_h - 40 { break; }

            let is_sel = idx == self.selected_idx;
            let bg = if is_sel { COLOR_SURFACE_L3 } else { COLOR_SURFACE_L2 };
            let border_c = if pkg.is_staged { COLOR_EMERALD } else { COLOR_BORDER_SUBTLE };

            fill_rounded_rect(buf, screen_w, screen_h, client_x, row_y, client_w, card_h, 4, bg);
            draw_rounded_outline(buf, screen_w, screen_h, client_x, row_y, client_w, card_h, 4, border_c);

            // Nome e Categoria
            draw_string(buf, screen_w, screen_h, pkg.name, client_x + 10, row_y + 6, COLOR_WHITE, Some(bg));
            draw_string(buf, screen_w, screen_h, pkg.category, client_x + 10, row_y + 20, COLOR_TEXT_MUTED, Some(bg));

            // Status Badge
            if pkg.is_staged {
                let badge_x = client_x + client_w.saturating_sub(160);
                draw_pill(buf, screen_w, screen_h, badge_x, row_y + 8, 148, 20, 0x0006_4E3B, COLOR_EMERALD);
                draw_string(buf, screen_w, screen_h, "Pronto p/ Shutdown", badge_x + 10, row_y + 12, COLOR_EMERALD, Some(0x0006_4E3B));
            } else {
                let badge_x = client_x + client_w.saturating_sub(110);
                draw_pill(buf, screen_w, screen_h, badge_x, row_y + 8, 98, 20, COLOR_SURFACE_L1, COLOR_BORDER_MUTED);
                draw_string(buf, screen_w, screen_h, "Atualizado", badge_x + 16, row_y + 12, COLOR_TEXT_SEC, Some(COLOR_SURFACE_L1));
            }
        }

        // 3. Rodapé com Instruções de Ação
        let footer_y = win_y + win_h - 32;
        draw_line(buf, screen_w, screen_h, client_x as isize, footer_y as isize, (client_x + client_w) as isize, footer_y as isize, COLOR_BORDER_SUBTLE);
        draw_string(
            buf,
            screen_w,
            screen_h,
            "Para aplicar com tela desligada: Menu Iniciar -> Desligar Sistema -> Confirmar.",
            client_x + 4,
            footer_y + 8,
            COLOR_CYAN_NEON,
            Some(COLOR_CARD),
        );
    }
}
