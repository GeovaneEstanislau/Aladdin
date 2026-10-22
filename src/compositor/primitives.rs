//! Primitivas Gráficas 2D de Alta Performance com Suporte a Recorte (Clipping)

use super::damage::ClipRect;

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
    fill_rect(buf, screen_w, screen_h, x, y, width, height, bg);
    draw_rect_outline(buf, screen_w, screen_h, x, y, width, height, border);

    if total > 0 && used > 0 && width > 4 {
        let fill_w = ((width - 4) * used.min(total)) / total;
        if fill_w > 0 {
            fill_rect(buf, screen_w, screen_h, x + 2, y + 2, fill_w, height.saturating_sub(4), fg);
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
