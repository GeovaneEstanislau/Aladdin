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

#[test]
fn test_client_window_lifecycle_and_helpers() {
    let mut wm = WindowManager::new();

    // 1. Cria janela com dimensões úteis de cliente (320 x 200)
    let win_id = wm.create_client_window(101, "Aplicacao Ring 3", 50, 50, 320, 200)
        .expect("Deveria criar janela cliente via helper");
    assert_eq!(win_id, 0);

    let win = wm.windows[win_id].as_ref().unwrap();
    assert_eq!(win.title_str(), "Aplicacao Ring 3");
    assert_eq!(win.owner_pid, Some(101));
    assert_eq!(win.client_width(), 320);
    assert_eq!(win.client_height(), 200);

    // 2. Atualiza buffer de cliente com dados válidos
    let expected_len = 320 * 200;
    let mut fake_buffer = alloc::vec![0x0012_3456u32; expected_len];
    fake_buffer[0] = 0x00FF_0000;

    let update_res = wm.update_client_buffer(101, win_id, &fake_buffer);
    assert!(update_res.is_ok());
    assert_eq!(wm.client_buffers[win_id].as_ref().unwrap()[0], 0x00FF_0000);

    // Tentativa de outro PID atualizar deve falhar com permissão negada
    let fraud_res = wm.update_client_buffer(999, win_id, &fake_buffer);
    assert_eq!(fraud_res, Err(aladdin::wm::WindowOpError::PermissionDenied));

    // Tentativa de atualizar com dimensões inconsistentes deve falhar
    let bad_dim = wm.update_client_buffer(101, win_id, &[0x1234; 10]);
    assert_eq!(bad_dim, Err(aladdin::wm::WindowOpError::InvalidDimensions));

    // Tentativa de atualizar janela inexistente deve retornar NotFound
    let not_found_up = wm.update_client_buffer(101, 7, &fake_buffer);
    assert_eq!(not_found_up, Err(aladdin::wm::WindowOpError::NotFound));

    // Tentativa de fechar por outro PID deve falhar
    let fraud_close = wm.close_client_window(999, win_id);
    assert_eq!(fraud_close, Err(aladdin::wm::WindowOpError::PermissionDenied));

    // Tentativa de fechar janela inexistente deve falhar
    let not_found_cl = wm.close_client_window(101, 7);
    assert_eq!(not_found_cl, Err(aladdin::wm::WindowOpError::NotFound));

    // 3. Fechamento seguro pelo proprietário legítimo
    let ok_close = wm.close_client_window(101, win_id);
    assert!(ok_close.is_ok());
    assert!(wm.windows[win_id].is_none());
    assert!(wm.client_buffers[win_id].is_none());
}

#[test]
fn test_window_manager_stress_and_scissor_clipping() {
    let mut wm = WindowManager::new();

    // 1. Abre até a capacidade máxima de janelas (MAX_WINDOWS = 8)
    let mut opened_ids = [0usize; aladdin::wm::MAX_WINDOWS];
    for i in 0..aladdin::wm::MAX_WINDOWS {
        let win_id = wm.create_client_window(i + 1, "App", 50, 50, 100, 100)
            .expect("Deveria abrir janela até o limite máximo");
        opened_ids[i] = win_id;
    }

    // 2. A 9ª janela deve ser rejeitada com erro de capacidade (EMFILE / Err)
    let overflow_win = wm.create_client_window(99, "Overflow", 50, 50, 100, 100);
    assert!(overflow_win.is_err());

    // 3. Testa recorte de tesoura (Scissor Clipping)
    let clip = wm.windows[opened_ids[0]].as_ref().unwrap().client_clip();
    assert!(clip.min_x > 0);
    assert!(clip.min_y > 0);
    assert!(clip.max_x > clip.min_x);
    assert!(clip.max_y > clip.min_y);

    // 4. Fechamento em massa por PID
    wm.close_all_for_pid(opened_ids[3] + 1);
    assert!(wm.windows[opened_ids[3]].is_none());
    assert!(wm.client_buffers[opened_ids[3]].is_none());

    // 5. Após fechar uma janela, novo slot é liberado
    let recovered_id = wm.create_client_window(888, "Nova Janela", 50, 50, 200, 150)
        .expect("Deveria reutilizar slot liberado");
    assert_eq!(recovered_id, opened_ids[3]);
}

#[test]
fn test_performance_and_memory_invariants() {
    // 1. Verificação de Footprint em Memória
    let win_size = core::mem::size_of::<Window>();
    // Estrutura Window deve ser compacta (<= 192 bytes, tamanho real 168 bytes)
    assert!(win_size <= 192, "Tamanho de Window excedeu limite: {} bytes", win_size);

    let rect_size = core::mem::size_of::<DirtyRect>();
    // DirtyRect: 4 x usize (32 bytes em x86_64)
    assert!(rect_size <= 32, "Tamanho de DirtyRect excedeu limite: {} bytes", rect_size);

    let clip_size = core::mem::size_of::<ClipRect>();
    assert!(clip_size <= 32, "Tamanho de ClipRect excedeu limite: {} bytes", clip_size);

    // 2. Throughput de Criação e Destruição de Janelas
    let mut wm = WindowManager::new();
    let start = std::time::Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let win_id = wm.create_client_window(42, "Bench App", 10, 10, 100, 100)
            .expect("Deveria criar janela no bench");
        let _ = wm.close_client_window(42, win_id).expect("Deveria fechar janela");
    }
    let elapsed = start.elapsed();
    let per_op_micros = elapsed.as_micros() as f64 / iterations as f64;
    println!("Throughput de ciclo de vida de janela: {:.2} µs por criacao+fechamento", per_op_micros);
    // Deve levar menos de 100 µs por ciclo de alocação/desalocação no host em modo debug (< 10 µs em release)
    assert!(per_op_micros < 100.0, "Ciclo de vida de janela muito lento: {:.2} µs", per_op_micros);

    // 3. Throughput de Operações Geométricas de Dirty Rects
    let rect_start = std::time::Instant::now();
    let mut current_rect = DirtyRect::new(0, 0, 10, 10);
    for i in 0..10_000 {
        let next = DirtyRect::new(i % 1024, (i * 2) % 768, 50, 50);
        current_rect = current_rect.union(&next);
    }
    let rect_elapsed = rect_start.elapsed();
    let per_rect_nanos = rect_elapsed.as_nanos() as f64 / 10_000.0;
    println!("Throughput de union de DirtyRect: {:.2} ns por uniao", per_rect_nanos);
    assert!(per_rect_nanos < 100.0, "Operacao de DirtyRect muito lenta: {:.2} ns", per_rect_nanos);
}

#[test]
fn test_client_window_protocol_contract_validation() {
    let mut wm = WindowManager::new();

    // 1. Validar criação de múltiplas janelas para o mesmo PID
    let pid = 200;
    let w1 = wm.create_client_window(pid, "App 1", 20, 20, 200, 150).expect("W1 ok");
    let w2 = wm.create_client_window(pid, "App 2", 40, 40, 200, 150).expect("W2 ok");
    assert_ne!(w1, w2);

    // 2. Validação estrita de integridade dimensional
    // Buffer menor que o esperado (esperado: 200 * 150 = 30000 pixels)
    let small_buf = alloc::vec![0u32; 1000];
    let err_small = wm.update_client_buffer(pid, w1, &small_buf);
    assert_eq!(err_small, Err(aladdin::wm::WindowOpError::InvalidDimensions));

    // Buffer maior que o esperado
    let large_buf = alloc::vec![0u32; 50000];
    let err_large = wm.update_client_buffer(pid, w1, &large_buf);
    assert_eq!(err_large, Err(aladdin::wm::WindowOpError::InvalidDimensions));

    // Buffer com dimensão exata
    let exact_buf = alloc::vec![0x00FF_AABB; 200 * 150];
    let ok_up = wm.update_client_buffer(pid, w1, &exact_buf);
    assert!(ok_up.is_ok());

    // 3. Validação de segurança cross-PID (isolamento inter-processos)
    let foreign_pid = 201;
    let cross_up = wm.update_client_buffer(foreign_pid, w1, &exact_buf);
    assert_eq!(cross_up, Err(aladdin::wm::WindowOpError::PermissionDenied));

    let cross_close = wm.close_client_window(foreign_pid, w1);
    assert_eq!(cross_close, Err(aladdin::wm::WindowOpError::PermissionDenied));

    // Janelas ainda devem estar abertas
    assert!(wm.windows[w1].is_some());
    assert!(wm.windows[w2].is_some());

    // 4. Limpeza automática de todas as janelas por encerramento de processo (zombie cleanup)
    wm.close_all_for_pid(pid);
    assert!(wm.windows[w1].is_none());
    assert!(wm.windows[w2].is_none());
    assert!(wm.client_buffers[w1].is_none());
    assert!(wm.client_buffers[w2].is_none());

    // 5. Verificação de reuso de slot após limpeza
    let new_pid = 300;
    let w_new = wm.create_client_window(new_pid, "Novo App", 10, 10, 100, 100).expect("Reuso ok");
    assert!(w_new < aladdin::wm::MAX_WINDOWS);
    assert_eq!(wm.windows[w_new].as_ref().unwrap().owner_pid, Some(new_pid));
}

#[test]
fn test_session_client_damage_and_zombie_cleanup() {
    let mut session = aladdin::desktop::session::AladdinSession::new();
    assert!(!session.client_damaged);

    // 1. Criar janela cliente através da sessão deve marcar client_damaged
    let pid = 500;
    let win_id = session.create_client_window(pid, "Sessao App", 120, 80).expect("Janela criada");
    assert!(session.client_damaged);
    session.client_damaged = false;

    // 2. Atualizar buffer da janela cliente deve marcar client_damaged
    let buf = alloc::vec![0x0011_2233; 120 * 80];
    let up_res = session.update_client_buffer(pid, win_id, &buf);
    assert!(up_res.is_ok());
    assert!(session.client_damaged);
    session.client_damaged = false;

    // 3. Atualização rejeitada por PID intruso não deve marcar client_damaged
    let intruso_pid = 666;
    let intruso_res = session.update_client_buffer(intruso_pid, win_id, &buf);
    assert_eq!(intruso_res, Err(aladdin::wm::WindowOpError::PermissionDenied));
    assert!(!session.client_damaged);

    // 4. Fechar janelas do PID deve limpar recursos e marcar client_damaged
    session.close_windows_for_pid(pid);
    assert!(session.client_damaged);
    assert!(session.wm.windows[win_id].is_none());
}

#[test]
fn test_dirty_rect_edge_cases_and_clamping() {
    // 1. is_empty
    assert!(DirtyRect::new(10, 10, 0, 50).is_empty());
    assert!(DirtyRect::new(10, 10, 50, 0).is_empty());
    assert!(DirtyRect::new(10, 10, 0, 0).is_empty());
    assert!(!DirtyRect::new(10, 10, 50, 50).is_empty());

    // 2. clamp parcialmente fora
    let r_partial = DirtyRect::new(800, 600, 300, 200);
    let clamped = r_partial.clamp(1024, 768);
    assert_eq!(clamped.x, 800);
    assert_eq!(clamped.y, 600);
    assert_eq!(clamped.width, 224); // 1024 - 800
    assert_eq!(clamped.height, 168); // 768 - 600

    // 3. clamp completamente fora
    let r_outside = DirtyRect::new(1200, 800, 100, 100);
    let clamped_out = r_outside.clamp(1024, 768);
    assert_eq!(clamped_out.x, 1024);
    assert_eq!(clamped_out.y, 768);
    assert_eq!(clamped_out.width, 0);
    assert_eq!(clamped_out.height, 0);
    assert!(clamped_out.is_empty());

    // 4. intersect: sobreposição, disjunção e tangência
    let r1 = DirtyRect::new(10, 10, 50, 50);
    let r2 = DirtyRect::new(30, 30, 50, 50);
    let inter = r1.intersect(&r2).expect("Deveria haver interseção");
    assert_eq!(inter.x, 30);
    assert_eq!(inter.y, 30);
    assert_eq!(inter.width, 30);
    assert_eq!(inter.height, 30);

    let r_disjoint = DirtyRect::new(100, 100, 20, 20);
    assert!(r1.intersect(&r_disjoint).is_none());

    // Tangente na borda direita (x=60 não sobrepõe x=10..60)
    let r_tangent = DirtyRect::new(60, 10, 20, 50);
    assert!(r1.intersect(&r_tangent).is_none());
}

#[test]
fn test_client_window_dimensional_overflow_and_zero_rejection() {
    let mut wm = WindowManager::new();

    // 1. Rejeição de dimensões zero
    assert!(wm.create_client_window(1, "Zero Width", 10, 10, 0, 100).is_err());
    assert!(wm.create_client_window(1, "Zero Height", 10, 10, 100, 0).is_err());
    assert!(wm.create_client_window(1, "Zero Both", 10, 10, 0, 0).is_err());

    // 2. Rejeição de overflow de multiplicação de dimensões
    assert!(wm.create_client_window(1, "Overflow W", 10, 10, usize::MAX / 2, 4).is_err());
    assert!(wm.create_client_window(1, "Overflow H", 10, 10, 4, usize::MAX / 2).is_err());

    // 3. Rejeição de limite máximo de área (> 1024 * 768 = 786432)
    assert!(wm.create_client_window(1, "Too Big", 10, 10, 2000, 2000).is_err());
    assert!(wm.create_client_window(1, "Just Over", 10, 10, 1025, 768).is_err());

    // 4. Criação no limite suportado (1024 * 768) deve ser aceita
    let max_ok = wm.create_client_window(1, "Max Screen", 0, 0, 1024, 768);
    assert!(max_ok.is_ok());
    let win_id = max_ok.unwrap();
    assert_eq!(wm.client_buffers[win_id].as_ref().unwrap().len(), 1024 * 768);

    // 5. Teste de saturating arithmetic em Window::new_with_client_size
    let extreme_win = Window::new_with_client_size(0, "Extreme", 0, 0, usize::MAX, usize::MAX);
    assert_eq!(extreme_win.width, usize::MAX); // saturating_add evitou overflow panic
    assert_eq!(extreme_win.height, usize::MAX);
}

#[test]
fn test_window_manager_z_order_composition() {
    let mut wm = WindowManager::new();

    // Cria 3 janelas sobrepostas
    let w0 = wm.create_client_window(10, "Janela Fundo", 0, 0, 100, 100).expect("W0 ok");
    let w1 = wm.create_client_window(10, "Janela Meio", 20, 20, 100, 100).expect("W1 ok");
    let w2 = wm.create_client_window(10, "Janela Topo", 40, 40, 100, 100).expect("W2 ok");

    assert_eq!(wm.top_window_id(), w2);

    // Ponto (45, 45) está nas três janelas -> topo deve ser w2
    assert_eq!(wm.find_window_at(45, 45), Some(w2));

    // Ponto (25, 25) está em w0 e w1 -> topo deve ser w1
    assert_eq!(wm.find_window_at(25, 25), Some(w1));

    // Traz w0 para a frente
    wm.bring_to_front(w0);
    assert_eq!(wm.top_window_id(), w0);
    assert_eq!(wm.find_window_at(25, 25), Some(w0));

    // Fecha a janela w1 (do meio)
    let _ = wm.close_client_window(10, w1);
    assert!(wm.windows[w1].is_none());

    // w0 continua no topo
    assert_eq!(wm.top_window_id(), w0);
    assert_eq!(wm.find_window_at(25, 25), Some(w0));

    // Ponto (45, 45) que estava em w2 e w0: w0 cobre (0..102, 0..126)
    assert_eq!(wm.find_window_at(45, 45), Some(w0));

    // Ponto (110, 110) está fora de w0 mas dentro de w2 (x=40..142, y=40..166)
    assert_eq!(wm.find_window_at(110, 110), Some(w2));
}

