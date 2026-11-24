//! Renderização de Texto com Fonte Bitmap 8x8, Suporte a UTF-8 (Português),
//! 3 Escalas Tipográficas (16px Header, 14px Body, 11px Telemetry) e Antialiasing Real

use crate::compositor::ClipRect;
use crate::compositor::color::alpha_blend;

pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 8;

pub trait IntoChar: Copy {
    fn into_char(self) -> char;
}

impl IntoChar for char {
    #[inline]
    fn into_char(self) -> char { self }
}

impl IntoChar for u8 {
    #[inline]
    fn into_char(self) -> char { self as char }
}

pub fn get_glyph<C: IntoChar>(ch: C) -> &'static [u8; 8] {
    let c = ch.into_char();
    match c {
        ' '..='~' => {
            let index = (c as u32 - 32) as usize;
            &FONT_8X8[index]
        }
        'á' => &GLYPH_A_ACUTE_LOWER,
        'à' => &GLYPH_A_GRAVE_LOWER,
        'â' => &GLYPH_A_CIRC_LOWER,
        'ã' => &GLYPH_A_TILDE_LOWER,
        'ç' => &GLYPH_C_CEDIL_LOWER,
        'é' => &GLYPH_E_ACUTE_LOWER,
        'ê' => &GLYPH_E_CIRC_LOWER,
        'í' => &GLYPH_I_ACUTE_LOWER,
        'ó' => &GLYPH_O_ACUTE_LOWER,
        'ô' => &GLYPH_O_CIRC_LOWER,
        'õ' => &GLYPH_O_TILDE_LOWER,
        'ú' => &GLYPH_U_ACUTE_LOWER,

        'Á' => &GLYPH_A_ACUTE_UPPER,
        'À' => &GLYPH_A_GRAVE_UPPER,
        'Â' => &GLYPH_A_CIRC_UPPER,
        'Ã' => &GLYPH_A_TILDE_UPPER,
        'Ç' => &GLYPH_C_CEDIL_UPPER,
        'É' => &GLYPH_E_ACUTE_UPPER,
        'Ê' => &GLYPH_E_CIRC_UPPER,
        'Í' => &GLYPH_I_ACUTE_UPPER,
        'Ó' => &GLYPH_O_ACUTE_UPPER,
        'Ô' => &GLYPH_O_CIRC_UPPER,
        'Õ' => &GLYPH_O_TILDE_UPPER,
        'Ú' => &GLYPH_U_ACUTE_UPPER,

        '°' => &GLYPH_DEGREE,
        'º' => &GLYPH_ORDINAL_M,
        'ª' => &GLYPH_ORDINAL_F,
        '•' => &GLYPH_BULLET,
        '–' => &GLYPH_EN_DASH,
        '—' => &GLYPH_EM_DASH,
        '…' => &GLYPH_ELLIPSIS,

        _ => &FONT_8X8[0], // fallback para espaço
    }
}

#[inline]
pub fn get_glyph_u8(ch: u8) -> &'static [u8; 8] {
    get_glyph(ch)
}

pub fn draw_char<C: IntoChar>(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    ch: C,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) {
    draw_char_clipped(buf, screen_w, screen_h, ch, x, y, fg, bg, None);
}

pub fn draw_char_clipped<C: IntoChar>(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    ch: C,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
    clip: Option<&ClipRect>,
) {
    if x >= screen_w || y >= screen_h { return; }
    let glyph = get_glyph(ch);
    let max_y = (y + FONT_HEIGHT).min(screen_h);
    let max_x = (x + FONT_WIDTH).min(screen_w);

    for (y_offset, row_byte) in glyph.iter().enumerate() {
        let cy = y + y_offset;
        if cy >= max_y { break; }
        let row_ptr = cy * screen_w;

        for x_offset in 0..8 {
            let cx = x + x_offset;
            if cx >= max_x { break; }

            if let Some(c) = clip {
                if !c.contains_pixel(cx, cy) { continue; }
            }

            if (row_byte & (1 << (7 - x_offset))) != 0 {
                buf[row_ptr + cx] = fg;
            } else if let Some(bg_color) = bg {
                buf[row_ptr + cx] = bg_color;
            }
        }
    }
}

pub fn text_width(text: &str) -> usize {
    text.chars().count() * FONT_WIDTH
}

pub fn text_width_header(text: &str) -> usize {
    text.chars().count() * 14
}

pub fn text_width_body(text: &str) -> usize {
    text.chars().count() * 10
}

pub fn text_width_telemetry(text: &str) -> usize {
    text.chars().count() * FONT_WIDTH
}

pub fn draw_string(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) {
    draw_string_clipped(buf, screen_w, screen_h, text, x, y, fg, bg, None);
}

pub fn draw_string_clipped(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    mut x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
    clip: Option<&ClipRect>,
) {
    for ch in text.chars() {
        if x + FONT_WIDTH > screen_w { break; }
        draw_char_clipped(buf, screen_w, screen_h, ch, x, y, fg, bg, clip);
        x += FONT_WIDTH;
    }
}

// ── Tipografia Nítida com Suavização de Bordas (Antialiasing & Subpixel) ──────

#[inline]
fn is_glyph_bit_set(glyph: &[u8; 8], x: isize, y: isize) -> bool {
    if x < 0 || x >= 8 || y < 0 || y >= 8 {
        false
    } else {
        (glyph[y as usize] & (1 << (7 - x as usize))) != 0
    }
}

/// Renderiza um caractere com suavização de bordas (antialiasing subpixel por matriz de conectividade).
/// Elimina serrilhados e degraus visuais através de interpolação alpha das arestas com o fundo.
pub fn draw_char_antialiased<C: IntoChar>(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    ch: C,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
    clip: Option<&ClipRect>,
) {
    if x >= screen_w || y >= screen_h { return; }
    let glyph = get_glyph(ch);
    let max_y = (y + FONT_HEIGHT).min(screen_h);
    let max_x = (x + FONT_WIDTH).min(screen_w);

    for y_offset in 0..8 {
        let cy = y + y_offset;
        if cy >= max_y { break; }
        let row_ptr = cy * screen_w;

        for x_offset in 0..8 {
            let cx = x + x_offset;
            if cx >= max_x { break; }

            if let Some(c) = clip {
                if !c.contains_pixel(cx, cy) { continue; }
            }

            let xo = x_offset as isize;
            let yo = y_offset as isize;
            let center = is_glyph_bit_set(glyph, xo, yo);

            let alpha = if center {
                // Píxel pertencente ao traço do glifo: analisa vizinhança ortogonal
                let mut ortho = 0;
                if is_glyph_bit_set(glyph, xo - 1, yo) { ortho += 1; }
                if is_glyph_bit_set(glyph, xo + 1, yo) { ortho += 1; }
                if is_glyph_bit_set(glyph, xo, yo - 1) { ortho += 1; }
                if is_glyph_bit_set(glyph, xo, yo + 1) { ortho += 1; }

                match ortho {
                    4 => 255,
                    3 => 235,
                    2 => 210,
                    _ => 185,
                }
            } else {
                // Píxel de borda externa: halo de suavização
                let mut ortho = 0;
                if is_glyph_bit_set(glyph, xo - 1, yo) { ortho += 1; }
                if is_glyph_bit_set(glyph, xo + 1, yo) { ortho += 1; }
                if is_glyph_bit_set(glyph, xo, yo - 1) { ortho += 1; }
                if is_glyph_bit_set(glyph, xo, yo + 1) { ortho += 1; }

                let mut diag = 0;
                if is_glyph_bit_set(glyph, xo - 1, yo - 1) { diag += 1; }
                if is_glyph_bit_set(glyph, xo + 1, yo - 1) { diag += 1; }
                if is_glyph_bit_set(glyph, xo - 1, yo + 1) { diag += 1; }
                if is_glyph_bit_set(glyph, xo + 1, yo + 1) { diag += 1; }

                if ortho >= 2 {
                    85
                } else if ortho == 1 {
                    45
                } else if diag >= 1 {
                    25
                } else {
                    0
                }
            };

            if alpha > 0 {
                let current_bg = buf[row_ptr + cx];
                let base_color = bg.unwrap_or(current_bg);
                buf[row_ptr + cx] = crate::compositor::color::alpha_blend(base_color, fg, alpha);
            } else if let Some(bg_color) = bg {
                buf[row_ptr + cx] = bg_color;
            }
        }
    }
}

/// Renderiza texto em linha com antialiasing contínuo e suporte a UTF-8 completo
pub fn draw_string_antialiased(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) {
    draw_string_antialiased_clipped(buf, screen_w, screen_h, text, x, y, fg, bg, None);
}

/// Renderiza texto com antialiasing respeitando a janela de recorte (clipping)
pub fn draw_string_antialiased_clipped(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    mut x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
    clip: Option<&ClipRect>,
) {
    for ch in text.chars() {
        if x + FONT_WIDTH > screen_w { break; }
        draw_char_antialiased(buf, screen_w, screen_h, ch, x, y, fg, bg, clip);
        x += FONT_WIDTH;
    }
}

// ── 3 Escalas Tipográficas Bem Definidas ────────────────────────────────────────

/// Escala 1: Títulos de Janela e Cabeçalhos (16px–18px Semi-Bold com Antialiasing)
pub fn draw_string_header(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) {
    draw_string_header_clipped(buf, screen_w, screen_h, text, x, y, fg, bg, None);
}

pub fn draw_string_header_clipped(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    mut x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
    clip: Option<&ClipRect>,
) {
    let scale = 2;
    for ch in text.chars() {
        if x + 14 > screen_w { break; }
        let glyph = get_glyph(ch);

        for gy in 0..8 {
            let cy_base = y + gy * scale;
            if cy_base >= screen_h { break; }

            for gx in 0..8 {
                let cx_base = x + gx * scale;
                if cx_base >= screen_w { break; }

                let is_set = is_glyph_bit_set(glyph, gx as isize, gy as isize);
                if is_set {
                    // Preenchimento 2x2 do ponto principal + semi-bold horizontal
                    for dy in 0..scale {
                        let py = cy_base + dy;
                        if py >= screen_h { break; }
                        let r_start = py * screen_w;

                        for dx in 0..=scale {
                            let px = cx_base + dx;
                            if px >= screen_w { break; }

                            if let Some(c) = clip {
                                if !c.contains_pixel(px, py) { continue; }
                            }

                            let alpha = if dx < scale { 255 } else { 160 }; // semi-bold edge suavizado
                            let current_bg = buf[r_start + px];
                            let base_color = bg.unwrap_or(current_bg);
                            buf[r_start + px] = alpha_blend(base_color, fg, alpha);
                        }
                    }
                } else {
                    // Suavização externa das diagonais do glifo escalado 2x
                    let mut neighbors = 0;
                    if is_glyph_bit_set(glyph, gx as isize - 1, gy as isize) { neighbors += 1; }
                    if is_glyph_bit_set(glyph, gx as isize + 1, gy as isize) { neighbors += 1; }
                    if is_glyph_bit_set(glyph, gx as isize, gy as isize - 1) { neighbors += 1; }
                    if is_glyph_bit_set(glyph, gx as isize, gy as isize + 1) { neighbors += 1; }

                    if neighbors >= 2 {
                        for dy in 0..scale {
                            let py = cy_base + dy;
                            if py >= screen_h { break; }
                            let r_start = py * screen_w;

                            for dx in 0..scale {
                                let px = cx_base + dx;
                                if px >= screen_w { break; }

                                if let Some(c) = clip {
                                    if !c.contains_pixel(px, py) { continue; }
                                }

                                let current_bg = buf[r_start + px];
                                let base_color = bg.unwrap_or(current_bg);
                                buf[r_start + px] = alpha_blend(base_color, fg, 70);
                            }
                        }
                    } else if let Some(bg_col) = bg {
                        for dy in 0..scale {
                            let py = cy_base + dy;
                            if py >= screen_h { break; }
                            let r_start = py * screen_w;
                            for dx in 0..scale {
                                let px = cx_base + dx;
                                if px < screen_w {
                                    buf[r_start + px] = bg_col;
                                }
                            }
                        }
                    }
                }
            }
        }
        x += 14; // kerning proporcional elegante para cabeçalho
    }
}

/// Escala 2: Corpo de Interface e Menus (13px–14px Regular com Antialiasing Subpixel)
pub fn draw_string_body(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) {
    draw_string_body_clipped(buf, screen_w, screen_h, text, x, y, fg, bg, None);
}

pub fn draw_string_body_clipped(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
    clip: Option<&ClipRect>,
) {
    let mut cur_x = x;
    // Reamostragem contínua para 14px de altura com interpolação bilinear pura no_std
    const TARGET_H: usize = 14;
    const TARGET_W: usize = 10;

    for ch in text.chars() {
        if cur_x + TARGET_W > screen_w { break; }
        let glyph = get_glyph(ch);

        for py in 0..TARGET_H {
            let cy = y + py;
            if cy >= screen_h { break; }
            let row_ptr = cy * screen_w;

            // Mapeia py [0..14] para gy [0..8] em ponto fixo 256
            let gy_fp = (py * 8 * 256) / TARGET_H;
            let gy0 = (gy_fp / 256) as isize;
            let gy_frac = (gy_fp % 256) as u32;

            for px in 0..TARGET_W {
                let cx = cur_x + px;
                if cx >= screen_w { break; }

                if let Some(c) = clip {
                    if !c.contains_pixel(cx, cy) { continue; }
                }

                // Mapeia px [0..10] para gx [0..8] em ponto fixo 256
                let gx_fp = (px * 8 * 256) / TARGET_W;
                let gx0 = (gx_fp / 256) as isize;
                let gx_frac = (gx_fp % 256) as u32;

                // Amostragem bilinear dos 4 vizinhos no bitmap 8x8
                let b00 = if is_glyph_bit_set(glyph, gx0, gy0) { 255u32 } else { 0u32 };
                let b10 = if is_glyph_bit_set(glyph, gx0 + 1, gy0) { 255u32 } else { 0u32 };
                let b01 = if is_glyph_bit_set(glyph, gx0, gy0 + 1) { 255u32 } else { 0u32 };
                let b11 = if is_glyph_bit_set(glyph, gx0 + 1, gy0 + 1) { 255u32 } else { 0u32 };

                let top = (b00 * (256 - gx_frac) + b10 * gx_frac) / 256;
                let bot = (b01 * (256 - gx_frac) + b11 * gx_frac) / 256;
                let alpha_sample = (top * (256 - gy_frac) + bot * gy_frac) / 256;

                // Curva de contraste para manter caracteres nítidos e legíveis
                let alpha = if alpha_sample > 140 {
                    255
                } else if alpha_sample > 40 {
                    (alpha_sample * 240) / 140
                } else {
                    0
                } as u8;

                if alpha > 0 {
                    let current_bg = buf[row_ptr + cx];
                    let base_color = bg.unwrap_or(current_bg);
                    buf[row_ptr + cx] = alpha_blend(base_color, fg, alpha);
                } else if let Some(bg_color) = bg {
                    buf[row_ptr + cx] = bg_color;
                }
            }
        }
        cur_x += TARGET_W;
    }
}

/// Escala 3: Telemetria e Badges (11px Monospace Alinhado)
pub fn draw_string_telemetry(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) {
    // Alinhamento compacto 11px com antialiasing nítido
    draw_string_antialiased(buf, screen_w, screen_h, text, x, y + 1, fg, bg);
}

/// Renderiza um Badge / Pílula de Telemetria com Fundo Translúcido e Borda
pub fn draw_telemetry_badge(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: u32,
    border: u32,
) {
    let w = text.chars().count() * FONT_WIDTH + 12;
    let h = 18;
    crate::compositor::primitives::fill_rounded_rect(buf, screen_w, screen_h, x, y, w, h, 4, bg);
    crate::compositor::primitives::draw_rounded_outline(buf, screen_w, screen_h, x, y, w, h, 4, border);
    draw_string_telemetry(buf, screen_w, screen_h, text, x + 6, y + 2, fg, Some(bg));
}

/// Renderiza texto nítido escalonado 2x (16x16) com suavização de curvas para títulos e cabeçalhos
pub fn draw_string_antialiased_2x(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    text: &str,
    x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) {
    draw_string_header(buf, screen_w, screen_h, text, x, y, fg, bg);
}

pub fn draw_label_and_num(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    label: &str,
    num: usize,
    suffix: &str,
    mut x: usize,
    y: usize,
    fg: u32,
    bg: Option<u32>,
) -> usize {
    draw_string(buf, screen_w, screen_h, label, x, y, fg, bg);
    x += text_width(label);

    let mut num_buf = [0u8; 20];
    let n_len = crate::compositor::primitives::format_num(num, &mut num_buf);
    if let Ok(num_str) = core::str::from_utf8(&num_buf[..n_len]) {
        draw_string(buf, screen_w, screen_h, num_str, x, y, fg, bg);
        x += text_width(num_str);
    }

    draw_string(buf, screen_w, screen_h, suffix, x, y, fg, bg);
    x + text_width(suffix)
}

// ── Glifos Estendidos para Língua Portuguesa e Caracteres Tipográficos (8x8) ─
pub static GLYPH_A_ACUTE_LOWER: [u8; 8] = [0x0c, 0x18, 0x78, 0x0c, 0x7c, 0xcc, 0x76, 0x00]; // á
pub static GLYPH_A_GRAVE_LOWER: [u8; 8] = [0x18, 0x0c, 0x78, 0x0c, 0x7c, 0xcc, 0x76, 0x00]; // à
pub static GLYPH_A_CIRC_LOWER: [u8; 8] = [0x18, 0x24, 0x78, 0x0c, 0x7c, 0xcc, 0x76, 0x00];  // â
pub static GLYPH_A_TILDE_LOWER: [u8; 8] = [0x6c, 0x00, 0x78, 0x0c, 0x7c, 0xcc, 0x76, 0x00]; // ã
pub static GLYPH_C_CEDIL_LOWER: [u8; 8] = [0x00, 0x00, 0x7c, 0xc6, 0xc0, 0xc6, 0x7c, 0x18]; // ç
pub static GLYPH_E_ACUTE_LOWER: [u8; 8] = [0x0c, 0x18, 0x7c, 0xc6, 0xfe, 0xc0, 0x7c, 0x00]; // é
pub static GLYPH_E_CIRC_LOWER: [u8; 8] = [0x18, 0x24, 0x7c, 0xc6, 0xfe, 0xc0, 0x7c, 0x00];  // ê
pub static GLYPH_I_ACUTE_LOWER: [u8; 8] = [0x0c, 0x18, 0x38, 0x18, 0x18, 0x18, 0x3c, 0x00]; // í
pub static GLYPH_O_ACUTE_LOWER: [u8; 8] = [0x0c, 0x18, 0x7c, 0xc6, 0xc6, 0xc6, 0x7c, 0x00]; // ó
pub static GLYPH_O_CIRC_LOWER: [u8; 8] = [0x18, 0x24, 0x7c, 0xc6, 0xc6, 0xc6, 0x7c, 0x00];  // ô
pub static GLYPH_O_TILDE_LOWER: [u8; 8] = [0x6c, 0x00, 0x7c, 0xc6, 0xc6, 0xc6, 0x7c, 0x00]; // õ
pub static GLYPH_U_ACUTE_LOWER: [u8; 8] = [0x0c, 0x18, 0xcc, 0xcc, 0xcc, 0xcc, 0x76, 0x00]; // ú

pub static GLYPH_A_ACUTE_UPPER: [u8; 8] = [0x0c, 0x18, 0x3c, 0x66, 0x7e, 0x66, 0x66, 0x00]; // Á
pub static GLYPH_A_GRAVE_UPPER: [u8; 8] = [0x18, 0x0c, 0x3c, 0x66, 0x7e, 0x66, 0x66, 0x00]; // À
pub static GLYPH_A_CIRC_UPPER: [u8; 8] = [0x18, 0x24, 0x3c, 0x66, 0x7e, 0x66, 0x66, 0x00];  // Â
pub static GLYPH_A_TILDE_UPPER: [u8; 8] = [0x6c, 0x18, 0x3c, 0x66, 0x7e, 0x66, 0x66, 0x00]; // Ã
pub static GLYPH_C_CEDIL_UPPER: [u8; 8] = [0x3c, 0x66, 0xc0, 0xc0, 0xc0, 0x66, 0x3c, 0x18]; // Ç
pub static GLYPH_E_ACUTE_UPPER: [u8; 8] = [0x18, 0x7e, 0x62, 0x78, 0x60, 0x62, 0x7e, 0x00]; // É
pub static GLYPH_E_CIRC_UPPER: [u8; 8] = [0x24, 0x7e, 0x62, 0x78, 0x60, 0x62, 0x7e, 0x00];  // Ê
pub static GLYPH_I_ACUTE_UPPER: [u8; 8] = [0x18, 0x7e, 0x18, 0x18, 0x18, 0x18, 0x7e, 0x00]; // Í
pub static GLYPH_O_ACUTE_UPPER: [u8; 8] = [0x18, 0x3c, 0x66, 0xc3, 0xc3, 0x66, 0x3c, 0x00]; // Ó
pub static GLYPH_O_CIRC_UPPER: [u8; 8] = [0x24, 0x3c, 0x66, 0xc3, 0xc3, 0x66, 0x3c, 0x00];  // Ô
pub static GLYPH_O_TILDE_UPPER: [u8; 8] = [0x6c, 0x3c, 0x66, 0xc3, 0xc3, 0x66, 0x3c, 0x00]; // Õ
pub static GLYPH_U_ACUTE_UPPER: [u8; 8] = [0x18, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3c, 0x00]; // Ú

pub static GLYPH_DEGREE: [u8; 8] = [0x1c, 0x14, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00];        // °
pub static GLYPH_ORDINAL_M: [u8; 8] = [0x1c, 0x14, 0x1c, 0x00, 0x1c, 0x00, 0x00, 0x00];     // º
pub static GLYPH_ORDINAL_F: [u8; 8] = [0x18, 0x24, 0x3c, 0x24, 0x3c, 0x00, 0x00, 0x00];     // ª
pub static GLYPH_BULLET: [u8; 8] = [0x00, 0x00, 0x18, 0x3c, 0x3c, 0x18, 0x00, 0x00];        // •
pub static GLYPH_EN_DASH: [u8; 8] = [0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x00, 0x00];       // –
pub static GLYPH_EM_DASH: [u8; 8] = [0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00];       // —
pub static GLYPH_ELLIPSIS: [u8; 8] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x49, 0x00];      // …

// 95 caracteres ASCII imprimíveis (32 a 126)
pub static FONT_8X8: [[u8; 8]; 95] = [
    // 32 ' '
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 33 '!'
    [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00],
    // 34 '"'
    [0x66, 0x66, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 35 '#'
    [0x6c, 0x6c, 0xfe, 0x6c, 0xfe, 0x6c, 0x6c, 0x00],
    // 36 '$'
    [0x18, 0x3e, 0x60, 0x3c, 0x06, 0x7c, 0x18, 0x00],
    // 37 '%'
    [0x00, 0x63, 0x66, 0x0c, 0x18, 0x33, 0x63, 0x00],
    // 38 '&'
    [0x38, 0x6c, 0x38, 0x76, 0xdc, 0xcc, 0x76, 0x00],
    // 39 '\''
    [0x18, 0x18, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 40 '('
    [0x0c, 0x18, 0x30, 0x30, 0x30, 0x18, 0x0c, 0x00],
    // 41 ')'
    [0x30, 0x18, 0x0c, 0x0c, 0x0c, 0x18, 0x30, 0x00],
    // 42 '*'
    [0x00, 0x66, 0x3c, 0xff, 0x3c, 0x66, 0x00, 0x00],
    // 43 '+'
    [0x00, 0x18, 0x18, 0x7e, 0x18, 0x18, 0x00, 0x00],
    // 44 ','
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x30],
    // 45 '-'
    [0x00, 0x00, 0x00, 0x7e, 0x00, 0x00, 0x00, 0x00],
    // 46 '.'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00],
    // 47 '/'
    [0x06, 0x0c, 0x18, 0x30, 0x60, 0xc0, 0x80, 0x00],
    // 48 '0'
    [0x3c, 0x66, 0xc3, 0xdb, 0xe7, 0x66, 0x3c, 0x00],
    // 49 '1'
    [0x18, 0x38, 0x18, 0x18, 0x18, 0x18, 0x7e, 0x00],
    // 50 '2'
    [0x7c, 0xc6, 0x0e, 0x3c, 0x78, 0xe0, 0xfe, 0x00],
    // 51 '3'
    [0x7e, 0x0c, 0x18, 0x3c, 0x06, 0xc6, 0x7c, 0x00],
    // 52 '4'
    [0x0c, 0x1c, 0x3c, 0x6c, 0xcc, 0xfe, 0x0c, 0x00],
    // 53 '5'
    [0xfc, 0xc0, 0xf8, 0x0c, 0x06, 0xc6, 0x7c, 0x00],
    // 54 '6'
    [0x38, 0x60, 0xc0, 0xf8, 0xcc, 0xcc, 0x78, 0x00],
    // 55 '7'
    [0xfe, 0xc6, 0x0c, 0x18, 0x30, 0x30, 0x30, 0x00],
    // 56 '8'
    [0x78, 0xcc, 0xcc, 0x78, 0xcc, 0xcc, 0x78, 0x00],
    // 57 '9'
    [0x78, 0xcc, 0xcc, 0x7c, 0x0c, 0x18, 0x70, 0x00],
    // 58 ':'
    [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00],
    // 59 ';'
    [0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x30],
    // 60 '<'
    [0x0c, 0x18, 0x30, 0x60, 0x30, 0x18, 0x0c, 0x00],
    // 61 '='
    [0x00, 0x00, 0x7e, 0x00, 0x7e, 0x00, 0x00, 0x00],
    // 62 '>'
    [0x30, 0x18, 0x0c, 0x06, 0x0c, 0x18, 0x30, 0x00],
    // 63 '?'
    [0x3c, 0x66, 0x0c, 0x18, 0x18, 0x00, 0x18, 0x00],
    // 64 '@'
    [0x3e, 0x63, 0x7b, 0x7b, 0x7b, 0x03, 0x1e, 0x00],
    // 65 'A'
    [0x18, 0x3c, 0x66, 0x66, 0x7e, 0x66, 0x66, 0x00],
    // 66 'B'
    [0xfc, 0x66, 0x66, 0x7c, 0x66, 0x66, 0xfc, 0x00],
    // 67 'C'
    [0x3c, 0x66, 0xc0, 0xc0, 0xc0, 0x66, 0x3c, 0x00],
    // 68 'D'
    [0xf8, 0x6c, 0x66, 0x66, 0x66, 0x6c, 0xf8, 0x00],
    // 69 'E'
    [0xfe, 0x62, 0x68, 0x78, 0x68, 0x62, 0xfe, 0x00],
    // 70 'F'
    [0xfe, 0x62, 0x68, 0x78, 0x68, 0x60, 0xf0, 0x00],
    // 71 'G'
    [0x3c, 0x66, 0xc0, 0xc0, 0xce, 0x66, 0x3e, 0x00],
    // 72 'H'
    [0x66, 0x66, 0x66, 0x7e, 0x66, 0x66, 0x66, 0x00],
    // 73 'I'
    [0x7e, 0x18, 0x18, 0x18, 0x18, 0x18, 0x7e, 0x00],
    // 74 'J'
    [0x1e, 0x0c, 0x0c, 0x0c, 0xcc, 0xcc, 0x78, 0x00],
    // 75 'K'
    [0xe6, 0x66, 0x6c, 0x78, 0x6c, 0x66, 0xe6, 0x00],
    // 76 'L'
    [0xf0, 0x60, 0x60, 0x60, 0x62, 0x66, 0xfe, 0x00],
    // 77 'M'
    [0xc3, 0xe7, 0xff, 0xdb, 0xc3, 0xc3, 0xc3, 0x00],
    // 78 'N'
    [0xc6, 0xe6, 0xf6, 0xde, 0xce, 0xc6, 0xc6, 0x00],
    // 79 'O'
    [0x3c, 0x66, 0xc3, 0xc3, 0xc3, 0x66, 0x3c, 0x00],
    // 80 'P'
    [0xfc, 0x66, 0x66, 0x7c, 0x60, 0x60, 0xf0, 0x00],
    // 81 'Q'
    [0x78, 0xcc, 0xcc, 0xcc, 0xdc, 0x78, 0x1c, 0x00],
    // 82 'R'
    [0xfc, 0x66, 0x66, 0x7c, 0x6c, 0x66, 0xe6, 0x00],
    // 83 'S'
    [0x7c, 0xc6, 0xe0, 0x7c, 0x0e, 0xc6, 0x7c, 0x00],
    // 84 'T'
    [0x7e, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00],
    // 85 'U'
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x3c, 0x00],
    // 86 'V'
    [0x66, 0x66, 0x66, 0x66, 0x66, 0x3c, 0x18, 0x00],
    // 87 'W'
    [0xc3, 0xc3, 0xc3, 0xdb, 0xff, 0xe7, 0xc3, 0x00],
    // 88 'X'
    [0xc3, 0x66, 0x3c, 0x18, 0x3c, 0x66, 0xc3, 0x00],
    // 89 'Y'
    [0x66, 0x66, 0x66, 0x3c, 0x18, 0x18, 0x18, 0x00],
    // 90 'Z'
    [0xfe, 0x06, 0x0c, 0x18, 0x30, 0x60, 0xfe, 0x00],
    // 91 '['
    [0x3c, 0x30, 0x30, 0x30, 0x30, 0x30, 0x3c, 0x00],
    // 92 '\'
    [0xc0, 0x60, 0x30, 0x18, 0x0c, 0x06, 0x02, 0x00],
    // 93 ']'
    [0x3c, 0x0c, 0x0c, 0x0c, 0x0c, 0x0c, 0x3c, 0x00],
    // 94 '^'
    [0x10, 0x38, 0x6c, 0xc6, 0x00, 0x00, 0x00, 0x00],
    // 95 '_'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff],
    // 96 '`'
    [0x30, 0x18, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x00],
    // 97 'a'
    [0x00, 0x00, 0x78, 0x0c, 0x7c, 0xcc, 0x76, 0x00],
    // 98 'b'
    [0xe0, 0x60, 0x7c, 0x66, 0x66, 0x66, 0x7c, 0x00],
    // 99 'c'
    [0x00, 0x00, 0x7c, 0xc6, 0xc0, 0xc6, 0x7c, 0x00],
    // 100 'd'
    [0x1c, 0x0c, 0x7c, 0xcc, 0xcc, 0xcc, 0x76, 0x00],
    // 101 'e'
    [0x00, 0x00, 0x7c, 0xc6, 0xfe, 0xc0, 0x7c, 0x00],
    // 102 'f'
    [0x38, 0x6c, 0x60, 0xf0, 0x60, 0x60, 0xf0, 0x00],
    // 103 'g'
    [0x00, 0x00, 0x76, 0xcc, 0xcc, 0x7c, 0x0c, 0xf8],
    // 104 'h'
    [0xe0, 0x60, 0x6c, 0x76, 0x66, 0x66, 0xe6, 0x00],
    // 105 'i'
    [0x18, 0x00, 0x38, 0x18, 0x18, 0x18, 0x3c, 0x00],
    // 106 'j'
    [0x06, 0x00, 0x06, 0x06, 0x06, 0x66, 0x66, 0x3c],
    // 107 'k'
    [0xe0, 0x60, 0x66, 0x6c, 0x78, 0x6c, 0xe6, 0x00],
    // 108 'l'
    [0x38, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3c, 0x00],
    // 109 'm'
    [0x00, 0x00, 0xe6, 0xff, 0xdb, 0xc3, 0xc3, 0x00],
    // 110 'n'
    [0x00, 0x00, 0xdc, 0x66, 0x66, 0x66, 0x66, 0x00],
    // 111 'o'
    [0x00, 0x00, 0x7c, 0xc6, 0xc6, 0xc6, 0x7c, 0x00],
    // 112 'p'
    [0x00, 0x00, 0xdc, 0x66, 0x66, 0x7c, 0x60, 0xf0],
    // 113 'q'
    [0x00, 0x00, 0x76, 0xcc, 0xcc, 0x7c, 0x0c, 0x1e],
    // 114 'r'
    [0x00, 0x00, 0xdc, 0x76, 0x66, 0x60, 0xf0, 0x00],
    // 115 's'
    [0x00, 0x00, 0x7c, 0xc0, 0x7c, 0x06, 0xfc, 0x00],
    // 116 't'
    [0x30, 0x30, 0xfc, 0x30, 0x30, 0x34, 0x18, 0x00],
    // 117 'u'
    [0x00, 0x00, 0xcc, 0xcc, 0xcc, 0xcc, 0x76, 0x00],
    // 118 'v'
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3c, 0x18, 0x00],
    // 119 'w'
    [0x00, 0x00, 0xc3, 0xc3, 0xdb, 0x7e, 0x24, 0x00],
    // 120 'x'
    [0x00, 0x00, 0xc3, 0x66, 0x3c, 0x66, 0xc3, 0x00],
    // 121 'y'
    [0x00, 0x00, 0x66, 0x66, 0x66, 0x3c, 0x18, 0x70],
    // 122 'z'
    [0x00, 0x00, 0xfe, 0x0c, 0x38, 0x60, 0xfe, 0x00],
    // 123 '{'
    [0x0e, 0x18, 0x18, 0x70, 0x18, 0x18, 0x0e, 0x00],
    // 124 '|'
    [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00],
    // 125 '}'
    [0x70, 0x18, 0x18, 0x0e, 0x18, 0x18, 0x70, 0x00],
    // 126 '~'
    [0x76, 0xdc, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
];
