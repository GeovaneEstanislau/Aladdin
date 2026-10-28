//! Aplicativo de Terminal Interativo (Shell Integrado)

use crate::compositor::*;
use crate::font::*;
use crate::contracts::process::ProcessHost;

pub const TERM_HISTORY_MAX: usize = 6;
pub const TERM_LINE_LEN: usize = 48;

#[derive(Clone, Copy)]
pub struct TerminalApp {
    pub history: [[u8; TERM_LINE_LEN]; TERM_HISTORY_MAX],
    pub history_lens: [usize; TERM_HISTORY_MAX],
    pub history_colors: [u32; TERM_HISTORY_MAX],
    pub history_count: usize,
    pub input: [u8; 36],
    pub input_len: usize,
}

impl TerminalApp {
    pub const fn new() -> Self {
        Self {
            history: [[0; TERM_LINE_LEN]; TERM_HISTORY_MAX],
            history_lens: [0; TERM_HISTORY_MAX],
            history_colors: [0; TERM_HISTORY_MAX],
            history_count: 0,
            input: [0; 36],
            input_len: 0,
        }
    }

    pub fn init_defaults(&mut self) {
        self.push_line("Jinn OS Terminal v0.0.3 (Ring 0 / Ring 3)", COLOR_CYAN);
        self.push_line("Digite 'help' para comandos ou clique com o mouse.", COLOR_TEXT_MUTED);
        self.push_line("Comandos: spawn, ps, uptime, ls, cat, reboot", 0x0060_A5FA);
    }

    pub fn push_line(&mut self, text: &str, color: u32) {
        if self.history_count >= TERM_HISTORY_MAX {
            for i in 0..(TERM_HISTORY_MAX - 1) {
                self.history[i] = self.history[i + 1];
                self.history_lens[i] = self.history_lens[i + 1];
                self.history_colors[i] = self.history_colors[i + 1];
            }
            self.history_count = TERM_HISTORY_MAX - 1;
        }

        let idx = self.history_count;
        let bytes = text.as_bytes();
        let copy_len = bytes.len().min(TERM_LINE_LEN);
        for j in 0..copy_len {
            self.history[idx][j] = bytes[j];
        }
        self.history_lens[idx] = copy_len;
        self.history_colors[idx] = color;
        self.history_count += 1;
    }

    pub fn handle_key(&mut self, ch: u8, proc_host: &mut dyn ProcessHost) {
        match ch {
            b'\n' | b'\r' => {
                let len = self.input_len;
                if len > 0 {
                    let mut input_copy = [0u8; 36];
                    let copy_len = len.min(36);
                    for i in 0..copy_len {
                        input_copy[i] = self.input[i];
                    }

                    // Ecoa o comando digitado no terminal
                    if let Ok(cmd_str) = core::str::from_utf8(&input_copy[..copy_len]) {
                        self.push_line(cmd_str, 0x00F8_FAFC);

                        // Trata comandos internos locais
                        if cmd_str == "clear" {
                            self.history_count = 0;
                        } else {
                            // Despacha para o host de processos / shell real do kernel
                            proc_host.execute_command(cmd_str, &mut |line, color| {
                                self.push_line(line, color);
                            });
                        }
                    }

                    self.input_len = 0;
                }
            }
            0x08 => { // Backspace
                if self.input_len > 0 {
                    self.input_len -= 1;
                }
            }
            0x20..=0x7E => { // Caracter imprimível
                if self.input_len < 35 {
                    self.input[self.input_len] = ch;
                    self.input_len += 1;
                }
            }
            _ => {}
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
        is_active: bool,
        ticks: u64,
        spawn_count: usize,
    ) {
        let term_box_x = win_x + 8;
        let term_box_y = win_y + 28;
        let term_box_w = win_w - 16;
        let term_box_h = win_h.saturating_sub(68);

        // Caixa escura do terminal
        fill_rect(buf, screen_w, screen_h, term_box_x, term_box_y, term_box_w, term_box_h, 0x000F_172A);
        draw_rect_outline(buf, screen_w, screen_h, term_box_x, term_box_y, term_box_w, term_box_h, 0x001E_293B);

        // Linhas de histórico
        for i in 0..self.history_count {
            let len = self.history_lens[i];
            if len > 0 {
                if let Ok(line_str) = core::str::from_utf8(&self.history[i][..len]) {
                    let line_y = term_box_y + 8 + (i * 18);
                    draw_string(buf, screen_w, screen_h, line_str, term_box_x + 8, line_y, self.history_colors[i], Some(0x000F_172A));
                }
            }
        }

        // Prompt de digitação
        let prompt_y = term_box_y + term_box_h - 22;
        draw_string(buf, screen_w, screen_h, "jinn> ", term_box_x + 8, prompt_y, COLOR_CYAN, Some(0x000F_172A));

        let input_len = self.input_len;
        if input_len > 0 {
            if let Ok(input_str) = core::str::from_utf8(&self.input[..input_len]) {
                draw_string(buf, screen_w, screen_h, input_str, term_box_x + 56, prompt_y, COLOR_TEXT_PRI, Some(0x000F_172A));
            }
        }

        // Cursor piscante quando a janela está ativa
        if is_active {
            let cursor_x = term_box_x + 56 + (input_len * FONT_WIDTH);
            if (ticks % 50) < 25 {
                draw_char(buf, screen_w, screen_h, b'_', cursor_x, prompt_y, COLOR_CYAN, Some(0x000F_172A));
            }
        }

        // Botão de atalho para execução de aplicativo
        let btn_y = win_y + win_h - 34;
        fill_rect(buf, screen_w, screen_h, win_x + 8, btn_y, 220, 24, COLOR_BTN_BG);
        draw_rect_outline(buf, screen_w, screen_h, win_x + 8, btn_y, 220, 24, 0x0093_C5FD);
        draw_string(buf, screen_w, screen_h, "+ Executar Jinn Browser", win_x + 14, btn_y + 4, COLOR_WHITE, Some(COLOR_BTN_BG));

        if spawn_count > 0 {
            draw_label_and_num(buf, screen_w, screen_h, "Apps: ", spawn_count, " PIDs", win_x + 236, btn_y + 6, COLOR_EMERALD, Some(COLOR_CARD));
        }
    }
}
