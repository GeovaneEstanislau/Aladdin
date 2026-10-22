extern crate aladdin;
extern crate alloc;

use aladdin::contracts::framebuffer::DirtyRect;
use aladdin::compositor::ClipRect;
use aladdin::wm::{Window, WindowManager};
use aladdin::apps::TerminalApp;

#[test]
fn test_dirty_rect_union_and_intersects() {
    let r1 = DirtyRect::new(10, 10, 50, 50);
    let r2 = DirtyRect::new(30, 30, 50, 50);
    assert!(r1.intersects(&r2));

    let u = r1.union(&r2);
    assert_eq!(u.x, 10);
    assert_eq!(u.y, 10);
    assert_eq!(u.width, 70);
    assert_eq!(u.height, 70);

    let r3 = DirtyRect::new(100, 100, 20, 20);
    assert!(!r1.intersects(&r3));
}

#[test]
fn test_clip_rect_bounds() {
    let clip = ClipRect::new(50, 50, 200, 200);
    assert!(clip.contains_pixel(50, 50));
    assert!(clip.contains_pixel(199, 199));
    assert!(!clip.contains_pixel(49, 50));
    assert!(!clip.contains_pixel(200, 200));

    let c2 = ClipRect::new(100, 100, 300, 300);
    let inter = clip.intersect(&c2).expect("Deveria intersectar");
    assert_eq!(inter.min_x, 100);
    assert_eq!(inter.min_y, 100);
    assert_eq!(inter.max_x, 200);
    assert_eq!(inter.max_y, 200);
}

#[test]
fn test_window_geometry_and_maximize() {
    let mut win = Window::new(0, "Test Window", 100, 100, 300, 200);
    assert!(win.contains_point(150, 150));
    assert!(!win.contains_point(50, 50));

    // Maximizar
    let damage = win.toggle_maximize(1024, 768, 26, 54);
    assert!(win.is_maximized);
    assert_eq!(win.x, 0);
    assert_eq!(win.y, 27);
    assert_eq!(win.width, 1024);
    assert_eq!(win.height, 678);
    assert!(damage.width >= 1024);

    // Restaurar
    let _ = win.toggle_maximize(1024, 768, 26, 54);
    assert!(!win.is_maximized);
    assert_eq!(win.x, 100);
    assert_eq!(win.y, 100);
    assert_eq!(win.width, 300);
    assert_eq!(win.height, 200);
}

#[test]
fn test_window_manager_dynamic_open_close() {
    let mut wm = WindowManager::new();
    let id0 = wm.open_window(Window::new(0, "W0", 10, 10, 100, 100)).expect("Deveria abrir");
    let id1 = wm.open_window(Window::new(1, "W1", 20, 20, 100, 100)).expect("Deveria abrir");
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(wm.top_window_id(), 1);

    let damage = wm.close_window(0).expect("Deveria fechar W0");
    assert_eq!(damage.width, 100);
    assert!(wm.windows[0].is_none());
}

#[test]
fn test_window_manager_z_order() {
    let mut wm = WindowManager::new();
    wm.windows[0] = Some(Window::new(0, "Win 0", 10, 10, 100, 100));
    wm.windows[1] = Some(Window::new(1, "Win 1", 20, 20, 100, 100));
    wm.windows[2] = Some(Window::new(2, "Win 2", 30, 30, 100, 100));
    wm.windows[3] = Some(Window::new(3, "Win 3", 40, 40, 100, 100));
    wm.order = [0, 1, 2, 3, 4, 5, 6, 7];

    wm.bring_to_front(1);
    assert_eq!(wm.top_window_id(), 1);

    wm.bring_to_front(0);
    assert_eq!(wm.top_window_id(), 0);
}

#[test]
fn test_terminal_history_buffer() {
    let mut term = TerminalApp::new();
    term.push_line("Line 1", 0xFFFFFF);
    term.push_line("Line 2", 0xFFFFFF);
    term.push_line("Line 3", 0xFFFFFF);
    term.push_line("Line 4", 0xFFFFFF);
    term.push_line("Line 5", 0xFFFFFF);
    term.push_line("Line 6", 0xFFFFFF);
    assert_eq!(term.history_count, 6);

    // Overflow scrolls
    term.push_line("Line 7", 0x00FF00);
    assert_eq!(term.history_count, 6);
    let last_len = term.history_lens[5];
    assert_eq!(&term.history[5][..last_len], b"Line 7");
}

#[test]
fn test_client_window_buffer_and_title() {
    let mut wm = WindowManager::new();
    let mut client_win = Window::new(0, "Demo Ring 3 Client", 150, 120, 320, 240);
    client_win.owner_pid = Some(42);

    assert_eq!(client_win.title_str(), "Demo Ring 3 Client");
    assert_eq!(client_win.owner_pid, Some(42));
    assert_eq!(client_win.client_width(), 318);
    assert_eq!(client_win.client_height(), 240 - 24 - 2);

    let id = wm.open_window(client_win).expect("Deveria abrir janela cliente");
    assert_eq!(id, 0);

    // Aloca e anexa buffer de cliente
    let client_pixels = client_win.client_width() * client_win.client_height();
    wm.client_buffers[id] = Some(alloc::vec![0x00FF_00FF; client_pixels]);
    assert!(wm.client_buffers[id].is_some());
    assert_eq!(wm.client_buffers[id].as_ref().unwrap().len(), client_pixels);

    let damage = wm.close_window(id).expect("Deveria fechar janela cliente");
    assert_eq!(damage.width, 320);
    assert!(wm.client_buffers[id].is_none());
}
