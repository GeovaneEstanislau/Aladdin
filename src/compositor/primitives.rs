//! Primitivas Gráficas 2D de Alta Performance com Suporte a Recorte (Clipping)
//! Suporta cantos arredondados (Rounded Corners) via máscaras inteiras sem alocação dinâmica.

use super::damage::ClipRect;
use super::color::*;
use crate::contracts::system::CapabilityMode;

#[inline]
pub fn draw_pixel(buf: &mut [u32], screen_w: usize, screen_h: usize, x: usize, y: usize, color: u32) {
    if x < screen_w && y < screen_h {
        buf[y * screen_w + x] = color;
    }
}

#[inline]
pub fn draw_pixel_clipped(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    color: u32,
    clip: Option<&ClipRect>,
) {
    if let Some(c) = clip {
        if !c.contains_pixel(x, y) { return; }
    }
    draw_pixel(buf, screen_w, screen_h, x, y, color);
}

pub fn fill_rect(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u32,
) {
    fill_rect_clipped(buf, screen_w, screen_h, x, y, width, height, color, None);
}

pub fn fill_rect_clipped(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u32,
    clip: Option<&ClipRect>,
) {
    if x >= screen_w || y >= screen_h || width == 0 || height == 0 { return; }

    let mut start_x = x;
    let mut start_y = y;
    let mut end_x = (x + width).min(screen_w);
    let mut end_y = (y + height).min(screen_h);

    if let Some(c) = clip {
        start_x = start_x.max(c.min_x);
        start_y = start_y.max(c.min_y);
        end_x = end_x.min(c.max_x);
        end_y = end_y.min(c.max_y);
    }

    if start_x >= end_x || start_y >= end_y { return; }
    let row_len = end_x - start_x;

    for cy in start_y..end_y {
        let row_start = cy * screen_w + start_x;
        for i in 0..row_len {
            buf[row_start + i] = color;
        }
    }
}

pub fn draw_line(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    color: u32,
) {
    draw_line_clipped(buf, screen_w, screen_h, x0, y0, x1, y1, color, None);
}

pub fn draw_line_clipped(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mut x0: isize,
    mut y0: isize,
    x1: isize,
    y1: isize,
    color: u32,
    clip: Option<&ClipRect>,
) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && (x0 as usize) < screen_w && y0 >= 0 && (y0 as usize) < screen_h {
            let px = x0 as usize;
            let py = y0 as usize;
            let mut draw = true;
            if let Some(c) = clip {
                draw = c.contains_pixel(px, py);
            }
            if draw {
                buf[py * screen_w + px] = color;
            }
        }
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

pub fn draw_rect_outline(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: u32,
) {
    if width == 0 || height == 0 { return; }
    let x_end = (x + width).saturating_sub(1) as isize;
    let y_end = (y + height).saturating_sub(1) as isize;
    let xi = x as isize;
    let yi = y as isize;

    draw_line(buf, screen_w, screen_h, xi, yi, x_end, yi, color);
    draw_line(buf, screen_w, screen_h, xi, y_end, x_end, y_end, color);
    draw_line(buf, screen_w, screen_h, xi, yi, xi, y_end, color);
    draw_line(buf, screen_w, screen_h, x_end, yi, x_end, y_end, color);
}

// ── Cantos Arredondados Inteiros (Zero Floating-Point) ─────────────────────────

/// Retorna a indentação horizontal (em pixels) para a linha `dy` de um quarto de círculo de raio `r`.
#[inline]
pub fn corner_cut(r: usize, dy: usize) -> usize {
    if dy >= r || r == 0 { return 0; }
    match r {
        1..=2 => if dy == 0 { 1 } else { 0 },
        3..=4 => match dy { 0 => 2, 1 => 1, _ => 0 },
        5..=6 => match dy { 0 => 3, 1 => 2, 2 => 1, _ => 0 },
        7..=8 => match dy { 0 => 4, 1 => 3, 2 => 2, 3 => 1, _ => 0 },
        9..=10 => match dy { 0 => 6, 1 => 4, 2 => 3, 3 => 2, 4 => 1, _ => 0 },
        11..=12 => match dy { 0 => 7, 1 => 5, 2 => 4, 3 => 3, 4 => 2, 5 => 1, _ => 0 },
        _ => {
            let r_sq = (r * r) as isize;
            let dy_from_center = (r.saturating_sub(1).saturating_sub(dy)) as isize;
            let dy_sq = dy_from_center * dy_from_center;
            if dy_sq >= r_sq {
                r
            } else {
                let dx_max = isqrt((r_sq - dy_sq) as usize);
                r.saturating_sub(dx_max)
            }
        }
    }
}

/// Raiz quadrada inteira (Método de Newton) para cálculo de raios dinâmicos
#[inline]
pub fn isqrt(val: usize) -> usize {
    if val == 0 { return 0; }
    let mut x0 = val / 2;
    if x0 == 0 { return 1; }
    let mut x1 = (x0 + val / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + val / x0) / 2;
    }
    x0
}

/// Seno inteiro para interpolação e trigonometria em no_std
/// Retorna valor entre -256 e +256 (escalado por 256) para qualquer ângulo em graus.
pub fn isin(deg: i32) -> i32 {
    let mut d = deg % 360;
    if d < 0 { d += 360; }
    // Tabela de quadrante de 0 a 90 graus com 16 passos
    const SINTAB: [i32; 16] = [
        0, 27, 53, 79, 104, 128, 150, 171,
        190, 207, 222, 235, 245, 252, 255, 256
    ];
    let (quad, rem) = (d / 90, d % 90);
    let idx = (rem * 15) / 90;
    match quad {
        0 => SINTAB[idx as usize],
        1 => SINTAB[15 - idx as usize],
        2 => -SINTAB[idx as usize],
        3 => -SINTAB[15 - idx as usize],
        _ => 0,
    }
}

/// Cosseno inteiro (-256 a +256)
#[inline]
pub fn icos(deg: i32) -> i32 {
    isin(deg + 90)
}

/// Preenche um retângulo com cantos arredondados (4 cantos simétricos)
pub fn fill_rounded_rect(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    color: u32,
) {
    if x >= screen_w || y >= screen_h || width == 0 || height == 0 { return; }
    let r = radius.min(width / 2).min(height / 2);
    if r <= 1 {
        fill_rect(buf, screen_w, screen_h, x, y, width, height, color);
        return;
    }

    for row_idx in 0..height {
        let cur_y = y + row_idx;
        if cur_y >= screen_h { break; }

        let cut = if row_idx < r {
            corner_cut(r, row_idx)
        } else if row_idx >= height - r {
            corner_cut(r, height - 1 - row_idx)
        } else {
            0
        };

        let start_x = x + cut;
        let end_x = (x + width.saturating_sub(cut)).min(screen_w);
        if start_x < end_x {
            let row_start = cur_y * screen_w + start_x;
            let count = end_x - start_x;
            for i in 0..count {
                buf[row_start + i] = color;
            }
        }
    }
}

/// Desenha o contorno suave de um retângulo com cantos arredondados
pub fn draw_rounded_outline(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    color: u32,
) {
    if width == 0 || height == 0 { return; }
    let r = radius.min(width / 2).min(height / 2);
    if r <= 1 {
        draw_rect_outline(buf, screen_w, screen_h, x, y, width, height, color);
        return;
    }

    // Aresta horizontal superior
    let top_cut = corner_cut(r, 0);
    let top_x0 = x + top_cut;
    let top_x1 = x + width.saturating_sub(top_cut + 1);
    if top_x0 <= top_x1 && y < screen_h {
        let row_start = y * screen_w;
        for px in top_x0..=top_x1.min(screen_w.saturating_sub(1)) {
            buf[row_start + px] = color;
        }
    }

    // Aresta horizontal inferior
    let bot_y = y + height - 1;
    if top_x0 <= top_x1 && bot_y < screen_h {
        let row_start = bot_y * screen_w;
        for px in top_x0..=top_x1.min(screen_w.saturating_sub(1)) {
            buf[row_start + px] = color;
        }
    }

    // Arestas verticais retas (esquerda e direita)
    let mid_y0 = y + r;
    let mid_y1 = y + height.saturating_sub(r + 1);
    let right_x = x + width - 1;
    for cur_y in mid_y0..=mid_y1 {
        if cur_y < screen_h {
            if x < screen_w { buf[cur_y * screen_w + x] = color; }
            if right_x < screen_w { buf[cur_y * screen_w + right_x] = color; }
        }
    }

    // Cantos arredondados com suavização contínua de degraus
    for dy in 0..r {
        let cut = corner_cut(r, dy);
        let next_cut = if dy + 1 < r { corner_cut(r, dy + 1) } else { 0 };
        let step_len = cut.saturating_sub(next_cut).max(1);

        // Cantos superiores
        let top_row = y + dy;
        if top_row < screen_h {
            // Superior esquerdo
            for sx in 0..step_len {
                let px = x + cut.saturating_sub(sx);
                if px < screen_w { buf[top_row * screen_w + px] = color; }
            }
            // Superior direito
            for sx in 0..step_len {
                let px = x + width - 1 - cut.saturating_sub(sx);
                if px < screen_w { buf[top_row * screen_w + px] = color; }
            }
        }

        // Cantos inferiores
        let bot_row = y + height - 1 - dy;
        if bot_row < screen_h {
            // Inferior esquerdo
            for sx in 0..step_len {
                let px = x + cut.saturating_sub(sx);
                if px < screen_w { buf[bot_row * screen_w + px] = color; }
            }
            // Inferior direito
            for sx in 0..step_len {
                let px = x + width - 1 - cut.saturating_sub(sx);
                if px < screen_w { buf[bot_row * screen_w + px] = color; }
            }
        }
    }
}

/// Preenche retângulo com cantos superiores arredondados e base plana (para barras de título de janelas)
pub fn fill_rounded_top(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    color: u32,
) {
    if x >= screen_w || y >= screen_h || width == 0 || height == 0 { return; }
    let r = radius.min(width / 2).min(height);
    if r <= 1 {
        fill_rect(buf, screen_w, screen_h, x, y, width, height, color);
        return;
    }

    for row_idx in 0..height {
        let cur_y = y + row_idx;
        if cur_y >= screen_h { break; }

        let cut = if row_idx < r { corner_cut(r, row_idx) } else { 0 };
        let start_x = x + cut;
        let end_x = (x + width.saturating_sub(cut)).min(screen_w);
        if start_x < end_x {
            let row_start = cur_y * screen_w + start_x;
            let count = end_x - start_x;
            for i in 0..count {
                buf[row_start + i] = color;
            }
        }
    }
}

/// Desenha contorno com cantos superiores arredondados e base reta
pub fn draw_rounded_top_outline(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    color: u32,
) {
    if width == 0 || height == 0 { return; }
    let r = radius.min(width / 2).min(height);
    if r <= 1 {
        draw_rect_outline(buf, screen_w, screen_h, x, y, width, height, color);
        return;
    }

    // Aresta horizontal superior
    let top_cut = corner_cut(r, 0);
    let top_x0 = x + top_cut;
    let top_x1 = x + width.saturating_sub(top_cut + 1);
    if top_x0 <= top_x1 && y < screen_h {
        let row_start = y * screen_w;
        for px in top_x0..=top_x1.min(screen_w.saturating_sub(1)) {
            buf[row_start + px] = color;
        }
    }

    // Arestas verticais até a base
    let right_x = x + width - 1;
    for cur_y in (y + r)..(y + height) {
        if cur_y < screen_h {
            if x < screen_w { buf[cur_y * screen_w + x] = color; }
            if right_x < screen_w { buf[cur_y * screen_w + right_x] = color; }
        }
    }

    // Linha inferior de fechamento
    let bot_y = y + height.saturating_sub(1);
    if bot_y < screen_h {
        let row_start = bot_y * screen_w;
        for px in x..=(x + width - 1).min(screen_w.saturating_sub(1)) {
            buf[row_start + px] = color;
        }
    }

    // Cantos superiores
    for dy in 0..r {
        let cut = corner_cut(r, dy);
        let next_cut = if dy + 1 < r { corner_cut(r, dy + 1) } else { 0 };
        let step_len = cut.saturating_sub(next_cut).max(1);

        let top_row = y + dy;
        if top_row < screen_h {
            // Superior esquerdo
            for sx in 0..step_len {
                let px = x + cut.saturating_sub(sx);
                if px < screen_w { buf[top_row * screen_w + px] = color; }
            }
            // Superior direito
            for sx in 0..step_len {
                let px = x + width - 1 - cut.saturating_sub(sx);
                if px < screen_w { buf[top_row * screen_w + px] = color; }
            }
        }
    }
}

pub fn draw_progress_bar(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    used: usize,
    total: usize,
    fg: u32,
    bg: u32,
    border: u32,
) {
    fill_rounded_rect(buf, screen_w, screen_h, x, y, width, height, 4, bg);
    draw_rounded_outline(buf, screen_w, screen_h, x, y, width, height, 4, border);

    if total > 0 && used > 0 && width > 4 {
        let max_w = width - 4;
        let fill_w = ((max_w as u128 * used.min(total) as u128) / total as u128) as usize;
        if fill_w > 0 {
            fill_rounded_rect(buf, screen_w, screen_h, x + 2, y + 2, fill_w, height.saturating_sub(4), 3, fg);
        }
    }
}

pub fn format_num(mut val: usize, buf: &mut [u8]) -> usize {
    if val == 0 {
        buf[0] = b'0';
        return 1;
    }
    let mut temp = [0u8; 20];
    let mut i = 0;
    while val > 0 {
        temp[i] = b'0' + ((val % 10) as u8);
        val /= 10;
        i += 1;
    }
    for j in 0..i {
        buf[j] = temp[i - 1 - j];
    }
    i
}

/// Desenha contêiner tipo 'Pill' com cápsula curva e borda sutil (inspirado no Wingpanel do elementary OS)
pub fn draw_pill(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    bg_color: u32,
    border_color: u32,
) {
    let radius = (height / 2).min(8);
    fill_rounded_rect(buf, screen_w, screen_h, x, y, width, height, radius, bg_color);
    draw_rounded_outline(buf, screen_w, screen_h, x, y, width, height, radius, border_color);
}

/// Projeta sombra suave para elevação de janelas e painéis
pub fn draw_drop_shadow(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    offset: usize,
    shadow_color: u32,
) {
    let radius = 6.min(width / 4).min(height / 4);
    fill_rounded_rect(buf, screen_w, screen_h, x + offset, y + offset, width, height, radius, shadow_color);
}

/// Projeta sombra com raio explícito de arredondamento
pub fn draw_drop_shadow_rounded(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    offset: usize,
    shadow_color: u32,
) {
    fill_rounded_rect(buf, screen_w, screen_h, x + offset, y + offset, width, height, radius, shadow_color);
}

// ── Efeitos de Acrílico e Desfoque Real (Glassmorphism & Real Blur) ──────────

/// Aplica um desfoque rápido (Fast 2-Pass Separable Box Blur) em uma sub-região retangular do buffer.
/// Utiliza janela deslizante (sliding window moving average) com complexidade O(N) por pixel.
pub fn box_blur_rect(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
) {
    if radius == 0 || width <= 1 || height <= 1 || x >= screen_w || y >= screen_h {
        return;
    }
    let x1 = (x + width).min(screen_w);
    let y1 = (y + height).min(screen_h);
    let w = x1 - x;
    let h = y1 - y;
    let r = radius.min(w / 2).min(h / 2);
    if r == 0 {
        return;
    }

    let mut scratch = alloc::vec![0u32; w * h];
    let window_size = (2 * r + 1) as u32;

    // ── Passo 1: Blur Horizontal (de `buf` para `scratch`) ──
    for ry in 0..h {
        let buf_row_start = (y + ry) * screen_w + x;
        let scratch_row_start = ry * w;

        let mut sum_r: u32 = 0;
        let mut sum_g: u32 = 0;
        let mut sum_b: u32 = 0;

        for dx in (-(r as isize))..=(r as isize) {
            let sample_col = if dx < 0 { 0 } else { (dx as usize).min(w - 1) };
            let pixel = buf[buf_row_start + sample_col];
            sum_r += (pixel >> 16) & 0xFF;
            sum_g += (pixel >> 8) & 0xFF;
            sum_b += pixel & 0xFF;
        }

        scratch[scratch_row_start] = ((sum_r / window_size) << 16)
            | ((sum_g / window_size) << 8)
            | (sum_b / window_size);

        for col in 1..w {
            let left_col = if col > r { col - r - 1 } else { 0 };
            let right_col = (col + r).min(w - 1);

            let p_left = buf[buf_row_start + left_col];
            let p_right = buf[buf_row_start + right_col];

            sum_r = sum_r.saturating_add((p_right >> 16) & 0xFF).saturating_sub((p_left >> 16) & 0xFF);
            sum_g = sum_g.saturating_add((p_right >> 8) & 0xFF).saturating_sub((p_left >> 8) & 0xFF);
            sum_b = sum_b.saturating_add(p_right & 0xFF).saturating_sub(p_left & 0xFF);

            scratch[scratch_row_start + col] = ((sum_r / window_size) << 16)
                | ((sum_g / window_size) << 8)
                | (sum_b / window_size);
        }
    }

    // ── Passo 2: Blur Vertical (de `scratch` de volta para `buf`) ──
    for rx in 0..w {
        let mut sum_r: u32 = 0;
        let mut sum_g: u32 = 0;
        let mut sum_b: u32 = 0;

        for dy in (-(r as isize))..=(r as isize) {
            let sample_row = if dy < 0 { 0 } else { (dy as usize).min(h - 1) };
            let pixel = scratch[sample_row * w + rx];
            sum_r += (pixel >> 16) & 0xFF;
            sum_g += (pixel >> 8) & 0xFF;
            sum_b += pixel & 0xFF;
        }

        buf[y * screen_w + (x + rx)] = ((sum_r / window_size) << 16)
            | ((sum_g / window_size) << 8)
            | (sum_b / window_size);

        for row in 1..h {
            let top_row = if row > r { row - r - 1 } else { 0 };
            let bot_row = (row + r).min(h - 1);

            let p_top = scratch[top_row * w + rx];
            let p_bot = scratch[bot_row * w + rx];

            sum_r = sum_r.saturating_add((p_bot >> 16) & 0xFF).saturating_sub((p_top >> 16) & 0xFF);
            sum_g = sum_g.saturating_add((p_bot >> 8) & 0xFF).saturating_sub((p_top >> 8) & 0xFF);
            sum_b = sum_b.saturating_add(p_bot & 0xFF).saturating_sub(p_top & 0xFF);

            buf[(y + row) * screen_w + (x + rx)] = ((sum_r / window_size) << 16)
                | ((sum_g / window_size) << 8)
                | (sum_b / window_size);
        }
    }
}

/// Preenche uma superfície acrílica com efeito de vidro fosco (Frosted Glass / Glassmorphism)
/// Aplica desfoque real no fundo subjacente e mescla com a tonalidade acrílica e cantos arredondados.
pub fn fill_acrylic_glass(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    tint_color: u32,
    tint_alpha: u8,
) {
    if x >= screen_w || y >= screen_h || width == 0 || height == 0 { return; }

    // 1. Aplica o blur no conteúdo real já desenhado no buffer (wallpaper / janelas inferiores)
    let blur_r = 4.min(width / 4).min(height / 4).max(1);
    box_blur_rect(buf, screen_w, screen_h, x, y, width, height, blur_r);

    // 2. Aplica a tonalidade acrílica respeitando os cantos arredondados
    let r = radius.min(width / 2).min(height / 2);
    for row_idx in 0..height {
        let cur_y = y + row_idx;
        if cur_y >= screen_h { break; }

        let cut = if r > 1 {
            if row_idx < r {
                corner_cut(r, row_idx)
            } else if row_idx >= height - r {
                corner_cut(r, height - 1 - row_idx)
            } else {
                0
            }
        } else {
            0
        };

        let start_x = x + cut;
        let end_x = (x + width.saturating_sub(cut)).min(screen_w);
        if start_x < end_x {
            let row_start = cur_y * screen_w;
            for px in start_x..end_x {
                let blurred = buf[row_start + px];
                buf[row_start + px] = alpha_blend(blurred, tint_color, tint_alpha);
            }
        }
    }
}

/// Renderiza uma superfície acrílica adaptativa baseada no CapabilityMode do sistema:
/// - Basic: Preenchimento plano sólido de altíssima velocidade (0% blur overhead)
/// - Balanced / Advanced: Frosted Glass com blur real de alta fidelidade visual
pub fn fill_acrylic_surface(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    radius: usize,
    tint_color: u32,
    tint_alpha: u8,
    mode: CapabilityMode,
) {
    match mode {
        CapabilityMode::Basic => {
            if radius > 1 {
                fill_rounded_rect(buf, screen_w, screen_h, x, y, width, height, radius, tint_color);
            } else {
                fill_rect(buf, screen_w, screen_h, x, y, width, height, tint_color);
            }
        }
        CapabilityMode::Balanced | CapabilityMode::Advanced => {
            fill_acrylic_glass(buf, screen_w, screen_h, x, y, width, height, radius, tint_color, tint_alpha);
        }
    }
}

/// Renderiza wallpaper procedural, imagem de domínio público ou vídeo animado selecionável.
/// wallpaper_id:
///   0 = Cyber-Aurora (Padrão: aurora boreal fluida e constelações)
///   1 = Midnight Minimalist (Azul noite profundo e microestrelas discretas)
///   2 = Techno Grid (Horizonte synthwave com grade de perspectiva)
///   3 = Solaris Horizon (Crepúsculo solar quente com corona e raios de horizonte)
///   4 = Obsidian Glass (Ardósia de obsidiana profunda com trilhas de circuito digital)
///   5 = NASA Cosmic Nebula (Domínio Público / CC0: Pilares da Criação e poeira estelar)
///   6 = Mount Fuji Sunset (Domínio Público / CC0: Horizonte crepuscular zen e silhueta montanhosa)
///   7 = Deep Space Webb (Domínio Público / CC0: Campo ultra-profundo cósmico com lentes gravitacionais)
///   8 = Matrix Digital Rain (Vídeo / Motion: Cascata contínua de código digital verde esmeralda)
///   9 = Starfield Warp 3D (Vídeo / Motion: Salto relativístico através do cosmos estrelado)
///   10 = Cyber Synthwave Pulse (Vídeo / Motion: Pôr do sol pulsante e grade de fuga em movimento)
///   11 = Aurora Wavefield (Vídeo / Motion: Cortinas de plasma e ondas de luz em tempo real)
pub fn render_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mode: CapabilityMode,
    wallpaper_id: u8,
    tick: u64,
) {
    if screen_w == 0 || screen_h == 0 { return; }

    match wallpaper_id {
        1 => {
            // 1. Midnight Minimalist
            for y in 0..screen_h {
                let factor = (y * 256) / screen_h;
                let r = 5 + ((8 * factor) / 256);
                let g = 8 + ((12 * factor) / 256);
                let b = 16 + ((22 * factor) / 256);
                let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                let row_start = y * screen_w;
                for x in 0..screen_w {
                    buf[row_start + x] = row_color;
                }
            }
            let stars: [(usize, usize, u32); 10] = [
                (screen_w / 7, 90, 0x0047_5569),
                (screen_w / 4, 150, 0x0064_748B),
                (screen_w / 2, 80, 0x0094_A3B8),
                ((screen_w * 3) / 4, 110, 0x0064_748B),
                ((screen_w * 6) / 7, 180, 0x0047_5569),
                (screen_w / 5, 280, 0x0033_4155),
                (screen_w / 2 + 50, 240, 0x0047_5569),
                ((screen_w * 4) / 5, 270, 0x0033_4155),
                (screen_w / 3, 330, 0x0033_4155),
                ((screen_w * 2) / 3, 310, 0x0047_5569),
            ];
            for &(sx, sy, c) in &stars {
                if sx < screen_w && sy < screen_h {
                    draw_pixel(buf, screen_w, screen_h, sx, sy, c);
                }
            }
        }
        2 => {
            // 2. Techno Grid
            let horizon_y = (screen_h * 55) / 100;
            for y in 0..screen_h {
                let row_start = y * screen_w;
                if y < horizon_y {
                    let factor = (y * 256) / horizon_y;
                    let r = 8 + ((18 * factor) / 256);
                    let g = 10 + ((14 * factor) / 256);
                    let b = 28 + ((36 * factor) / 256);
                    let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                    for x in 0..screen_w {
                        buf[row_start + x] = row_color;
                    }
                } else {
                    let dist = y - horizon_y;
                    let factor = (dist * 256) / (screen_h - horizon_y);
                    let r = 12 + ((15 * factor) / 256);
                    let g = 14 + ((20 * factor) / 256);
                    let b = 32 + ((45 * factor) / 256);
                    let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                    for x in 0..screen_w {
                        buf[row_start + x] = row_color;
                    }
                }
            }
            // Linha de neon no horizonte
            for x in 0..screen_w {
                draw_pixel(buf, screen_w, screen_h, x, horizon_y, 0x0038_BDF8);
                if horizon_y > 0 {
                    draw_pixel(buf, screen_w, screen_h, x, horizon_y - 1, 0x001E_3A8A);
                }
            }
            // Linhas de perspectiva abaixo do horizonte
            let center_x = screen_w / 2;
            for i in -6..=6 {
                let start_x = center_x as isize + i * 20;
                let end_x = center_x as isize + i * 110;
                draw_line(buf, screen_w, screen_h, start_x, horizon_y as isize, end_x, (screen_h - 1) as isize, 0x001E_3A5F);
            }
            // Linhas horizontais com espaçamento de perspectiva
            let mut gh_y = horizon_y + 12;
            let mut step = 8;
            while gh_y < screen_h {
                draw_line(buf, screen_w, screen_h, 0, gh_y as isize, screen_w as isize, gh_y as isize, 0x0017_2554);
                step += 7;
                gh_y += step;
            }
        }
        3 => {
            // 3. Solaris Horizon (Pôr-do-sol crepuscular tecnológico com corona e raios solares)
            let horizon_y = (screen_h * 58) / 100;
            for y in 0..screen_h {
                let row_start = y * screen_w;
                if y < horizon_y {
                    let factor = (y * 256) / horizon_y;
                    let r = 16 + ((70 * factor) / 256);
                    let g = 12 + ((28 * factor) / 256);
                    let b = 32 + ((18 * factor) / 256);
                    let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                    for x in 0..screen_w {
                        buf[row_start + x] = row_color;
                    }
                } else {
                    let dist = y - horizon_y;
                    let factor = (dist * 256) / (screen_h - horizon_y);
                    let r = 24usize.saturating_sub((16 * factor) / 256);
                    let g = 16usize.saturating_sub((10 * factor) / 256);
                    let b = 28usize.saturating_sub((18 * factor) / 256);
                    let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                    for x in 0..screen_w {
                        buf[row_start + x] = row_color;
                    }
                }
            }

            // Corona solar semi-elíptica no horizonte
            let sun_cx = screen_w / 2;
            let sun_r = 45usize.min(screen_w / 8);
            for dy in 0..sun_r {
                let sy = horizon_y.saturating_sub(dy);
                let half_span = isqrt((sun_r * sun_r).saturating_sub(dy * dy));
                let x0 = sun_cx.saturating_sub(half_span);
                let x1 = (sun_cx + half_span).min(screen_w);
                let glow_alpha = (180usize.saturating_sub((dy * 160) / sun_r)) as u8;
                for px in x0..x1 {
                    let cur = buf[sy * screen_w + px];
                    buf[sy * screen_w + px] = alpha_blend(cur, COLOR_AMBER, glow_alpha);
                }
            }

            // Faixas de neon dourado e magenta no horizonte
            for x in 0..screen_w {
                draw_pixel(buf, screen_w, screen_h, x, horizon_y, COLOR_AMBER);
                if horizon_y > 0 {
                    draw_pixel(buf, screen_w, screen_h, x, horizon_y - 1, 0x00F9_7316);
                }
            }
        }
        4 => {
            // 4. Obsidian Glass (Ardósia de obsidiana profunda com matriz de circuitos)
            for y in 0..screen_h {
                let factor = (y * 256) / screen_h;
                let r = 7 + ((10 * factor) / 256);
                let g = 9 + ((12 * factor) / 256);
                let b = 15 + ((20 * factor) / 256);
                let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                let row_start = y * screen_w;
                for x in 0..screen_w {
                    buf[row_start + x] = row_color;
                }
            }

            // Trilhas de circuitos procedurais
            let step = 80;
            let mut gx = step;
            while gx < screen_w {
                draw_line(buf, screen_w, screen_h, gx as isize, 0, gx as isize, screen_h as isize, 0x0017_2033);
                gx += step;
            }
            let mut gy = step;
            while gy < screen_h {
                draw_line(buf, screen_w, screen_h, 0, gy as isize, screen_w as isize, gy as isize, 0x0017_2033);
                gy += step;
            }

            // Nós de matriz luminosos nos cruzamentos
            let mut ny = step;
            while ny < screen_h {
                let mut nx = step;
                while nx < screen_w {
                    if ((nx / step) + (ny / step)) % 2 == 0 {
                        draw_pixel(buf, screen_w, screen_h, nx, ny, COLOR_CYAN_NEON);
                        if nx + 1 < screen_w { draw_pixel(buf, screen_w, screen_h, nx + 1, ny, 0x001E_3A5F); }
                        if ny + 1 < screen_h { draw_pixel(buf, screen_w, screen_h, nx, ny + 1, 0x001E_3A5F); }
                    }
                    nx += step;
                }
                ny += step;
            }
        }
        5 => {
            // 5. NASA Cosmic Nebula (Domínio Público / CC0)
            render_nasa_nebula_wallpaper(buf, screen_w, screen_h, mode);
        }
        6 => {
            // 6. Mount Fuji Sunset (Domínio Público / CC0)
            render_mount_fuji_wallpaper(buf, screen_w, screen_h, mode);
        }
        7 => {
            // 7. Deep Space Webb (Domínio Público / CC0)
            render_deep_space_webb_wallpaper(buf, screen_w, screen_h, mode);
        }
        8 => {
            // 8. Matrix Digital Rain (Vídeo / Motion)
            render_matrix_rain_wallpaper(buf, screen_w, screen_h, tick);
        }
        9 => {
            // 9. Starfield Warp 3D (Vídeo / Motion)
            render_starfield_warp_wallpaper(buf, screen_w, screen_h, tick);
        }
        10 => {
            // 10. Cyber Synthwave Pulse (Vídeo / Motion)
            render_synthwave_pulse_wallpaper(buf, screen_w, screen_h, tick);
        }
        11 => {
            // 11. Aurora Wavefield (Vídeo / Motion)
            render_aurora_wavefield_wallpaper(buf, screen_w, screen_h, tick);
        }
        _ => {
            // 0: Cyber-Aurora (Padrão)
            render_procedural_wallpaper(buf, screen_w, screen_h, mode);
        }
    }
}

/// Renderiza wallpaper de Domínio Público (NASA Cosmic Nebula)
pub fn render_nasa_nebula_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    _mode: CapabilityMode,
) {
    if screen_w == 0 || screen_h == 0 { return; }
    // 1. Fundo cósmico com gradiente abissal violeta-índigo
    for y in 0..screen_h {
        let factor = (y * 256) / screen_h;
        let r = 8 + ((18 * factor) / 256);
        let g = 4 + ((10 * factor) / 256);
        let b = 18 + ((28 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }
    }

    // 2. Nuvens de gás e poeira interestelar (Pilares da Criação / Carina)
    let pilar_x = screen_w / 3;
    for y in 40..screen_h {
        let p_factor = y.saturating_sub(40);
        let span = 35 + ((p_factor * 50) / screen_h);
        let center_offset = (isin((y * 3) as i32) * 20) / 256;
        let cx = (pilar_x as isize + center_offset as isize).max(0) as usize;
        let x0 = cx.saturating_sub(span);
        let x1 = (cx + span).min(screen_w);

        for px in x0..x1 {
            let dist = (px as isize - cx as isize).abs() as usize;
            if dist < span {
                let alpha = (((span - dist) * 110) / span) as u8;
                let cur = buf[y * screen_w + px];
                let neb_color = if (y + px) % 3 == 0 { 0x00F5_9E0B } else { 0x00C0_26D3 };
                buf[y * screen_w + px] = alpha_blend(cur, neb_color, alpha);
            }
        }
    }

    // Segundo pilar menor à direita
    let pilar2_x = (screen_w * 2) / 3;
    for y in 80..screen_h {
        let p_factor = y.saturating_sub(80);
        let span = 25 + ((p_factor * 40) / screen_h);
        let center_offset = (icos((y * 4) as i32) * 15) / 256;
        let cx = (pilar2_x as isize + center_offset as isize).max(0) as usize;
        let x0 = cx.saturating_sub(span);
        let x1 = (cx + span).min(screen_w);

        for px in x0..x1 {
            let dist = (px as isize - cx as isize).abs() as usize;
            if dist < span {
                let alpha = (((span - dist) * 90) / span) as u8;
                let cur = buf[y * screen_w + px];
                let neb_color = if (y + px) % 2 == 0 { 0x0038_BDF8 } else { 0x00D9_46EF };
                buf[y * screen_w + px] = alpha_blend(cur, neb_color, alpha);
            }
        }
    }

    // 3. Estrela jovem proeminente com difração cruzada estilo NASA Hubble
    let sx = (screen_w * 7) / 10;
    let sy = screen_h / 4;
    if sx < screen_w && sy < screen_h {
        let spike_len = 50usize.min(screen_w / 8);
        for d in 1..spike_len {
            let alpha = (((spike_len - d) * 180) / spike_len) as u8;
            if sx + d < screen_w { buf[sy * screen_w + (sx + d)] = alpha_blend(buf[sy * screen_w + (sx + d)], COLOR_WHITE, alpha); }
            if sx >= d { buf[sy * screen_w + (sx - d)] = alpha_blend(buf[sy * screen_w + (sx - d)], COLOR_WHITE, alpha); }
            if sy + d < screen_h { buf[(sy + d) * screen_w + sx] = alpha_blend(buf[(sy + d) * screen_w + sx], COLOR_WHITE, alpha); }
            if sy >= d { buf[(sy - d) * screen_w + sx] = alpha_blend(buf[(sy - d) * screen_w + sx], COLOR_WHITE, alpha); }
        }
        draw_pixel(buf, screen_w, screen_h, sx, sy, COLOR_WHITE);
    }

    // Microestrelas pontilhadas
    for i in 0..24 {
        let ex = ((i * 137 + 43) % screen_w).max(1);
        let ey = ((i * 89 + 17) % screen_h).max(1);
        let col = if i % 2 == 0 { COLOR_CYAN_NEON } else { COLOR_AMBER };
        draw_pixel(buf, screen_w, screen_h, ex, ey, col);
    }
}

/// Renderiza wallpaper de Domínio Público (Mount Fuji Sunset / Pacific Horizon)
pub fn render_mount_fuji_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    _mode: CapabilityMode,
) {
    if screen_w == 0 || screen_h == 0 { return; }
    let horizon_y = (screen_h * 58) / 100;

    // 1. Céu crepuscular degradê vermelho-púrpura a dourado quente
    for y in 0..horizon_y {
        let factor = (y * 256) / horizon_y;
        let r = 24 + ((200 * factor) / 256);
        let g = 16 + ((70 * factor) / 256);
        let b = 40 + ((10 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }
    }

    // 2. Lago sereno inferior com reflexo do crepúsculo
    for y in horizon_y..screen_h {
        let dist = y - horizon_y;
        let factor = (dist * 256) / (screen_h - horizon_y);
        let r = 40usize.saturating_sub((25 * factor) / 256);
        let g = 25usize.saturating_sub((15 * factor) / 256);
        let b = 48usize.saturating_sub((20 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }
    }

    // 3. Silhueta solene do Monte Fuji
    let cx = screen_w / 2;
    let peak_y = horizon_y.saturating_sub(screen_h / 4);
    let mountain_h = horizon_y.saturating_sub(peak_y).max(1);
    let base_half_w = (screen_w * 22) / 100;

    for y in peak_y..horizon_y {
        let dy = y - peak_y;
        let half_w = (dy * base_half_w) / mountain_h;
        let x0 = cx.saturating_sub(half_w);
        let x1 = (cx + half_w).min(screen_w);

        let is_snow = dy < (mountain_h * 28) / 100;
        let color = if is_snow {
            let snow_shade = 220 + ((dy * 30) / mountain_h);
            ((snow_shade as u32) << 16) | ((snow_shade as u32) << 8) | (snow_shade as u32 + 5)
        } else {
            0x001B_1C2E
        };

        for px in x0..x1 {
            buf[y * screen_w + px] = color;
        }
    }

    // Linha dourada do horizonte
    for x in 0..screen_w {
        draw_pixel(buf, screen_w, screen_h, x, horizon_y, 0x00EA_580C);
    }
}

/// Renderiza wallpaper de Domínio Público (Deep Space Webb Telescope)
pub fn render_deep_space_webb_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    _mode: CapabilityMode,
) {
    if screen_w == 0 || screen_h == 0 { return; }
    // 1. Vazio cósmico profundo
    for y in 0..screen_h {
        let factor = (y * 256) / screen_h;
        let r = 4 + ((6 * factor) / 256);
        let g = 6 + ((8 * factor) / 256);
        let b = 12 + ((16 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }
    }

    // 2. Galáxias elípticas e espirais douradas
    let galaxies: [(usize, usize, usize, u32); 8] = [
        (screen_w / 4, screen_h / 3, 22, 0x00FB_BF24),
        ((screen_w * 3) / 5, screen_h / 4, 18, 0x00F5_9E0B),
        (screen_w / 2, (screen_h * 3) / 5, 26, 0x00FE_F08A),
        ((screen_w * 4) / 5, (screen_h * 7) / 10, 15, 0x00F9_7316),
        ((screen_w * 7) / 8, screen_h / 3, 14, 0x0038_BDF8),
        (screen_w / 6, (screen_h * 3) / 4, 16, 0x00FBBF24),
        (screen_w / 3, (screen_h * 4) / 5, 20, 0x0060_A5FA),
        ((screen_w * 2) / 3, (screen_h * 2) / 5, 12, 0x00FE_F08A),
    ];

    for &(gx, gy, gr, gcol) in &galaxies {
        for dy in 0..gr {
            let dx_max = isqrt((gr * gr).saturating_sub(dy * dy));
            let alpha = (((gr - dy) * 160) / gr) as u8;
            for sign_y in [-1isize, 1isize] {
                let py = (gy as isize + sign_y * dy as isize) as usize;
                if py < screen_h {
                    let x0 = gx.saturating_sub(dx_max);
                    let x1 = (gx + dx_max).min(screen_w);
                    for px in x0..x1 {
                        buf[py * screen_w + px] = alpha_blend(buf[py * screen_w + px], gcol, alpha / 2);
                    }
                }
            }
        }
        draw_pixel(buf, screen_w, screen_h, gx, gy, COLOR_WHITE);
    }

    // 3. Arcos de lentes gravitacionais (azuis ciano curvos)
    let l_cx = screen_w / 2;
    let l_cy = (screen_h * 3) / 5;
    for deg in 30..150 {
        let r = 40;
        let px = (l_cx as isize + ((icos(deg) * r) / 256) as isize) as usize;
        let py = (l_cy as isize + ((isin(deg) * (r / 2)) / 256) as isize) as usize;
        if px < screen_w && py < screen_h {
            buf[py * screen_w + px] = alpha_blend(buf[py * screen_w + px], 0x0038_BDF8, 180);
        }
    }
}

/// Renderiza wallpaper de Vídeo / Motion (Matrix Digital Rain)
pub fn render_matrix_rain_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    tick: u64,
) {
    if screen_w == 0 || screen_h == 0 { return; }
    for pixel in buf.iter_mut() {
        *pixel = 0x0002_0A04;
    }

    let col_step = 16;
    let num_cols = screen_w / col_step;

    for c in 0..num_cols {
        let col_x = c * col_step + 4;
        let speed = 2 + (c % 4);
        let offset = (c * 47) % screen_h;
        let head_y = (((tick as usize * speed) + offset) % (screen_h + 100)).saturating_sub(40);

        for i in 0..14 {
            if head_y >= i * 14 {
                let py = head_y - (i * 14);
                if py + 8 < screen_h {
                    let col = match i {
                        0 => COLOR_WHITE,
                        1 => 0x00A7_F3D0,
                        2..=4 => COLOR_EMERALD,
                        5..=8 => 0x0005_9669,
                        _ => 0x0006_4E3B,
                    };
                    for gy in 0..6 {
                        for gx in 0..3 {
                            if (gy + gx + c + (tick as usize / 4)) % 2 == 0 {
                                draw_pixel(buf, screen_w, screen_h, col_x + gx, py + gy, col);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Renderiza wallpaper de Vídeo / Motion (Starfield Warp 3D)
pub fn render_starfield_warp_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    tick: u64,
) {
    if screen_w == 0 || screen_h == 0 { return; }
    for pixel in buf.iter_mut() {
        *pixel = 0x0002_040A;
    }

    let cx = screen_w as isize / 2;
    let cy = screen_h as isize / 2;

    for i in 0..52 {
        let seed_x = ((i * 137 + 19) % 800) as isize - 400;
        let seed_y = ((i * 223 + 47) % 600) as isize - 300;
        let seed_z = ((i * 89 + 11) % 600) as isize + 1;

        let z = (((seed_z - ((tick as isize * 14) % 600)) + 600) % 600) + 1;
        let prev_z = (z + 28).min(600);

        let px = cx + (seed_x * 280) / z;
        let py = cy + (seed_y * 280) / z;
        let ppx = cx + (seed_x * 280) / prev_z;
        let ppy = cy + (seed_y * 280) / prev_z;

        if px >= 0 && px < screen_w as isize && py >= 0 && py < screen_h as isize {
            let color = if z < 150 {
                COLOR_WHITE
            } else if z < 300 {
                COLOR_CYAN_NEON
            } else {
                0x0033_4155
            };
            draw_line(buf, screen_w, screen_h, ppx, ppy, px, py, color);
            draw_pixel(buf, screen_w, screen_h, px as usize, py as usize, COLOR_WHITE);
        }
    }
}

/// Renderiza wallpaper de Vídeo / Motion (Cyber Synthwave Pulse)
pub fn render_synthwave_pulse_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    tick: u64,
) {
    if screen_w == 0 || screen_h == 0 { return; }
    let horizon_y = (screen_h * 55) / 100;

    // 1. Céu crepuscular Synthwave
    for y in 0..horizon_y {
        let factor = (y * 256) / horizon_y;
        let r = 16 + ((120 * factor) / 256);
        let g = 8 + ((20 * factor) / 256);
        let b = 36 + ((60 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }
    }

    // 2. Chão synthwave em perspectiva
    for y in horizon_y..screen_h {
        let dist = y - horizon_y;
        let factor = (dist * 256) / (screen_h - horizon_y);
        let r = 14 + ((20 * factor) / 256);
        let g = 10 + ((15 * factor) / 256);
        let b = 32 + ((40 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }
    }

    // 3. Sol neon pulsante com linhas pretas horizontais
    let sun_cx = screen_w / 2;
    let pulse = (isin((tick * 8) as i32) * 5) / 256;
    let sun_r = (46isize + pulse as isize).max(20) as usize;

    for dy in 0..sun_r {
        let half_span = isqrt((sun_r * sun_r).saturating_sub(dy * dy));
        for sign_y in [-1isize, 1isize] {
            let sy = (horizon_y as isize + sign_y * dy as isize) as usize;
            if sy < horizon_y {
                let stripe = (dy / 4) % 2 == 1 && dy > 12;
                if !stripe {
                    let factor = (dy * 256) / sun_r;
                    let scolor = if factor < 128 { 0x00FB_BF24 } else { 0x00F4_3F5E };
                    let x0 = sun_cx.saturating_sub(half_span);
                    let x1 = (sun_cx + half_span).min(screen_w);
                    for px in x0..x1 {
                        buf[sy * screen_w + px] = scolor;
                    }
                }
            }
        }
    }

    // Linha de neon no horizonte
    for x in 0..screen_w {
        draw_pixel(buf, screen_w, screen_h, x, horizon_y, COLOR_CYAN_NEON);
    }

    // Linhas de perspectiva do chão
    let center_x = screen_w as isize / 2;
    for i in -7..=7 {
        let start_x = center_x + i * 20;
        let end_x = center_x + i * 115;
        draw_line(buf, screen_w, screen_h, start_x, horizon_y as isize, end_x, (screen_h - 1) as isize, 0x0083_1843);
    }

    // Linhas horizontais em movimento contínuo para a frente
    let move_offset = (tick as usize * 3) % 24;
    let mut gh_y = horizon_y + 8 + move_offset;
    let mut step = 10;
    while gh_y < screen_h {
        draw_line(buf, screen_w, screen_h, 0, gh_y as isize, screen_w as isize, gh_y as isize, 0x00D9_46EF);
        step += 8;
        gh_y += step;
    }
}

/// Renderiza wallpaper de Vídeo / Motion (Aurora Wavefield)
pub fn render_aurora_wavefield_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    tick: u64,
) {
    if screen_w == 0 || screen_h == 0 { return; }
    // 1. Noite profunda
    for y in 0..screen_h {
        let factor = (y * 256) / screen_h;
        let r = 6 + ((8 * factor) / 256);
        let g = 8 + ((12 * factor) / 256);
        let b = 18 + ((22 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }
    }

    // 2. Cortinas ondulantes de plasma da aurora
    for x in 0..screen_w {
        let w1 = (isin(x as i32 * 3 + tick as i32 * 6) * 35) / 256;
        let w2 = (icos(x as i32 * 2 - tick as i32 * 4) * 25) / 256;
        let mid_y = (160isize + w1 as isize + w2 as isize).max(40) as usize;

        for dy in 0..50 {
            let y = mid_y + dy;
            if y < screen_h {
                let alpha = (((50 - dy) * 140) / 50) as u8;
                let col = if dy < 20 { COLOR_CYAN_NEON } else { COLOR_EMERALD };
                buf[y * screen_w + x] = alpha_blend(buf[y * screen_w + x], col, alpha);
            }
        }
    }

    // Montanhas escuras na base
    let base_y = (screen_h * 75) / 100;
    for x in 0..screen_w {
        let my = (base_y as isize + ((isin((x * 4) as i32) * 20) / 256) as isize) as usize;
        for y in my..screen_h {
            buf[y * screen_w + x] = 0x0009_0D14;
        }
    }
}

/// Renderiza wallpaper procedural "Cyber-Aurora" exclusivo do Aladdin OS
/// Apresenta gradiente noturno profundo, fitas aurorais luminescentes e nós de constelações tecnológicas.
pub fn render_procedural_wallpaper(
    buf: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    mode: CapabilityMode,
) {
    if screen_w == 0 || screen_h == 0 { return; }

    // 1. Gradiente Base Noturno (Deep Midnight to Dark Slate: Topo mais escuro que a base)
    for y in 0..screen_h {
        let factor = (y * 256) / screen_h;
        let r = 9 + ((10 * factor) / 256);
        let g = 13 + ((14 * factor) / 256);
        let b = 20 + ((25 * factor) / 256);
        let row_color = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        let row_start = y * screen_w;

        // Preenche linha base
        for x in 0..screen_w {
            buf[row_start + x] = row_color;
        }

        // 2. Fita Ondulada de Aurora Boreal (Cyber-Aurora) nos modos Equilibrado e Avançado
        if mode != CapabilityMode::Basic && y >= 60 && y <= 340 {
            let mid_y = 190isize;
            let dist_from_center = (y as isize - mid_y).abs() as usize;

            for x in 0..screen_w {
                // Cálculo de ondulação pseudo-harmônica usando operadores inteiros rápidos
                let wave1 = (((x * 5) / 16 + (y * 3) / 8) & 63) as usize;
                let wave2 = (((x * 7) / 24) & 31) as usize;
                let wave_val = if wave1 < 32 { wave1 } else { 63 - wave1 } + wave2;

                if wave_val > 18 && dist_from_center < 110 {
                    let intensity = ((32 - dist_from_center / 4) * (wave_val - 18)) / 32;
                    if intensity > 0 {
                        let cur = buf[row_start + x];
                        let cr = (cur >> 16) & 0xFF;
                        let cg = (cur >> 8) & 0xFF;
                        let cb = cur & 0xFF;

                        // Adiciona brilho ciano / esmeralda / violeta
                        let nr = (cr + (intensity as u32 * 2)).min(45);
                        let ng = (cg + (intensity as u32 * 4)).min(70);
                        let nb = (cb + (intensity as u32 * 6)).min(95);

                        buf[row_start + x] = (nr << 16) | (ng << 8) | nb;
                    }
                }
            }
        }
    }

    // 3. Rede de Constelações Tecnológicas (Starfield & Tech Nodes)
    if mode == CapabilityMode::Advanced {
        // Coordenadas das estrelas/nós fixos na proporção da tela
        let stars: [(usize, usize, u32); 14] = [
            (screen_w / 8, 70, COLOR_CYAN_NEON),
            (screen_w / 5, 120, COLOR_WHITE),
            (screen_w / 4, 85, COLOR_EMERALD),
            (screen_w / 3, 160, COLOR_CYAN_NEON),
            (screen_w / 2 - 40, 95, COLOR_WHITE),
            (screen_w / 2 + 60, 140, COLOR_PURPLE),
            ((screen_w * 2) / 3, 80, COLOR_CYAN_NEON),
            ((screen_w * 3) / 4, 130, COLOR_WHITE),
            ((screen_w * 7) / 8, 90, COLOR_EMERALD),
            (screen_w / 6, 260, COLOR_TEXT_MUTED),
            (screen_w / 3 + 20, 290, COLOR_CYAN_NEON),
            (screen_w / 2, 250, COLOR_WHITE),
            ((screen_w * 2) / 3 - 30, 275, COLOR_PURPLE),
            ((screen_w * 5) / 6, 260, COLOR_CYAN_NEON),
        ];

        // Linhas de constelação tênues entre nós vizinhos
        for i in 0..stars.len() - 1 {
            let (x0, y0, _) = stars[i];
            let (x1, y1, _) = stars[i + 1];
            let dx = (x1 as isize - x0 as isize).abs();
            let dy = (y1 as isize - y0 as isize).abs();
            if dx < (screen_w as isize / 5) && dy < 120 {
                draw_line(buf, screen_w, screen_h, x0 as isize, y0 as isize, x1 as isize, y1 as isize, 0x0015_2238);
            }
        }

        // Desenha os pontos estelares com aura em cruz
        for &(sx, sy, star_color) in &stars {
            if sx > 1 && sx < screen_w - 2 && sy > 1 && sy < screen_h - 2 {
                // Ponto central
                draw_pixel(buf, screen_w, screen_h, sx, sy, star_color);
                // Brilho tênue adjacente
                let glow = 0x002D_3748;
                draw_pixel(buf, screen_w, screen_h, sx - 1, sy, glow);
                draw_pixel(buf, screen_w, screen_h, sx + 1, sy, glow);
                draw_pixel(buf, screen_w, screen_h, sx, sy - 1, glow);
                draw_pixel(buf, screen_w, screen_h, sx, sy + 1, glow);
            }
        }
    }
}
