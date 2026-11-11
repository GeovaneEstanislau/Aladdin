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
    wm.order = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

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
    assert!(per_op_micros < 350.0, "Ciclo de vida de janela muito lento: {:.2} µs", per_op_micros);

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
    assert!(per_rect_nanos < 350.0, "Operacao de DirtyRect muito lenta: {:.2} ns", per_rect_nanos);
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


struct MockSystem {
    profile: std::sync::atomic::AtomicU8,
    storage: &'static str,
}

impl MockSystem {
    fn new() -> Self {
        Self {
            profile: std::sync::atomic::AtomicU8::new(0), // 0 = PublicSecure, 1 = Development
            storage: "Aladdin JSA",
        }
    }
}

impl aladdin::contracts::system::SystemHost for MockSystem {
    fn stats(&self) -> aladdin::contracts::system::SystemStats {
        let prof = if self.profile.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            aladdin::contracts::system::JinnWallProfile::PublicSecure
        } else {
            aladdin::contracts::system::JinnWallProfile::Development
        };
        aladdin::contracts::system::SystemStats {
            uptime_secs: 42,
            task_count: 5,
            sched_ticks: 4200,
            used_ram_mb: 64,
            total_ram_mb: 512,
            current_tick: 4200,
            net_ip: [10, 0, 2, 15],
            net_adapter: "VirtIO-Net",
            net_state_desc: "Operacional",
            jinnwall_active_conns: 3,
            jinnwall_drops: 12,
            jinnwall_inspected: 150,
            jinnwall_accepted: 138,
            jinnwall_profile: prof,
            physical_frames_used: 16384,
            physical_frames_total: 131072,
            dynamic_pages_allocated: 20,
            dynamic_pages_freed: 5,
            vfs_open_fds: 3,
            vfs_mounted_volumes: 2,
        }
    }

    fn yield_cpu(&self) {}

    fn storage_profile_name(&self) -> &'static str {
        self.storage
    }

    fn cycle_storage_profile(&self) {}

    fn jinnwall_profile(&self) -> aladdin::contracts::system::JinnWallProfile {
        if self.profile.load(std::sync::atomic::Ordering::SeqCst) == 0 {
            aladdin::contracts::system::JinnWallProfile::PublicSecure
        } else {
            aladdin::contracts::system::JinnWallProfile::Development
        }
    }

    fn set_jinnwall_profile(&self, profile: aladdin::contracts::system::JinnWallProfile) {
        let val = match profile {
            aladdin::contracts::system::JinnWallProfile::PublicSecure => 0,
            aladdin::contracts::system::JinnWallProfile::Development => 1,
        };
        self.profile.store(val, std::sync::atomic::Ordering::SeqCst);
    }

    fn toggle_jinnwall_profile(&self) -> aladdin::contracts::system::JinnWallProfile {
        let cur = self.profile.load(std::sync::atomic::Ordering::SeqCst);
        let next = if cur == 0 { 1 } else { 0 };
        self.profile.store(next, std::sync::atomic::Ordering::SeqCst);
        if next == 0 {
            aladdin::contracts::system::JinnWallProfile::PublicSecure
        } else {
            aladdin::contracts::system::JinnWallProfile::Development
        }
    }

    fn reboot(&self) -> ! {
        panic!("Reboot called in test");
    }
}

#[test]
fn test_jinnwall_firewall_profile_and_telemetry() {
    use aladdin::contracts::system::{JinnWallProfile, SystemHost};

    let sys = MockSystem::new();
    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::PublicSecure);
    assert_eq!(sys.jinnwall_profile().name(), "PublicSecure");

    // Alterna perfil
    let p1 = sys.toggle_jinnwall_profile();
    assert_eq!(p1, JinnWallProfile::Development);
    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::Development);
    assert_eq!(sys.jinnwall_profile().name(), "Development");

    // Alterna novamente
    let p2 = sys.toggle_jinnwall_profile();
    assert_eq!(p2, JinnWallProfile::PublicSecure);

    // Set explicito
    sys.set_jinnwall_profile(JinnWallProfile::Development);
    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::Development);

    // Valida consistencia de SystemStats
    let stats = sys.stats();
    assert_eq!(stats.jinnwall_profile, JinnWallProfile::Development);
    assert_eq!(stats.jinnwall_drops, 12);
    assert_eq!(stats.jinnwall_active_conns, 3);
    assert_eq!(stats.jinnwall_inspected, 150);
    assert_eq!(stats.jinnwall_accepted, 138);
}

#[test]
fn test_settings_app_jinnwall_rendering_and_click_toggle() {
    use aladdin::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost};
    use aladdin::desktop::session::AladdinSession;

    let sys = MockSystem::new();
    let mut session = AladdinSession::new();

    // Janela 4 deve estar fechada inicialmente
    assert!(session.wm.windows[4].is_some());
    assert!(!session.wm.windows[4].as_ref().unwrap().is_open);

    // Abre janela 4
    if let Some(w) = session.wm.windows[4].as_mut() {
        w.is_open = true;
    }
    session.wm.bring_to_front(4);

    // Renderiza Ajustes para buffer
    let mut buf = alloc::vec![0u32; 1024 * 768];
    session.settings.render(
        &mut buf,
        1024,
        768,
        session.wm.windows[4].as_ref().unwrap().x,
        session.wm.windows[4].as_ref().unwrap().y,
        session.wm.windows[4].as_ref().unwrap().width,
        session.wm.windows[4].as_ref().unwrap().height,
        CapabilityMode::Balanced,
        &sys,
    );

    // Simula clique exatamente no botao 'Alternar Perfil Firewall'
    let win = session.wm.windows[4].as_ref().unwrap();
    let btn_x = win.x + 30;
    let btn_y = win.y + 166 + 40 + 5;

    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::PublicSecure);

    struct DummyProc;
    impl aladdin::contracts::process::ProcessHost for DummyProc {
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(1) }
        fn execute_command(&mut self, _cmd: &str, _cb: &mut dyn FnMut(&str, u32)) {}
    }

    struct DummyVfs;
    impl aladdin::contracts::vfs::VfsHost for DummyVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    let mut proc = DummyProc;
    let vfs = DummyVfs;

    let _ = session.handle_mouse(btn_x, btn_y, 1, &sys, &vfs, &mut proc, 1024, 768);
    // Apos clique, o perfil deve ter mudado para Development!
    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::Development);
}

#[test]
fn test_telemetry_consistency_and_zero_values() {
    use aladdin::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost, SystemStats};
    use aladdin::apps::monitor::SystemMonitorApp;
    use aladdin::desktop::session::AladdinSession;

    struct ZeroSystem;
    impl SystemHost for ZeroSystem {
        fn stats(&self) -> SystemStats {
            SystemStats {
                uptime_secs: 0,
                task_count: 0,
                sched_ticks: 0,
                used_ram_mb: 0,
                total_ram_mb: 512,
                current_tick: 0,
                net_ip: [0, 0, 0, 0],
                net_adapter: "Nenhum",
                net_state_desc: "Sem placa de rede",
                jinnwall_active_conns: 0,
                jinnwall_drops: 0,
                jinnwall_inspected: 0,
                jinnwall_accepted: 0,
                jinnwall_profile: JinnWallProfile::PublicSecure,
                physical_frames_used: 0,
                physical_frames_total: 131072,
                dynamic_pages_allocated: 0,
                dynamic_pages_freed: 0,
                vfs_open_fds: 0,
                vfs_mounted_volumes: 0,
            }
        }
        fn yield_cpu(&self) {}
        fn storage_profile_name(&self) -> &'static str { "Nenhum" }
        fn cycle_storage_profile(&self) {}
        fn jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn set_jinnwall_profile(&self, _p: JinnWallProfile) {}
        fn toggle_jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn reboot(&self) -> ! { panic!() }
    }

    let sys = ZeroSystem;
    let mut buf = alloc::vec![0u32; 1024 * 768];

    // Testa render do Monitor com valores zero (incluindo barra de progresso)
    SystemMonitorApp.render(&mut buf, 1024, 768, 60, 60, 380, 270, &sys.stats());

    // Testa render dos Ajustes com valores zero
    let session = AladdinSession::new();
    session.settings.render(&mut buf, 1024, 768, 200, 150, 440, 276, CapabilityMode::Basic, &sys);

    let stats = sys.stats();
    assert_eq!(stats.jinnwall_drops, 0);
    assert_eq!(stats.jinnwall_active_conns, 0);
    assert_eq!(stats.jinnwall_profile, JinnWallProfile::PublicSecure);
}

#[test]
fn test_settings_and_monitor_profile_toggle_synchronization() {
    use aladdin::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost};
    use aladdin::apps::monitor::SystemMonitorApp;
    use aladdin::desktop::session::AladdinSession;

    let sys = MockSystem::new();
    let session = AladdinSession::new();
    let mut buf = alloc::vec![0u32; 1024 * 768];

    // 1. Estado inicial: PublicSecure
    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::PublicSecure);
    assert_eq!(sys.stats().jinnwall_profile, JinnWallProfile::PublicSecure);

    SystemMonitorApp.render(&mut buf, 1024, 768, 60, 60, 380, 270, &sys.stats());
    session.settings.render(&mut buf, 1024, 768, 200, 150, 440, 276, CapabilityMode::Balanced, &sys);

    // 2. Alterna para Development
    let new_prof = sys.toggle_jinnwall_profile();
    assert_eq!(new_prof, JinnWallProfile::Development);
    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::Development);
    assert_eq!(sys.stats().jinnwall_profile, JinnWallProfile::Development);

    // Renderiza novamente ambos e verifica coerencia
    SystemMonitorApp.render(&mut buf, 1024, 768, 60, 60, 380, 270, &sys.stats());
    session.settings.render(&mut buf, 1024, 768, 200, 150, 440, 276, CapabilityMode::Balanced, &sys);

    // 3. Alterna de volta para PublicSecure
    let back_prof = sys.toggle_jinnwall_profile();
    assert_eq!(back_prof, JinnWallProfile::PublicSecure);
    assert_eq!(sys.jinnwall_profile(), JinnWallProfile::PublicSecure);
    assert_eq!(sys.stats().jinnwall_profile, JinnWallProfile::PublicSecure);
}

#[test]
fn test_system_stats_saturation_and_boundary_safety() {
    use aladdin::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost, SystemStats};
    use aladdin::apps::monitor::SystemMonitorApp;
    use aladdin::desktop::session::AladdinSession;

    struct ExtremeSystem;
    impl SystemHost for ExtremeSystem {
        fn stats(&self) -> SystemStats {
            SystemStats {
                uptime_secs: usize::MAX,
                task_count: usize::MAX,
                sched_ticks: u64::MAX,
                used_ram_mb: usize::MAX,
                total_ram_mb: usize::MAX,
                current_tick: u64::MAX,
                net_ip: [255, 255, 255, 255],
                net_adapter: "VirtIO-Net",
                net_state_desc: "Operacional",
                jinnwall_active_conns: usize::MAX,
                jinnwall_drops: u64::MAX,
                jinnwall_inspected: u64::MAX,
                jinnwall_accepted: u64::MAX,
                jinnwall_profile: JinnWallProfile::Development,
                physical_frames_used: usize::MAX,
                physical_frames_total: usize::MAX,
                dynamic_pages_allocated: usize::MAX,
                dynamic_pages_freed: usize::MAX,
                vfs_open_fds: usize::MAX,
                vfs_mounted_volumes: usize::MAX,
            }
        }
        fn yield_cpu(&self) {}
        fn storage_profile_name(&self) -> &'static str { "NVMe-Extreme" }
        fn cycle_storage_profile(&self) {}
        fn jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::Development }
        fn set_jinnwall_profile(&self, _p: JinnWallProfile) {}
        fn toggle_jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::Development }
        fn reboot(&self) -> ! { panic!() }
    }

    let sys = ExtremeSystem;
    let mut buf = alloc::vec![0u32; 1024 * 768];

    // Verifica que valores saturados nao causam panico ou overflow em tempo de renderizacao
    SystemMonitorApp.render(&mut buf, 1024, 768, 60, 60, 380, 270, &sys.stats());

    let session = AladdinSession::new();
    session.settings.render(&mut buf, 1024, 768, 200, 150, 440, 276, CapabilityMode::Advanced, &sys);
}


#[test]
fn test_client_window_event_queue_and_polling() {
    use aladdin::contracts::input::{
        ClientWindowEvent, EVENT_KEY_DOWN, EVENT_MOUSE_DOWN, EVENT_WINDOW_CLOSE,
    };
    use aladdin::wm::{WindowManager, WindowOpError};

    let mut wm = WindowManager::new();
    let pid = 42;
    let foreign_pid = 99;

    let win_id = wm.create_client_window(pid, "Event App", 20, 20, 200, 150).expect("Window ok");

    // 1. Fila inicialmente vazia
    assert_eq!(wm.poll_client_event(pid, win_id), Ok(None));

    // 2. Rejeição de consulta por processo não autorizado (cross-PID security)
    assert_eq!(wm.poll_client_event(foreign_pid, win_id), Err(WindowOpError::PermissionDenied));

    // 3. Rejeição de consulta em janela inexistente
    assert_eq!(wm.poll_client_event(pid, 999), Err(WindowOpError::NotFound));

    // 4. Enfileiramento de eventos (MouseDown, KeyDown, WindowClose)
    let ev1 = ClientWindowEvent {
        event_type: EVENT_MOUSE_DOWN,
        win_id: win_id as u32,
        x: 45,
        y: 80,
        button_or_key: 1,
        modifiers: 0,
        timestamp: 100,
    };
    let ev2 = ClientWindowEvent {
        event_type: EVENT_KEY_DOWN,
        win_id: win_id as u32,
        x: 0,
        y: 0,
        button_or_key: b'w' as u32,
        modifiers: 0,
        timestamp: 105,
    };
    let ev3 = ClientWindowEvent {
        event_type: EVENT_WINDOW_CLOSE,
        win_id: win_id as u32,
        x: 0,
        y: 0,
        button_or_key: 0,
        modifiers: 0,
        timestamp: 110,
    };

    assert!(wm.push_client_event(win_id, ev1));
    assert!(wm.push_client_event(win_id, ev2));
    assert!(wm.push_client_event(win_id, ev3));

    // 5. Coleta ordenada (FIFO)
    assert_eq!(wm.poll_client_event(pid, win_id), Ok(Some(ev1)));
    assert_eq!(wm.poll_client_event(pid, win_id), Ok(Some(ev2)));
    assert_eq!(wm.poll_client_event(pid, win_id), Ok(Some(ev3)));
    assert_eq!(wm.poll_client_event(pid, win_id), Ok(None));

    // 6. Limpeza automática da fila ao fechar a janela
    assert!(wm.push_client_event(win_id, ev1));
    wm.close_window(win_id);
    assert_eq!(wm.poll_client_event(pid, win_id), Err(WindowOpError::NotFound));
}
#[test]
fn test_client_window_event_queue_overflow_and_close_protection() {
    use aladdin::contracts::input::{
        ClientWindowEvent, EVENT_KEY_DOWN, EVENT_MOUSE_DOWN, EVENT_WINDOW_CLOSE, MAX_CLIENT_EVENTS,
    };
    use aladdin::wm::WindowManager;

    let mut wm = WindowManager::new();
    let pid = 10;
    let win_id = wm.create_client_window(pid, "Queue Test", 10, 10, 100, 100).expect("Window ok");

    // 1. Enche a fila com 16 eventos normais
    for i in 0..MAX_CLIENT_EVENTS {
        let ev = ClientWindowEvent {
            event_type: EVENT_KEY_DOWN,
            win_id: win_id as u32,
            x: 0,
            y: 0,
            button_or_key: b'a' as u32 + i as u32,
            modifiers: 0,
            timestamp: i as u64,
        };
        assert!(wm.push_client_event(win_id, ev));
    }
    assert!(wm.client_events[win_id].is_full());

    // 2. Enfileira EVENT_WINDOW_CLOSE em fila cheia: deve ser aceito e descartar o evento normal mais antigo
    let close_ev = ClientWindowEvent {
        event_type: EVENT_WINDOW_CLOSE,
        win_id: win_id as u32,
        x: 0,
        y: 0,
        button_or_key: 0,
        modifiers: 0,
        timestamp: 100,
    };
    assert!(wm.push_client_event(win_id, close_ev));
    assert!(wm.client_events[win_id].contains_close_event());

    // 3. Tenta inundar a fila com novos eventos normais
    for i in 0..20 {
        let ev = ClientWindowEvent {
            event_type: EVENT_MOUSE_DOWN,
            win_id: win_id as u32,
            x: i as i32,
            y: i as i32,
            button_or_key: 1,
            modifiers: 0,
            timestamp: 200 + i as u64,
        };
        wm.push_client_event(win_id, ev);
    }

    // 4. Garante que EVENT_WINDOW_CLOSE NUNCA foi descartado pela inundação de eventos
    assert!(wm.client_events[win_id].contains_close_event());

    // 5. Coleta até encontrar o close_ev
    let mut found_close = false;
    while let Ok(Some(ev)) = wm.poll_client_event(pid, win_id) {
        if ev.event_type == EVENT_WINDOW_CLOSE {
            found_close = true;
            break;
        }
    }
    assert!(found_close, "EVENT_WINDOW_CLOSE deve ser preservado mesmo sob transbordo");
}


#[test]
fn test_client_window_lifecycle_and_dispatch_in_session() {
    use aladdin::contracts::input::{
        EVENT_KEY_DOWN, EVENT_MOUSE_DOWN, EVENT_MOUSE_UP, EVENT_WINDOW_CLOSE,
    };
    use aladdin::contracts::system::{JinnWallProfile, SystemHost, SystemStats};
    use aladdin::contracts::vfs::VfsHost;
    use aladdin::contracts::process::ProcessHost;
    use aladdin::desktop::session::AladdinSession;

    struct DummySys;
    impl SystemHost for DummySys {
        fn stats(&self) -> SystemStats {
            SystemStats {
                uptime_secs: 10,
                task_count: 3,
                sched_ticks: 50,
                used_ram_mb: 64,
                total_ram_mb: 512,
                current_tick: 50,
                net_ip: [10, 0, 2, 15],
                net_adapter: "VirtIO-Net",
                net_state_desc: "Operacional",
                jinnwall_active_conns: 0,
                jinnwall_drops: 0,
                jinnwall_inspected: 0,
                jinnwall_accepted: 0,
                jinnwall_profile: JinnWallProfile::Development,
                physical_frames_used: 16384,
                physical_frames_total: 131072,
                dynamic_pages_allocated: 0,
                dynamic_pages_freed: 0,
                vfs_open_fds: 0,
                vfs_mounted_volumes: 0,
            }
        }
        fn yield_cpu(&self) {}
        fn storage_profile_name(&self) -> &'static str { "Test" }
        fn cycle_storage_profile(&self) {}
        fn jinnwall_profile(&self) -> JinnWallProfile {
            JinnWallProfile::Development
        }
        fn set_jinnwall_profile(&self, _p: JinnWallProfile) {}
        fn toggle_jinnwall_profile(&self) -> JinnWallProfile {
            JinnWallProfile::PublicSecure
        }
        fn reboot(&self) -> ! { loop {} }
    }

    struct DummyVfs;
    impl VfsHost for DummyVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    struct DummyProc;
    impl ProcessHost for DummyProc {
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(1) }
        fn execute_command(&mut self, _cmd: &str, _cb: &mut dyn FnMut(&str, u32)) {}
    }

    let mut session = AladdinSession::new();
    let sys = DummySys;
    let vfs = DummyVfs;
    let mut proc = DummyProc;
    let pid = 77;

    let win_id = session.create_client_window(pid, "Lifecycle Window", 200, 150).expect("Window created");
    let (wx, wy, w_w) = {
        let win = session.wm.windows[win_id].as_ref().unwrap();
        (win.x, win.y, win.width)
    };

    // 1. Clique dentro da área cliente -> Gera EVENT_MOUSE_DOWN
    let client_click_x = wx + 30;
    let client_click_y = wy + 40;
    session.handle_mouse(client_click_x, client_click_y, 1, &sys, &vfs, &mut proc, 1024, 768);

    let ev = session.poll_client_event(pid, win_id).expect("poll ok").expect("event found");
    assert_eq!(ev.event_type, EVENT_MOUSE_DOWN);
    assert_eq!(ev.button_or_key, 1);
    assert_eq!(ev.x, 29); // wx + 30 - (wx + 1)
    assert_eq!(ev.y, 15); // wy + 40 - (wy + 24 + 1)

    // 2. Solta o botão esquerdo -> Gera EVENT_MOUSE_UP
    session.handle_mouse(client_click_x, client_click_y, 0, &sys, &vfs, &mut proc, 1024, 768);
    let ev_up = session.poll_client_event(pid, win_id).expect("poll ok").expect("event up found");
    assert_eq!(ev_up.event_type, EVENT_MOUSE_UP);

    // 3. Pressiona tecla com janela cliente focada -> Gera EVENT_KEY_DOWN
    session.handle_key(b'k', &mut proc, &sys, &vfs);
    let ev_key = session.poll_client_event(pid, win_id).expect("poll ok").expect("event key found");
    assert_eq!(ev_key.event_type, EVENT_KEY_DOWN);
    assert_eq!(ev_key.button_or_key, b'k' as u32);

    // 4. Clique no botão [X] de fechar
    let close_btn_x = wx + w_w - 16;
    let close_btn_y = wy + 8;
    session.handle_mouse(close_btn_x, close_btn_y, 1, &sys, &vfs, &mut proc, 1024, 768);

    // Janela fica oculta imediatamente mas NÃO é destruída ainda
    let win_state = session.wm.windows[win_id].as_ref().expect("Janela deve existir até close");
    assert!(!win_state.is_open);
    assert!(win_state.close_requested);

    // Processo cliente recebe EVENT_WINDOW_CLOSE
    let ev_close = session.poll_client_event(pid, win_id).expect("poll ok").expect("close event found");
    assert_eq!(ev_close.event_type, EVENT_WINDOW_CLOSE);

    // 5. Processo cliente confirma e encerra a janela via syscall
    assert!(session.close_client_window(pid, win_id).is_ok());

    // Agora sim a janela é completamente destruída
    assert_eq!(session.poll_client_event(pid, win_id), Err(aladdin::wm::WindowOpError::NotFound));
}

#[test]
fn test_browser_instance_identification_and_lifecycle() {
    use aladdin::contracts::process::ProcessHost;
    use aladdin::desktop::session::AladdinSession;

    struct TestProc {
        spawned_path: Option<alloc::string::String>,
        spawn_fail: bool,
        alive_pid: Option<usize>,
    }

    impl ProcessHost for TestProc {
        fn spawn_app(&mut self, path: &str) -> Result<usize, ()> {
            self.spawned_path = Some(alloc::string::String::from(path));
            if self.spawn_fail {
                Err(())
            } else {
                self.alive_pid = Some(105);
                Ok(105)
            }
        }
        fn execute_command(&mut self, _cmd: &str, _cb: &mut dyn FnMut(&str, u32)) {}
        fn is_process_alive(&self, pid: usize) -> bool {
            self.alive_pid == Some(pid)
        }
    }

    let mut session = AladdinSession::new();
    let mut proc = TestProc {
        spawned_path: None,
        spawn_fail: false,
        alive_pid: None,
    };

    // 1. Inicialmente sem navegador
    assert_eq!(session.find_browser_window_id(), None);
    assert_eq!(session.browser_pid, None);
    assert_eq!(session.browser_win_id, None);

    // 2. Outra aplicação abre janela cliente (slot 6, pois 0..5 são janelas do sistema)
    let foreign_pid = 42;
    let foreign_win_id = session.create_client_window(foreign_pid, "Editor de Texto", 300, 200)
        .expect("Janela de outra aplicacao criada");
    assert_eq!(foreign_win_id, 6);

    // A janela 6 NÃO deve ser identificada como navegador!
    assert_eq!(session.find_browser_window_id(), None);
    assert_eq!(session.browser_pid, None);
    assert_eq!(session.browser_win_id, None);

    // 2.1 Adversarial: Processo impostor (PID 666) tenta se passar por navegador pelo titulo
    let _rogue_win_id = session.create_client_window(666, "Jinn Browser v0.3.0 (Fake)", 300, 200)
        .expect("Janela impostora criada");
    // O titulo NAO pode enganar a sessao nem associar o navegador sem spawn_app legitimo!
    assert_eq!(session.find_browser_window_id(), None);
    assert_eq!(session.browser_pid, None);
    assert_eq!(session.browser_win_id, None);

    // 3. Lançamento do navegador via sessão
    session.launch_or_focus_browser(&mut proc);
    assert_eq!(proc.spawned_path.as_deref(), Some("/disk/browser.elf"));
    assert_eq!(session.browser_pid, Some(105));
    assert_eq!(session.browser_win_id, None); // Ainda não criou janela

    // 4. Processo do navegador cria sua janela (slot 8)
    let browser_win_id = session.create_client_window(105, "Jinn Browser v0.3.0", 300, 180)
        .expect("Janela do browser criada");
    assert_eq!(browser_win_id, 8);

    // Identificação explícita do navegador deve retornar o slot 8, e NÃO o slot 6
    assert_eq!(session.find_browser_window_id(), Some(8));
    assert_eq!(session.browser_win_id, Some(8));

    // 5. Tentativa de novo lançamento não duplica o processo
    proc.spawned_path = None;
    session.launch_or_focus_browser(&mut proc);
    // NÃO deve ter chamado spawn_app novamente
    assert_eq!(proc.spawned_path, None);
    // Janela deve estar no topo
    assert_eq!(session.wm.top_window_id(), 8);

    // 6. Minimização e restauração
    let w = session.wm.windows[8].as_mut().unwrap();
    w.is_minimized = true;
    assert_eq!(session.find_browser_window_id(), Some(8));

    session.launch_or_focus_browser(&mut proc);
    let w_restored = session.wm.windows[8].as_ref().unwrap();
    assert!(!w_restored.is_minimized);
    assert!(w_restored.is_open);

    // 7. Fechamento da janela do navegador
    assert!(session.close_client_window(105, 8).is_ok());
    assert_eq!(session.find_browser_window_id(), None);
    assert_eq!(session.browser_win_id, None);

    // 8. Fechamento do processo limpa estado mesmo se a janela já foi fechada
    session.close_windows_for_pid(105);
    assert_eq!(session.browser_pid, None);

    // 9. Lançamento quando spawn_app falha não propaga falso sucesso
    proc.spawn_fail = true;
    session.launch_or_focus_browser(&mut proc);
    assert_eq!(session.browser_pid, None);
    assert_eq!(session.browser_win_id, None);
    assert_eq!(session.find_browser_window_id(), None);
}

#[test]
fn test_browser_external_pid_registration_and_key_delivery() {
    use aladdin::contracts::process::ProcessHost;
    use aladdin::desktop::session::AladdinSession;

    struct TestProc;
    impl ProcessHost for TestProc {
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(300) }
        fn execute_command(&mut self, _cmd: &str, _cb: &mut dyn FnMut(&str, u32)) {}
        fn is_process_alive(&self, _pid: usize) -> bool { true }
    }

    struct DummyVfs;
    impl aladdin::contracts::vfs::VfsHost for DummyVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    let mut session = AladdinSession::new();
    let mut proc = TestProc;
    let sys = MockSystem::new();
    let vfs = DummyVfs;

    // 1. Registro de PID externo (iniciado via terminal ou shell)
    session.register_browser_pid(300);
    assert_eq!(session.browser_pid, Some(300));
    assert_eq!(session.browser_win_id, None);

    // 2. Janela criada pelo processo registrado
    let win_id = session.create_client_window(300, "Jinn Browser v0.3.0", 300, 180)
        .expect("Janela criada com sucesso");
    assert_eq!(session.find_browser_window_id(), Some(win_id));
    assert_eq!(session.browser_win_id, Some(win_id));

    // 3. Entrega de tecla ESC para a janela cliente ativa sem derrubar a sessao do desktop
    session.handle_key(0x1B, &mut proc, &sys, &vfs);
    assert_eq!(session.should_exit, false, "ESC nao deve encerrar o desktop quando janela cliente tem foco");

    let ev = session.poll_client_event(300, win_id).expect("poll ok").expect("evento presente");
    assert_eq!(ev.event_type, aladdin::contracts::input::EVENT_KEY_DOWN);
    assert_eq!(ev.button_or_key, 0x1B);

    // 4. Entrega de caracteres ASCII imprimiveis
    session.handle_key(b'x', &mut proc, &sys, &vfs);
    let ev_char = session.poll_client_event(300, win_id).expect("poll ok").expect("evento presente");
    assert_eq!(ev_char.event_type, aladdin::contracts::input::EVENT_KEY_DOWN);
    assert_eq!(ev_char.button_or_key, b'x' as u32);

    // 5. Fechamento da janela e comportamento subsequente do ESC no console
    assert!(session.close_client_window(300, win_id).is_ok());
    assert_eq!(session.find_browser_window_id(), None);

    // Com foco no terminal, ESC deve sinalizar saida para o console
    session.handle_key(0x1B, &mut proc, &sys, &vfs);
    assert_eq!(session.should_exit, true, "ESC deve sair para o console quando nenhuma janela cliente tem foco");
}


#[test]
fn test_two_windows_input_focus_and_event_isolation() {
    use aladdin::contracts::process::ProcessHost;
    use aladdin::contracts::input::{EVENT_KEY_DOWN, EVENT_MOUSE_DOWN};
    use aladdin::desktop::session::AladdinSession;

    struct TestProc;
    impl ProcessHost for TestProc {
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(300) }
        fn execute_command(&mut self, _cmd: &str, _cb: &mut dyn FnMut(&str, u32)) {}
        fn is_process_alive(&self, _pid: usize) -> bool { true }
    }

    struct DummyVfs;
    impl aladdin::contracts::vfs::VfsHost for DummyVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    let mut session = AladdinSession::new();
    let mut proc = TestProc;
    let sys = MockSystem::new();
    let vfs = DummyVfs;

    // 1. Criar Janela A (Navegador, PID 300)
    session.register_browser_pid(300);
    let win_browser = session.create_client_window(300, "Jinn Browser v0.3.0", 300, 200)
        .expect("Janela do navegador criada com sucesso");
    assert_eq!(session.find_browser_window_id(), Some(win_browser));

    // 2. Criar Janela B (Outra aplicacao / Editor, PID 400)
    let win_editor = session.create_client_window(400, "Jinn Text Editor", 320, 220)
        .expect("Janela do editor criada com sucesso");

    // Como win_editor foi criada por ultimo, ela esta no topo do Z-order (tem o foco ativo)
    assert_eq!(session.wm.top_window_id(), win_editor);

    // 3. Digitar teclas quando o editor esta no topo:
    // Os eventos de teclado devem ser entregues EXCLUSIVAMENTE ao Editor (PID 400),
    // e o Navegador (PID 300) NUNCA deve receber esses eventos!
    session.handle_key(b'A', &mut proc, &sys, &vfs);
    session.handle_key(b'B', &mut proc, &sys, &vfs);

    // O editor deve conter os 2 eventos
    let ev1 = session.poll_client_event(400, win_editor).expect("poll editor ok").expect("ev1");
    assert_eq!(ev1.event_type, EVENT_KEY_DOWN);
    assert_eq!(ev1.button_or_key, b'A' as u32);

    let ev2 = session.poll_client_event(400, win_editor).expect("poll editor ok").expect("ev2");
    assert_eq!(ev2.event_type, EVENT_KEY_DOWN);
    assert_eq!(ev2.button_or_key, b'B' as u32);

    // Fila do navegador DEVE ESTAR RIGOROSAMENTE VAZIA!
    let ev_browser = session.poll_client_event(300, win_browser).expect("poll browser ok");
    assert!(ev_browser.is_none(), "Navegador nao deve receber eventos direcionados a outra aplicacao!");

    // 4. Tentativa de violacao de isolamento: PID 300 tenta consultar a fila da janela do editor
    let forbidden_poll = session.poll_client_event(300, win_editor);
    assert_eq!(
        forbidden_poll,
        Err(aladdin::wm::WindowOpError::PermissionDenied),
        "Processo nao autorizado nao pode ler eventos de janelas de terceiros"
    );

    // 5. Alternancia de Foco: Clicar na area da janela do Navegador para trazê-la para o topo
    let (bx, by) = {
        let w = session.wm.windows[win_browser].as_ref().unwrap();
        (w.x + 10, w.y + 10)
    };
    session.handle_mouse(bx, by, 1, &sys, &vfs, &mut proc, 1024, 768);
    session.handle_mouse(bx, by, 0, &sys, &vfs, &mut proc, 1024, 768);

    // Agora o navegador deve estar no topo
    assert_eq!(session.wm.top_window_id(), win_browser);

    // 6. Digitar teclas com foco no Navegador: agora os eventos vao para o Navegador!
    session.handle_key(b'z', &mut proc, &sys, &vfs);

    let ev_nav = session.poll_client_event(300, win_browser).expect("poll browser ok").expect("ev_nav");
    assert_eq!(ev_nav.event_type, EVENT_KEY_DOWN);
    assert_eq!(ev_nav.button_or_key, b'z' as u32);

    // O editor agora NAO deve receber este evento
    let ev_ed_empty = session.poll_client_event(400, win_editor).expect("poll editor ok");
    assert!(ev_ed_empty.is_none(), "Editor nao deve receber eventos direcionados ao navegador!");

    // 7. Fechamento de janela com eventos pendentes:
    session.wm.push_client_event(win_editor, aladdin::contracts::input::ClientWindowEvent {
        event_type: EVENT_MOUSE_DOWN,
        win_id: win_editor as u32,
        x: 10,
        y: 10,
        button_or_key: 1,
        modifiers: 0,
        timestamp: 100,
    });
    assert!(session.close_client_window(400, win_editor).is_ok());

    // Janela fechada limpa sua fila e nao afeta o navegador
    assert_eq!(session.poll_client_event(400, win_editor), Err(aladdin::wm::WindowOpError::NotFound));
    assert!(session.wm.windows[win_browser].is_some());
}


#[test]
fn test_disk_manager_app_and_storage_contract() {
    use aladdin::apps::DiskManagerApp;
    use aladdin::contracts::storage::{StorageHost, DiskInfo, DiskBus, VolumeDisplayInfo};

    struct MockStorage {
        unmounted_ids: Vec<usize>,
        formatted_ids: Vec<usize>,
    }

    impl StorageHost for MockStorage {
        fn disk_count(&self) -> usize { 2 }
        fn get_disk(&self, index: usize) -> Option<DiskInfo> {
            if index == 0 {
                let mut d = DiskInfo::empty();
                let n = b"nvme0n1";
                d.name[..n.len()].copy_from_slice(n);
                d.name_len = n.len();
                d.bus = DiskBus::NvmePci;
                d.capacity_mb = 512;
                d.is_system_disk = true;
                Some(d)
            } else if index == 1 {
                let mut d = DiskInfo::empty();
                let n = b"sata0";
                d.name[..n.len()].copy_from_slice(n);
                d.name_len = n.len();
                d.bus = DiskBus::AhciSata;
                d.capacity_mb = 1024;
                d.is_system_disk = false;
                Some(d)
            } else {
                None
            }
        }

        fn volume_count(&self) -> usize { 2 }
        fn get_volume(&self, index: usize) -> Option<VolumeDisplayInfo> {
            if index == 0 {
                let mut v = VolumeDisplayInfo::empty();
                v.id = 0;
                let l = b"ROOT";
                v.label[..l.len()].copy_from_slice(l);
                v.label_len = l.len();
                let m = b"/";
                v.mount_point[..m.len()].copy_from_slice(m);
                v.mount_point_len = m.len();
                v.total_mb = 256;
                v.used_mb = 50;
                v.free_mb = 206;
                v.is_mounted = true;
                v.is_root = true;
                Some(v)
            } else if index == 1 {
                let mut v = VolumeDisplayInfo::empty();
                v.id = 1;
                let l = b"DATA";
                v.label[..l.len()].copy_from_slice(l);
                v.label_len = l.len();
                let m = b"/data";
                v.mount_point[..m.len()].copy_from_slice(m);
                v.mount_point_len = m.len();
                v.total_mb = 500;
                v.used_mb = 100;
                v.free_mb = 400;
                v.is_mounted = true;
                v.is_root = false;
                Some(v)
            } else {
                None
            }
        }

        fn mount_volume(&mut self, _volume_id: usize, _mount_point: &str) -> Result<(), &'static str> { Ok(()) }
        fn unmount_volume(&mut self, volume_id: usize) -> Result<(), &'static str> {
            if volume_id == 0 {
                Err("EBUSY: Nao e permitido desmontar o volume raiz")
            } else {
                self.unmounted_ids.push(volume_id);
                Ok(())
            }
        }
        fn format_volume(&mut self, volume_id: usize, _fs: &str) -> Result<(), &'static str> {
            if volume_id == 0 {
                Err("EPERM: Nao e permitido formatar o volume raiz do sistema")
            } else {
                self.formatted_ids.push(volume_id);
                Ok(())
            }
        }
    }

    let mut app = DiskManagerApp::new();
    let mut storage = MockStorage {
        unmounted_ids: Vec::new(),
        formatted_ids: Vec::new(),
    };

    // 1. Testa render em buffer
    let mut fb = vec![0u32; 1024 * 768];
    app.render(&mut fb, 1024, 768, 100, 100, 520, 310, &storage);

    // 2. Seleciona disco 1 (sata0)
    // Coordenada Y relativa de seleção: 54 + (1 * 18) = 72
    app.handle_click(50, 72, 520, 310, &mut storage);
    assert_eq!(app.selected_disk, 1);

    // 3. Seleciona volume 0 (root) e tenta desmontar
    app.handle_click(50, 140, 520, 310, &mut storage);
    assert_eq!(app.selected_volume, 0);

    // Botão Desmontar: rel_x: 114..202, rel_y: 222..248
    app.handle_click(150, 230, 520, 310, &mut storage);
    assert!(app.status_is_error);
    assert!(app.status_str().contains("raiz"));
    assert!(storage.unmounted_ids.is_empty());

    // 4. Seleciona volume 1 (data) e desmonta
    app.handle_click(50, 163, 520, 310, &mut storage);
    assert_eq!(app.selected_volume, 1);
    app.handle_click(150, 230, 520, 310, &mut storage);
    assert!(!app.status_is_error);
    assert_eq!(storage.unmounted_ids, vec![1]);

    // 5. Formatar: clica no botão Formatar (rel_x: 218..314, rel_y: 222..248)
    app.handle_click(250, 230, 520, 310, &mut storage);
    // Modal de confirmação DEVE estar aberto!
    assert!(app.confirm_dialog_open);
    assert_eq!(app.action_volume_id, Some(1));

    // 5.1 Cancela formatação
    // Botão Cancelar no modal: dx + 180 .. dx + 320, dy + 88 .. dy + 114
    let dw = 360;
    let dh = 130;
    let dx = (520 / 2) - (dw / 2); // 260 - 180 = 80
    let dy = (310 / 2) - (dh / 2); // 155 - 65 = 90
    app.handle_click(dx + 200, dy + 95, 520, 310, &mut storage);
    assert!(!app.confirm_dialog_open);
    assert!(storage.formatted_ids.is_empty());
    assert!(app.status_str().contains("cancelada"));

    // 5.2 Abre novamente e confirma formatação
    app.handle_click(250, 230, 520, 310, &mut storage);
    assert!(app.confirm_dialog_open);
    app.handle_click(dx + 50, dy + 95, 520, 310, &mut storage);
    assert!(!app.confirm_dialog_open);
    assert_eq!(storage.formatted_ids, vec![1]);
    assert!(!app.status_is_error);
    assert!(app.status_str().contains("sucesso"));
}


#[test]
fn test_aladdin_design_system_tokens() {
    use aladdin::compositor::*;

    // Valida elevação de superfícies
    assert_ne!(COLOR_SURFACE_L0, COLOR_SURFACE_L1);
    assert_ne!(COLOR_SURFACE_L1, COLOR_SURFACE_L2);
    assert_ne!(COLOR_SURFACE_L2, COLOR_SURFACE_L3);

    // Valida compatibilidade retroativa
    assert_eq!(COLOR_BG, COLOR_SURFACE_L0);
    assert_eq!(COLOR_CARD, COLOR_SURFACE_L2);
    assert_eq!(COLOR_CARD_ALT, COLOR_SURFACE_L3);

    // Valida tokens de acento Garuda Dr460nized & elementary OS
    assert_eq!(COLOR_CYAN, 0x0038_BDF8);
    assert_eq!(COLOR_CYAN_NEON, 0x0038_BDF8);
    assert_eq!(COLOR_PURPLE, 0x00A8_55F7);
    assert_eq!(COLOR_EMERALD, 0x0022_C55E);
    assert_eq!(COLOR_AMBER, 0x00F5_9E0B);
    assert_eq!(COLOR_RUBY, 0x00EF_4444);
    assert_eq!(COLOR_BORDER_ACT, 0x0038_BDF8);

    // Valida tipografia de alto contraste
    assert_eq!(COLOR_TEXT_PRI, 0x00F8_FAFC);
    assert_eq!(COLOR_TEXT_SEC, 0x00CBD5E1);
    assert_eq!(COLOR_TEXT_MUTED, 0x0094_A3B8);
}

#[test]
fn test_procedural_wallpaper_generation() {
    use aladdin::compositor::*;
    use aladdin::contracts::system::CapabilityMode;

    let w = 800;
    let h = 600;
    let mut buf = vec![0u32; w * h];

    // 1. Basic Mode: Gradiente simples, rápido
    render_procedural_wallpaper(&mut buf, w, h, CapabilityMode::Basic);
    assert_ne!(buf[0], 0);
    assert_ne!(buf[w * h - 1], 0);
    // Topo deve ser mais escuro que a base
    assert!(buf[0] < buf[w * (h - 1)]);

    // 2. Balanced Mode: Gradiente + linhas de grade sutis
    render_procedural_wallpaper(&mut buf, w, h, CapabilityMode::Balanced);
    assert_ne!(buf[0], 0);

    // 3. Advanced Mode: Gradiente + pontos de constelação ciano neon
    render_procedural_wallpaper(&mut buf, w, h, CapabilityMode::Advanced);
    let mut found_neon = false;
    for &p in buf.iter() {
        if p == COLOR_CYAN_NEON {
            found_neon = true;
            break;
        }
    }
    assert!(found_neon, "Advanced mode deve renderizar estrelas/pontos de grade com acento neon");
}

#[test]
fn test_ui_component_library() {
    use aladdin::ui::*;

    let w = 640;
    let h = 480;
    let mut buf = vec![0u32; w * h];

    // 1. Botões em diversos estilos e estados
    draw_button(&mut buf, w, h, 20, 20, 100, 30, "Salvar", ButtonStyle::Primary, WidgetState::Normal);
    draw_button(&mut buf, w, h, 130, 20, 100, 30, "Cancelar", ButtonStyle::Secondary, WidgetState::Hover);
    draw_button(&mut buf, w, h, 240, 20, 100, 30, "Sucesso", ButtonStyle::Success, WidgetState::Active);
    draw_button(&mut buf, w, h, 350, 20, 100, 30, "Excluir", ButtonStyle::Danger, WidgetState::Normal);
    draw_button(&mut buf, w, h, 460, 20, 100, 30, "Ghost", ButtonStyle::Ghost, WidgetState::Disabled);

    // 2. Input de texto
    draw_text_input(&mut buf, w, h, 20, 70, 200, 28, "jinn://home", true, 4);

    // 3. Badges de status
    draw_badge(&mut buf, w, h, 20, 120, "Ativo", 0x0022_C55E, 0x000F_172A);
    draw_badge(&mut buf, w, h, 100, 120, "Alerta", 0x00F5_9E0B, 0x000F_172A);

    // 4. TabBar
    let tabs = ["Geral", "Rede", "Seguranca", "Armazenamento"];
    draw_tab_bar(&mut buf, w, h, 20, 170, 400, 32, &tabs, 1);

    // 5. Tooltip
    draw_tooltip(&mut buf, w, h, 100, 220, "Clique para configurar");

    // 6. Dialog
    draw_dialog(&mut buf, w, h, 150, 260, 340, 150, "Confirmacao", "Deseja salvar as alteracoes?");

    // Verifica que o buffer recebeu dados não nulos
    let non_zero_pixels = buf.iter().filter(|&&p| p != 0).count();
    assert!(non_zero_pixels > 5000, "Componentes devem desenhar graficos no buffer");
}

#[test]
fn test_modern_window_chrome_and_dock_rendering() {
    use aladdin::wm::{Window, WindowManager};
    use aladdin::desktop::dock::render_dock;
    use aladdin::contracts::system::CapabilityMode;
    use aladdin::compositor::*;

    let w = 800;
    let h = 600;
    let mut buf = vec![0u32; w * h];
    let mut wm = WindowManager::new();

    let win = Window::new(0, "Terminal", 50, 50, 400, 300);
    wm.windows[0] = Some(win);

    // Renderiza cromo no modo Advanced
    wm.render_window_chrome(&mut buf, w, h, &win, true, CapabilityMode::Advanced);

    // Deve ter botões de semáforo
    let amber_found = buf.iter().any(|&p| p == COLOR_AMBER);
    let emerald_found = buf.iter().any(|&p| p == COLOR_EMERALD);
    let ruby_found = buf.iter().any(|&p| p == COLOR_RUBY);
    assert!(amber_found && emerald_found && ruby_found, "Cromo deve conter os 3 botoes de semaforo");

    // Renderiza Dock
    render_dock(&mut buf, w, h, 400, 550, &wm, None);

    // O dock deve desenhar borda e indicadores
    let cyan_found = buf.iter().any(|&p| p == COLOR_CYAN_NEON);
    assert!(cyan_found, "Dock deve conter indicadores e bordas luminescentes");
}

#[test]
fn test_file_manager_volume_switching_and_read_only_protection() {
    use aladdin::apps::files::FileManagerApp;
    use aladdin::contracts::vfs::VfsHost;

    struct TestVfs;
    impl VfsHost for TestVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    let mut fm = FileManagerApp::new();
    let vfs = TestVfs;

    // Inicia em /disk (FAT32, Leitura e Escrita)
    assert_eq!(fm.active_volume, 0);
    assert_eq!(fm.current_path_str(), "/disk");
    assert!(!fm.is_read_only());

    // Alterna para volume 1 (/ext2, Somente-leitura)
    fm.select_volume(1, &vfs);
    assert_eq!(fm.active_volume, 1);
    assert_eq!(fm.current_path_str(), "/ext2");
    assert!(fm.is_read_only());

    // Tentativas de modificação em volume RO devem ser rejeitadas
    assert!(fm.create_folder("NOVA_DIR", &vfs).is_err());
    assert!(fm.create_file("NOVO_ARQ", &vfs).is_err());
    assert!(fm.rename_selected("REN_DIR", &vfs).is_err());
    assert!(fm.delete_selected(&vfs).is_err());
    assert!(fm.status_str().contains("somente-leitura"));

    // Retorna para volume 0 (/disk)
    fm.select_volume(0, &vfs);
    assert_eq!(fm.active_volume, 0);
    assert_eq!(fm.current_path_str(), "/disk");
    assert!(!fm.is_read_only());
}

#[test]
fn test_file_manager_directory_navigation() {
    use aladdin::apps::files::FileManagerApp;
    use aladdin::contracts::vfs::VfsHost;

    struct NavVfs;
    impl VfsHost for NavVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    let mut fm = FileManagerApp::new();
    let vfs = NavVfs;

    assert_eq!(fm.current_path_str(), "/disk");

    // Navega para subdiretório
    fm.navigate_into("DOCS", &vfs);
    assert_eq!(fm.current_path_str(), "/disk/DOCS");

    // Navega para subdiretório mais profundo
    fm.navigate_into("TEXTS", &vfs);
    assert_eq!(fm.current_path_str(), "/disk/DOCS/TEXTS");

    // Sobe um nível
    fm.navigate_up(&vfs);
    assert_eq!(fm.current_path_str(), "/disk/DOCS");

    // Sobe para a raiz
    fm.navigate_up(&vfs);
    assert_eq!(fm.current_path_str(), "/disk");

    // Sobe além da raiz deve permanecer na raiz sem erro ou overflow
    fm.navigate_up(&vfs);
    assert_eq!(fm.current_path_str(), "/disk");
}

#[test]
fn test_file_manager_crud_operations_and_dialog_lifecycle() {
    use aladdin::apps::files::{FileManagerApp, FileDialogKind};
    use aladdin::contracts::vfs::VfsHost;
    use std::sync::atomic::{AtomicBool, Ordering};

    static DIR_CREATED: AtomicBool = AtomicBool::new(false);
    static FILE_CREATED: AtomicBool = AtomicBool::new(false);
    static FILE_REMOVED: AtomicBool = AtomicBool::new(false);
    static FILE_RENAMED: AtomicBool = AtomicBool::new(false);

    struct CrudVfs;
    impl VfsHost for CrudVfs {
        fn readdir(&self, _path: &str, cb: &mut dyn FnMut(&str, bool, bool, bool)) {
            cb("ARQUIVO.TXT", false, true, false);
        }
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn create_dir(&self, path: &str) -> Result<(), ()> {
            if path == "/disk/PASTA" {
                DIR_CREATED.store(true, Ordering::SeqCst);
                Ok(())
            } else {
                Err(())
            }
        }
        fn create_file(&self, path: &str) -> Result<(), ()> {
            if path == "/disk/NOVO.TXT" {
                FILE_CREATED.store(true, Ordering::SeqCst);
                Ok(())
            } else {
                Err(())
            }
        }
        fn remove_file(&self, path: &str) -> Result<(), ()> {
            if path == "/disk/ARQUIVO.TXT" {
                FILE_REMOVED.store(true, Ordering::SeqCst);
                Ok(())
            } else {
                Err(())
            }
        }
        fn rename_file(&self, old_path: &str, new_path: &str) -> Result<(), ()> {
            if old_path == "/disk/ARQUIVO.TXT" && new_path == "/disk/RENAMED.TXT" {
                FILE_RENAMED.store(true, Ordering::SeqCst);
                Ok(())
            } else {
                Err(())
            }
        }
    }

    let mut fm = FileManagerApp::new();
    let vfs = CrudVfs;
    fm.refresh_cache(&vfs);

    // 1. Criar pasta
    assert!(fm.create_folder("PASTA", &vfs).is_ok());
    assert!(DIR_CREATED.load(Ordering::SeqCst));

    // 2. Criar arquivo
    assert!(fm.create_file("NOVO.TXT", &vfs).is_ok());
    assert!(FILE_CREATED.load(Ordering::SeqCst));

    // 3. Selecionar item
    fm.selected_index = Some(0);

    // 4. Renomear item
    assert!(fm.rename_selected("RENAMED.TXT", &vfs).is_ok());
    assert!(FILE_RENAMED.load(Ordering::SeqCst));

    // Seleciona novamente para excluir (rename reseta selecao ao atualizar cache)
    fm.selected_index = Some(0);

    // 5. Excluir item
    assert!(fm.delete_selected(&vfs).is_ok());
    assert!(FILE_REMOVED.load(Ordering::SeqCst));

    // 6. Testar Diálogos e Input
    fm.open_dialog(FileDialogKind::NewFolder);
    assert_eq!(fm.dialog_kind, FileDialogKind::NewFolder);
    fm.handle_key(b'M', &vfs);
    fm.handle_key(b'Y', &vfs);
    assert_eq!(&fm.input_buffer[..fm.input_len], b"MY");
    fm.handle_key(8, &vfs); // Backspace
    assert_eq!(&fm.input_buffer[..fm.input_len], b"M");
    fm.handle_key(0x1B, &vfs); // ESC cancela
    assert_eq!(fm.dialog_kind, FileDialogKind::None);

    // 7. Testar Diálogo de Propriedades
    fm.selected_index = Some(0);
    fm.open_dialog(FileDialogKind::Properties);
    assert_eq!(fm.dialog_kind, FileDialogKind::Properties);
    fm.handle_key(b'\n', &vfs); // Enter fecha o diálogo de propriedades
    assert_eq!(fm.dialog_kind, FileDialogKind::None);

    // 8. Testar Clique do Mouse no Botão Primário do Diálogo Modal (Criar Pasta e Criar Arquivo)
    struct DummyProc;
    impl aladdin::contracts::process::ProcessHost for DummyProc {
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(1) }
        fn execute_command(&mut self, _cmd: &str, _cb: &mut dyn FnMut(&str, u32)) {}
    }
    let mut proc = DummyProc;
    let wx: usize = 50; let wy: usize = 50; let ww: usize = 600; let wh: usize = 400;

    // A) Diálogo Criar Pasta via Mouse Click
    fm.dialog_kind = FileDialogKind::NewFolder;
    let def_folder = b"PASTA";
    fm.input_buffer[..def_folder.len()].copy_from_slice(def_folder);
    fm.input_len = def_folder.len();

    DIR_CREATED.store(false, Ordering::SeqCst);
    let dw = aladdin::apps::files::DIALOG_W;
    let dh = aladdin::apps::files::DIALOG_H_NORMAL;
    let dx = wx + (ww.saturating_sub(dw)) / 2;
    let dy = wy + (wh.saturating_sub(dh)) / 2;
    let click_btn_x = dx + 24; // Dentro de [dx+16, dx+156]
    let click_btn_y = dy + dh - 16; // Dentro de [dy+dh-28, dy+dh-6]

    let handled = fm.handle_mouse_click(click_btn_x, click_btn_y, wx, wy, ww, wh, &vfs, &mut proc);
    assert!(handled, "Clique dentro do dialogo deve ser consumido");
    assert!(DIR_CREATED.load(Ordering::SeqCst), "Clique no botao 'Criar Pasta' deve criar a pasta no VFS!");
    assert_eq!(fm.dialog_kind, FileDialogKind::None, "Dialogo deve fechar apos criacao da pasta!");

    // B) Diálogo Criar Arquivo via Mouse Click
    fm.dialog_kind = FileDialogKind::NewFile;
    let def_file = b"NOVO.TXT";
    fm.input_buffer[..def_file.len()].copy_from_slice(def_file);
    fm.input_len = def_file.len();

    FILE_CREATED.store(false, Ordering::SeqCst);
    let handled_file = fm.handle_mouse_click(click_btn_x, click_btn_y, wx, wy, ww, wh, &vfs, &mut proc);
    assert!(handled_file, "Clique dentro do dialogo deve ser consumido");
    assert!(FILE_CREATED.load(Ordering::SeqCst), "Clique no botao 'Criar Arquivo' deve criar o arquivo no VFS!");
    assert_eq!(fm.dialog_kind, FileDialogKind::None, "Dialogo deve fechar apos criacao do arquivo!");
}

#[test]
fn test_file_manager_pagination_and_large_directory_listing() {
    use aladdin::apps::files::{FileManagerApp, MAX_FILES, ITEMS_PER_PAGE};
    use aladdin::contracts::vfs::VfsHost;
    use aladdin::contracts::process::ProcessHost;

    struct LargeDirVfs;
    impl VfsHost for LargeDirVfs {
        fn readdir(&self, _path: &str, cb: &mut dyn FnMut(&str, bool, bool, bool)) {
            let names = [
                "FILE01.TXT", "FILE02.TXT", "FILE03.TXT", "FILE04.TXT", "FILE05.TXT",
                "FILE06.TXT", "FILE07.TXT", "FILE08.TXT", "FILE09.TXT", "FILE10.TXT",
                "FILE11.TXT", "FILE12.TXT", "FILE13.TXT", "FILE14.TXT", "FILE15.TXT",
                "FILE16.TXT", "FILE17.TXT", "FILE18.TXT", "FILE19.TXT", "FILE20.TXT",
                "FILE21.TXT", "FILE22.TXT", "FILE23.TXT", "FILE24.TXT", "FILE25.TXT",
            ];
            for name in &names {
                cb(name, false, false, true);
            }
        }
        fn read_file(&self, _path: &str, _buf: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    struct MockProc;
    impl ProcessHost for MockProc {
        fn execute_command(&mut self, _cmd: &str, _output: &mut dyn FnMut(&str, u32)) {}
        fn is_process_alive(&self, _pid: usize) -> bool { false }
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(1) }
    }

    let mut fm = FileManagerApp::new();
    let vfs = LargeDirVfs;
    let mut proc = MockProc;

    // 1. Atualiza cache e valida superação do limite de 16 arquivos
    fm.refresh_cache(&vfs);
    assert_eq!(fm.file_cache_count, 25);
    assert!(fm.file_cache_count > 16, "O gerenciador deve suportar mais de 16 arquivos");
    assert_eq!(MAX_FILES, 64);
    assert_eq!(ITEMS_PER_PAGE, 8);

    // 2. Valida cálculo de páginas dinâmicas (25 itens com 8 por página = 4 páginas)
    assert_eq!(fm.total_pages(), 4);
    assert_eq!(fm.current_page, 0);

    // 3. Navegação entre páginas via métodos
    fm.next_page();
    assert_eq!(fm.current_page, 1);
    fm.next_page();
    assert_eq!(fm.current_page, 2);
    fm.next_page();
    assert_eq!(fm.current_page, 3);
    // Trava de saturação na última página
    fm.next_page();
    assert_eq!(fm.current_page, 3);

    // Retorno de páginas
    fm.prev_page();
    assert_eq!(fm.current_page, 2);
    fm.prev_page();
    assert_eq!(fm.current_page, 1);
    fm.prev_page();
    assert_eq!(fm.current_page, 0);
    // Trava de saturação na primeira página
    fm.prev_page();
    assert_eq!(fm.current_page, 0);

    // 4. Teste de cliques na barra de paginação do rodapé
    let wx: usize = 50;
    let wy: usize = 50;
    let ww: usize = 540;
    let wh: usize = 320;
    let list_x = wx + 130;
    let list_w = ww.saturating_sub(140);
    let main_y = wy + 78;
    let main_h = wh.saturating_sub(116);
    let pag_y = main_y + main_h.saturating_sub(22);

    // Clique no botão [Prox >] na barra de paginação
    let next_btn_x = list_x + list_w - 30;
    let clicked_next = fm.handle_mouse_click(next_btn_x, pag_y + 10, wx, wy, ww, wh, &vfs, &mut proc);
    assert!(clicked_next);
    assert_eq!(fm.current_page, 1);

    // Clique no primeiro item da página 1 (que deve corresponder ao índice 8: FILE09.TXT)
    let row0_y = main_y + 10;
    let clicked_item = fm.handle_mouse_click(list_x + 50, row0_y, wx, wy, ww, wh, &vfs, &mut proc);
    assert!(clicked_item);
    assert_eq!(fm.selected_index, Some(8));
    assert_eq!(fm.file_cache[8].name_str(), "FILE09.TXT");

    // Clique no botão [< Ant] na barra de paginação
    let prev_btn_x = list_x + 20;
    let clicked_prev = fm.handle_mouse_click(prev_btn_x, pag_y + 10, wx, wy, ww, wh, &vfs, &mut proc);
    assert!(clicked_prev);
    assert_eq!(fm.current_page, 0);
}

#[test]
fn test_file_manager_move_operation() {
    use aladdin::apps::files::FileManagerApp;
    use aladdin::contracts::vfs::VfsHost;
    use std::sync::atomic::{AtomicBool, Ordering};

    static MOVED_CORRECTLY: AtomicBool = AtomicBool::new(false);

    struct MoveVfs;
    impl VfsHost for MoveVfs {
        fn readdir(&self, _path: &str, cb: &mut dyn FnMut(&str, bool, bool, bool)) {
            cb("DOC.TXT", false, false, true);
        }
        fn read_file(&self, _path: &str, _buf: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
        fn rename_file(&self, old_path: &str, new_path: &str) -> Result<(), ()> {
            if old_path == "/disk/DOC.TXT" && new_path == "/disk/BACKUP/DOC.TXT" {
                MOVED_CORRECTLY.store(true, Ordering::SeqCst);
                Ok(())
            } else {
                Err(())
            }
        }
    }

    let mut fm = FileManagerApp::new();
    let vfs = MoveVfs;
    fm.refresh_cache(&vfs);

    fm.selected_index = Some(0);
    assert!(fm.move_selected("/disk/BACKUP", &vfs).is_ok());
    assert!(MOVED_CORRECTLY.load(Ordering::SeqCst));
}

#[test]
fn test_system_monitor_network_states_and_adapter_telemetry() {
    use aladdin::contracts::system::{SystemHost, SystemStats, JinnWallProfile};
    use aladdin::apps::monitor::SystemMonitorApp;

    struct NetStateHost {
        nic_name: &'static str,
        state_desc: &'static str,
        ip: [u8; 4],
    }

    impl SystemHost for NetStateHost {
        fn stats(&self) -> SystemStats {
            SystemStats {
                uptime_secs: 100,
                task_count: 4,
                sched_ticks: 1000,
                used_ram_mb: 32,
                total_ram_mb: 512,
                current_tick: 1000,
                net_ip: self.ip,
                net_adapter: self.nic_name,
                net_state_desc: self.state_desc,
                jinnwall_active_conns: 1,
                jinnwall_drops: 0,
                jinnwall_inspected: 10,
                jinnwall_accepted: 10,
                jinnwall_profile: JinnWallProfile::PublicSecure,
                physical_frames_used: 8192,
                physical_frames_total: 131072,
                dynamic_pages_allocated: 0,
                dynamic_pages_freed: 0,
                vfs_open_fds: 1,
                vfs_mounted_volumes: 1,
            }
        }
        fn yield_cpu(&self) {}
        fn storage_profile_name(&self) -> &'static str { "FAT32" }
        fn cycle_storage_profile(&self) {}
        fn jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn set_jinnwall_profile(&self, _profile: JinnWallProfile) {}
        fn toggle_jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn reboot(&self) -> ! { loop {} }
    }

    let mut buf = vec![0u32; 800 * 600];

    // Caso A: Operacional com Intel 82540EM (VirtualBox / Bare Metal)
    let host_e1000 = NetStateHost {
        nic_name: "Intel 82540EM",
        state_desc: "Rede: Ativa e operacional",
        ip: [192, 168, 1, 50],
    };
    SystemMonitorApp.render(&mut buf, 800, 600, 10, 10, 300, 300, &host_e1000.stats());

    // Caso B: Interface desconectada / sem PCI
    let host_none = NetStateHost {
        nic_name: "Nenhum",
        state_desc: "PCI: Sem placa de rede",
        ip: [0, 0, 0, 0],
    };
    SystemMonitorApp.render(&mut buf, 800, 600, 10, 10, 300, 300, &host_none.stats());
}

#[test]
fn test_window_snapping_left_and_right() {
    use aladdin::wm::Window;

    let mut win = Window::new(0, "Snap Target", 150, 120, 400, 300);
    assert!(!win.is_snapped_left);
    assert!(!win.is_snapped_right);
    assert!(!win.is_maximized);

    // 1. Encaixe à Esquerda (Snap Left)
    let dmg_left = win.snap_left(1024, 768, 26, 54);
    assert!(win.is_snapped_left);
    assert!(!win.is_snapped_right);
    assert!(!win.is_maximized);
    assert_eq!(win.x, 0);
    assert_eq!(win.y, 27);
    assert_eq!(win.width, 512);
    assert_eq!(win.height, 678);
    assert!(dmg_left.width >= 512);

    // 2. Encaixe à Direita (Snap Right)
    let dmg_right = win.snap_right(1024, 768, 26, 54);
    assert!(!win.is_snapped_left);
    assert!(win.is_snapped_right);
    assert_eq!(win.x, 512);
    assert_eq!(win.y, 27);
    assert_eq!(win.width, 512);
    assert_eq!(win.height, 678);
    assert!(dmg_right.width >= 512);

    // 3. Restauração (Restore Snap)
    win.restore_snap();
    assert!(!win.is_snapped_left);
    assert!(!win.is_snapped_right);
    assert_eq!(win.x, 150);
    assert_eq!(win.y, 120);
    assert_eq!(win.width, 400);
    assert_eq!(win.height, 300);
}

#[test]
fn test_toast_notifications_lifecycle_and_progress() {
    use aladdin::desktop::notifications::{NotificationCenter, TOAST_WIDTH};
    use aladdin::compositor::{COLOR_CYAN_NEON, COLOR_EMERALD, COLOR_RUBY};

    let mut center = NotificationCenter::new();
    assert_eq!(center.active_count(0), 0);

    // 1. Posta 3 notificações
    let id1 = center.post(b'*', "Notificacao 1", "Primeiro alerta", COLOR_CYAN_NEON, 100, 10);
    let id2 = center.post(b'!', "Notificacao 2", "Segundo alerta", COLOR_EMERALD, 200, 10);
    let id3 = center.post(b'X', "Notificacao 3", "Terceiro alerta", COLOR_RUBY, 300, 10);

    assert_eq!(center.active_count(10), 3);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);

    // 2. Expiração pelo tempo (no tick 120: id1 expirou pois 120 - 10 = 110 >= 100)
    assert_eq!(center.active_count(120), 2);

    // 3. Fechamento manual (dismiss)
    center.dismiss(id2);
    assert_eq!(center.active_count(120), 1);

    // 4. Teste de clique (handle_click)
    let screen_w = 1024;
    let toast_x = screen_w - (TOAST_WIDTH + 12);
    let click_toast = center.handle_click(toast_x + 20, 36, screen_w, 120);
    assert!(click_toast, "Clique dentro da notificacao ativa deve descartá-la");
    assert_eq!(center.active_count(120), 0);

    // 5. Renderização em buffer
    let mut buf = vec![0u32; 1024 * 768];
    center.post(b'*', "Teste Render", "Mensagem de teste", COLOR_CYAN_NEON, 200, 0);
    center.render(&mut buf, 1024, 768, 50, 0, 0);
    let non_zero = buf.iter().filter(|&&p| p != 0).count();
    assert!(non_zero > 1000, "Render de notificacao deve preencher buffer grafico");
}

#[test]
fn test_quick_settings_popover_interactions() {
    use aladdin::desktop::quick_settings::{QuickSettingsPopover, QuickSettingsAction};
    use aladdin::contracts::system::{CapabilityMode, JinnWallProfile, SystemHost, SystemStats};

    struct MockSys;
    impl SystemHost for MockSys {
        fn stats(&self) -> SystemStats {
            SystemStats {
                uptime_secs: 42,
                task_count: 5,
                sched_ticks: 4200,
                used_ram_mb: 24,
                total_ram_mb: 512,
                current_tick: 4200,
                net_ip: [192, 168, 1, 10],
                net_adapter: "VirtIO-Net",
                net_state_desc: "Ativo",
                jinnwall_active_conns: 3,
                jinnwall_drops: 0,
                jinnwall_inspected: 100,
                jinnwall_accepted: 100,
                jinnwall_profile: JinnWallProfile::PublicSecure,
                physical_frames_used: 6144,
                physical_frames_total: 131072,
                dynamic_pages_allocated: 0,
                dynamic_pages_freed: 0,
                vfs_open_fds: 2,
                vfs_mounted_volumes: 1,
            }
        }
        fn yield_cpu(&self) {}
        fn storage_profile_name(&self) -> &'static str { "FAT32" }
        fn cycle_storage_profile(&self) {}
        fn jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn set_jinnwall_profile(&self, _profile: JinnWallProfile) {}
        fn toggle_jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn reboot(&self) -> ! { loop {} }
    }

    let sys = MockSys;
    let pop_x = 700;
    let pop_y = 28;

    // 1. Cliques nos modos visuais FX
    // Eco
    let act_eco = QuickSettingsPopover::handle_click(pop_x, pop_y, pop_x + 20, pop_y + 60);
    assert_eq!(act_eco, QuickSettingsAction::SetMode(CapabilityMode::Basic));

    // Equilibrado
    let act_bal = QuickSettingsPopover::handle_click(pop_x, pop_y, pop_x + 110, pop_y + 60);
    assert_eq!(act_bal, QuickSettingsAction::SetMode(CapabilityMode::Balanced));

    // Glow
    let act_glow = QuickSettingsPopover::handle_click(pop_x, pop_y, pop_x + 210, pop_y + 60);
    assert_eq!(act_glow, QuickSettingsAction::SetMode(CapabilityMode::Advanced));

    // 2. Alternar Firewall
    let act_fw = QuickSettingsPopover::handle_click(pop_x, pop_y, pop_x + 40, pop_y + 175);
    assert_eq!(act_fw, QuickSettingsAction::ToggleFirewall);

    // 3. Ciclar Storage JSA
    let act_jsa = QuickSettingsPopover::handle_click(pop_x, pop_y, pop_x + 180, pop_y + 175);
    assert_eq!(act_jsa, QuickSettingsAction::CycleStorage);

    // 4. Clique fora do popover retorna None
    let act_out = QuickSettingsPopover::handle_click(pop_x, pop_y, 10, 10);
    assert_eq!(act_out, QuickSettingsAction::None);

    // 5. Renderização sem pânico
    let mut buf = vec![0u32; 1024 * 768];
    QuickSettingsPopover::render(&mut buf, 1024, 768, pop_x, pop_y, 750, 50, CapabilityMode::Balanced, &sys);
    let filled = buf.iter().filter(|&&p| p != 0).count();
    assert!(filled > 2000, "Popover deve renderizar card e controles");
}

#[test]
fn test_app_switcher_hud_lifecycle_and_tab_navigation() {
    use aladdin::desktop::session::AladdinSession;
    use aladdin::contracts::system::{SystemHost, SystemStats, JinnWallProfile};
    use aladdin::contracts::vfs::VfsHost;
    use aladdin::contracts::process::ProcessHost;

    struct DummySys;
    impl SystemHost for DummySys {
        fn stats(&self) -> SystemStats {
            SystemStats {
                uptime_secs: 10,
                task_count: 2,
                sched_ticks: 100,
                used_ram_mb: 16,
                total_ram_mb: 256,
                current_tick: 100,
                net_ip: [0, 0, 0, 0],
                net_adapter: "None",
                net_state_desc: "Down",
                jinnwall_active_conns: 0,
                jinnwall_drops: 0,
                jinnwall_inspected: 0,
                jinnwall_accepted: 0,
                jinnwall_profile: JinnWallProfile::PublicSecure,
                physical_frames_used: 0,
                physical_frames_total: 131072,
                dynamic_pages_allocated: 0,
                dynamic_pages_freed: 0,
                vfs_open_fds: 0,
                vfs_mounted_volumes: 0,
            }
        }
        fn yield_cpu(&self) {}
        fn storage_profile_name(&self) -> &'static str { "FAT32" }
        fn cycle_storage_profile(&self) {}
        fn jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn set_jinnwall_profile(&self, _profile: JinnWallProfile) {}
        fn toggle_jinnwall_profile(&self) -> JinnWallProfile { JinnWallProfile::PublicSecure }
        fn reboot(&self) -> ! { loop {} }
    }

    struct DummyVfs;
    impl VfsHost for DummyVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _buf: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    struct DummyProc;
    impl ProcessHost for DummyProc {
        fn execute_command(&mut self, _cmd: &str, _output: &mut dyn FnMut(&str, u32)) {}
        fn is_process_alive(&self, _pid: usize) -> bool { false }
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(1) }
    }

    let mut session = AladdinSession::new();
    let sys = DummySys;
    let vfs = DummyVfs;
    let mut proc = DummyProc;

    // Inicialmente fechado
    assert!(!session.app_switcher_open);

    // 1. Pressiona Tab (0x09): abre o switcher
    session.handle_key(0x09, &mut proc, &sys, &vfs);
    assert!(session.app_switcher_open);
    assert_eq!(session.app_switcher_index, 1);

    // 2. Pressiona Tab novamente: incrementa o índice para próxima janela
    session.handle_key(0x09, &mut proc, &sys, &vfs);
    assert!(session.app_switcher_open);
    assert_eq!(session.app_switcher_index, 2);

    // 3. Pressiona Enter ('\n'): confirma seleção e fecha o switcher
    session.handle_key(b'\n', &mut proc, &sys, &vfs);
    assert!(!session.app_switcher_open);

    // 4. Reabre com Tab e fecha com Escape (0x1B)
    session.handle_key(0x09, &mut proc, &sys, &vfs);
    assert!(session.app_switcher_open);
    session.handle_key(0x1B, &mut proc, &sys, &vfs);
    assert!(!session.app_switcher_open);
}

#[test]
fn test_dock_tooltips_and_window_state_indicators() {
    use aladdin::wm::{Window, WindowManager};
    use aladdin::desktop::dock::render_dock;
    use aladdin::compositor::{COLOR_EMERALD, COLOR_AMBER, COLOR_CYAN_NEON};

    let w = 1024;
    let h = 768;
    let mut buf = vec![0u32; w * h];
    let mut wm = WindowManager::new();

    // Janela 0 aberta em background (deve mostrar ponto esmeralda)
    let mut win0 = Window::new(0, "Monitor", 50, 50, 300, 200);
    win0.is_open = true;
    win0.is_minimized = false;
    wm.windows[0] = Some(win0);

    // Janela 1 aberta e com foco (deve mostrar barra ciano)
    let mut win1 = Window::new(1, "Terminal", 200, 200, 400, 300);
    win1.is_open = true;
    win1.is_minimized = false;
    wm.windows[1] = Some(win1);

    // Janela 3 minimizada (deve mostrar ponto âmbar)
    let mut win3 = Window::new(3, "Arquivos", 100, 100, 300, 200);
    win3.is_open = true;
    win3.is_minimized = true;
    wm.windows[3] = Some(win3);

    wm.order = [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 1]; // Win 1 no topo

    // Renderiza Dock com o mouse sobre o primeiro slot (gera tooltip)
    let dock_x = (w / 2) - (aladdin::desktop::dock::DOCK_WIDTH / 2);
    render_dock(&mut buf, w, h, dock_x + 30, h - 30, &wm, None);

    let has_cyan = buf.iter().any(|&p| p == COLOR_CYAN_NEON);
    let has_emerald = buf.iter().any(|&p| p == COLOR_EMERALD);
    let has_amber = buf.iter().any(|&p| p == COLOR_AMBER);

    assert!(has_cyan, "Janela com foco ativo deve exibir indicador ciano no Dock");
    assert!(has_emerald, "Janela em segundo plano deve exibir indicador esmeralda no Dock");
    assert!(has_amber, "Janela minimizada deve exibir indicador âmbar no Dock");
}

#[test]
fn test_client_window_maximize_and_resize_event_dispatch() {
    use aladdin::contracts::input::EVENT_WINDOW_RESIZE;
    use aladdin::contracts::process::ProcessHost;
    use aladdin::contracts::vfs::VfsHost;

    struct TestProc;
    impl ProcessHost for TestProc {
        fn spawn_app(&mut self, _path: &str) -> Result<usize, ()> { Ok(42) }
        fn execute_command(&mut self, _cmd: &str, _cb: &mut dyn FnMut(&str, u32)) {}
        fn is_process_alive(&self, _pid: usize) -> bool { true }
    }

    struct DummyVfs;
    impl VfsHost for DummyVfs {
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, _dest: &mut [u8]) -> Result<usize, ()> { Ok(0) }
        fn write_file(&self, _path: &str, _data: &[u8]) -> Result<(), ()> { Ok(()) }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    let mut session = aladdin::desktop::session::AladdinSession::new();
    let sys = MockSystem::new();
    let vfs = DummyVfs;
    let mut proc = TestProc;

    let win_id = session.create_client_window(42, "Browser Test", 300, 180)
        .expect("Janela cliente criada");
    assert!(win_id < aladdin::wm::manager::MAX_WINDOWS);

    // Inicialmente buffer é 300 * 180 = 54000
    assert_eq!(session.wm.client_buffers[win_id].as_ref().unwrap().len(), 300 * 180);

    // Posição do botão de maximizar da janela cliente (x + width - 44 + 8)
    let win = session.wm.windows[win_id].as_ref().unwrap();
    let max_btn_x = win.x + win.width - 36;
    let max_btn_y = win.y + 10;

    // Simula clique no botão maximizar
    session.handle_mouse(max_btn_x, max_btn_y, 1, &sys, &vfs, &mut proc, 1024, 768);

    // Verifica que a janela foi maximizada
    let win_max = session.wm.windows[win_id].as_ref().unwrap();
    assert!(win_max.is_maximized);
    let expected_client_w = win_max.client_width();
    let expected_client_h = win_max.client_height();
    let expected_pixels = expected_client_w * expected_client_h;

    // Buffer interno do compositor foi redimensionado atomicamente
    assert_eq!(session.wm.client_buffers[win_id].as_ref().unwrap().len(), expected_pixels);

    // Evento EVENT_WINDOW_RESIZE foi enfileirado para o processo cliente
    let ev = session.poll_client_event(42, win_id).expect("Poll ok").expect("Evento deve existir");
    assert_eq!(ev.event_type, EVENT_WINDOW_RESIZE);
    assert_eq!(ev.x, expected_client_w as i32);
    assert_eq!(ev.y, expected_client_h as i32);

    // Teste de envio de buffer com as novas dimensões
    let new_buf = alloc::vec![0x00FF_0000; expected_pixels];
    let update_res = session.update_client_buffer(42, win_id, &new_buf);
    assert!(update_res.is_ok(), "Atualização com novas dimensões de tela cheia deve ter sucesso");
}

#[test]
fn test_box_blur_and_acrylic_glassmorphism() {
    use aladdin::compositor::*;

    let w = 120;
    let h = 80;
    let mut buf = vec![0u32; w * h];

    // Cria padrão de alto contraste: metade esquerda preta (0), metade direita branca (0x00FF_FFFF)
    for y in 0..h {
        for x in 0..w {
            buf[y * w + x] = if x < w / 2 { 0x0000_0000 } else { 0x00FF_FFFF };
        }
    }

    // 1. Aplica box-blur na região central
    box_blur_rect(&mut buf, w, h, 20, 20, 80, 40, 4);

    // O píxel na fronteira do blur (x = 60, y = 40) não deve ser nem puro 0 nem puro 0xFF_FFFF
    let mid_pixel = buf[40 * w + 60];
    let r = (mid_pixel >> 16) & 0xFF;
    let g = (mid_pixel >> 8) & 0xFF;
    let b = mid_pixel & 0xFF;
    assert!(r > 30 && r < 225, "Blur deve suavizar a borda de contraste: r={}", r);
    assert!(g > 30 && g < 225, "Blur deve suavizar a borda de contraste: g={}", g);
    assert!(b > 30 && b < 225, "Blur deve suavizar a borda de contraste: b={}", b);

    // 2. Aplica fill_acrylic_glass com tonalidade ciano e raio de cantos
    fill_acrylic_glass(&mut buf, w, h, 10, 10, 100, 60, 8, COLOR_CYAN, 128);
    let acrylic_pixel = buf[30 * w + 30];
    assert_ne!(acrylic_pixel, 0, "Superfície acrílica deve possuir pigmentação mesclada");
}

#[test]
fn test_crisp_antialiased_typography() {
    use aladdin::font::*;

    let w = 80;
    let h = 40;
    let mut buf = vec![0x0010_1520; w * h]; // Fundo ardósia escuro

    // Renderiza caractere 'A' com antialiasing
    draw_char_antialiased(&mut buf, w, h, b'A', 10, 10, 0x00FF_FFFF, None, None);

    // Deve haver pixels intermediários (antialiased) entre o fundo (0x0010_1520) e o branco puro (0x00FF_FFFF)
    let mut found_halo = false;
    for y in 10..18 {
        for x in 10..18 {
            let px = buf[y * w + x];
            if px != 0x0010_1520 && px != 0x00FF_FFFF {
                found_halo = true;
                break;
            }
        }
    }
    assert!(found_halo, "Antialiasing deve gerar gradientes e halos nas arestas do glifo");

    // Renderiza string completa
    draw_string_antialiased(&mut buf, w, h, "Jinn", 30, 10, 0x0038_BDF8, None);
    let mut found_string_pixel = false;
    for y in 10..18 {
        for x in 30..62 {
            if buf[y * w + x] != 0x0010_1520 {
                found_string_pixel = true;
                break;
            }
        }
    }
    assert!(found_string_pixel, "Texto 'Jinn' deve estar presente no buffer");
}

#[test]
fn test_advanced_procedural_wallpapers() {
    use aladdin::compositor::*;
    use aladdin::contracts::system::CapabilityMode;

    let w = 400;
    let h = 300;
    let mut buf = vec![0u32; w * h];

    // Testa todos os 12 temas de wallpapers (0..=4 procedurais, 5..=7 domínio público, 8..=11 vídeos motion)
    for wp_id in 0..=11 {
        render_wallpaper(&mut buf, w, h, CapabilityMode::Advanced, wp_id, 42);
        // Garante que o buffer foi inteiramente preenchido (sem pixels nulos na base e topo)
        assert_ne!(buf[0], 0, "Wallpaper #{} topo não deve ser nulo", wp_id);
        assert_ne!(buf[(h - 1) * w], 0, "Wallpaper #{} base não deve ser nula", wp_id);
    }
}

#[test]
fn test_fluid_window_motion_interpolation() {
    use aladdin::wm::{Window, WindowManager};

    let mut wm = WindowManager::new();
    let win = Window::new(0, "Animated Window", 100, 100, 300, 200);
    let win_id = wm.open_window(win).expect("Janela deve abrir");

    // Dispara snap_left (deve iniciar animação suave em direção a x=0)
    let _dirty = wm.windows[win_id].as_mut().unwrap().snap_left(1024, 768, 26, 44);

    let win_ref = wm.windows[win_id].as_ref().unwrap();
    assert_eq!(win_ref.x, 0, "Destino lógico final deve ser 0");
    assert!(win_ref.anim_ticks_remaining > 0, "Transição de animação deve estar ativa");
    assert_eq!(win_ref.render_x(), 100, "Primeiro frame de renderização deve começar na posição anterior (100)");

    // Executa ticks de animação e verifica aproximação suave
    let mut last_render_x = win_ref.render_x();
    while wm.windows[win_id].as_ref().unwrap().anim_ticks_remaining > 0 {
        let animated = wm.tick_animations();
        assert!(animated, "tick_animations deve reportar atividade de movimento");
        let cur_render_x = wm.windows[win_id].as_ref().unwrap().render_x();
        assert!(cur_render_x <= last_render_x, "render_x deve se aproximar monotonicamente de 0");
        last_render_x = cur_render_x;
    }

    // Ao fim da transição, a posição renderizada deve coincidir exatamente com a posição lógica
    let win_final = wm.windows[win_id].as_ref().unwrap();
    assert_eq!(win_final.render_x(), 0);
    assert_eq!(win_final.anim_ticks_remaining, 0);
}

#[test]
fn test_wallpaper_dynamic_and_audio_settings() {
    use aladdin::apps::settings::SettingsApp;
    use aladdin::contracts::system::CapabilityMode;
    use aladdin::contracts::vfs::VfsHost;

    use std::collections::BTreeMap;

    struct MockVfs {
        files: std::cell::RefCell<BTreeMap<String, Vec<u8>>>,
    }
    impl VfsHost for MockVfs {
        fn create_file(&self, path: &str) -> Result<(), ()> {
            self.files.borrow_mut().entry(path.to_string()).or_insert_with(Vec::new);
            Ok(())
        }
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, path: &str, buf: &mut [u8]) -> Result<usize, ()> {
            if let Some(data) = self.files.borrow().get(path) {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                Err(())
            }
        }
        fn write_file(&self, path: &str, buf: &[u8]) -> Result<(), ()> {
            self.files.borrow_mut().insert(path.to_string(), buf.to_vec());
            Ok(())
        }
        fn remove_file(&self, path: &str) -> Result<(), ()> {
            self.files.borrow_mut().remove(path);
            Ok(())
        }
        fn rename_file(&self, old_path: &str, new_path: &str) -> Result<(), ()> {
            let data = self.files.borrow_mut().remove(old_path);
            if let Some(d) = data {
                self.files.borrow_mut().insert(new_path.to_string(), d);
                Ok(())
            } else {
                Err(())
            }
        }
    }

    let mock_vfs = MockVfs { files: std::cell::RefCell::new(BTreeMap::new()) };
    let mut app = SettingsApp::new();

    // Valores padrão
    assert_eq!(app.wallpaper_id, 0);
    assert_eq!(app.wallpaper_audio_enabled, false);
    assert_eq!(app.wallpaper_audio_volume, 40);

    // Altera para Wallpaper de Vídeo (Matrix Rain - ID 8) com Áudio ativado a 70%
    app.wallpaper_id = 8;
    app.wallpaper_audio_enabled = true;
    app.wallpaper_audio_volume = 70;

    // Salva no VFS
    assert!(app.save_to_vfs(&mock_vfs, CapabilityMode::Advanced).is_ok());

    // Carrega em nova instância
    let mut app2 = SettingsApp::new();
    let mode2 = app2.load_from_vfs(&mock_vfs);

    assert_eq!(mode2, Some(CapabilityMode::Advanced));
    assert_eq!(app2.wallpaper_id, 8);
    assert_eq!(app2.wallpaper_audio_enabled, true);
    assert_eq!(app2.wallpaper_audio_volume, 70);
}

#[test]
fn test_settings_corrupted_file_recovery_and_fuzzing() {
    use aladdin::apps::settings::SettingsApp;
    use aladdin::contracts::system::CapabilityMode;
    use aladdin::contracts::vfs::VfsHost;

    struct FuzzVfs {
        payload: std::cell::RefCell<Vec<u8>>,
    }
    impl VfsHost for FuzzVfs {
        fn create_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, _path: &str, buf: &mut [u8]) -> Result<usize, ()> {
            let p = self.payload.borrow();
            let len = p.len().min(buf.len());
            buf[..len].copy_from_slice(&p[..len]);
            Ok(len)
        }
        fn write_file(&self, _path: &str, buf: &[u8]) -> Result<(), ()> {
            *self.payload.borrow_mut() = buf.to_vec();
            Ok(())
        }
        fn remove_file(&self, _path: &str) -> Result<(), ()> { Ok(()) }
    }

    let fuzz_vfs = FuzzVfs { payload: std::cell::RefCell::new(Vec::new()) };

    // Cenário 1: Arquivo completamente vazio (0 bytes)
    *fuzz_vfs.payload.borrow_mut() = vec![];
    let mut app = SettingsApp::new();
    assert_eq!(app.load_from_vfs(&fuzz_vfs), None);
    assert_eq!(app.wallpaper_id, 0); // Padrão preservado
    assert_eq!(app.wallpaper_audio_volume, 40);

    // Cenário 2: Bytes binários arbitrários (não-UTF8)
    *fuzz_vfs.payload.borrow_mut() = vec![0xFF, 0xFE, 0xAA, 0x00, 0x12, 0x88, 0x99];
    let mut app = SettingsApp::new();
    assert_eq!(app.load_from_vfs(&fuzz_vfs), None);
    assert_eq!(app.wallpaper_id, 0);

    // Cenário 3: Chaves truncadas e valores fora dos limites válidos
    let corrupted_text = b"# ALADDIN_CONFIG_V1\nMODE=99\nWALLPAPER=255\nWALLPAPER_AUDIO=abc\nWALLPAPER_VOL=999\nGARBAGE_LINE\n";
    *fuzz_vfs.payload.borrow_mut() = corrupted_text.to_vec();
    let mut app = SettingsApp::new();
    let mode = app.load_from_vfs(&fuzz_vfs);
    assert_eq!(mode, None); // Mode 99 é inválido, então retorna None
    assert_eq!(app.wallpaper_id, 0); // Wallpaper 255 é rejeitado (>11), mantendo 0
    assert_eq!(app.wallpaper_audio_enabled, false); // "abc" falha no parse, mantendo false
    assert_eq!(app.wallpaper_audio_volume, 40); // 999 excede u8 e falha no parse, mantendo o padrão seguro 40

    // Cenário 3b: Valor válido de u8 mas acima de 100 (ex: 150) é saturado com segurança para 100
    let clamp_text = b"WALLPAPER_VOL=150\n";
    *fuzz_vfs.payload.borrow_mut() = clamp_text.to_vec();
    let mut app_clamp = SettingsApp::new();
    app_clamp.load_from_vfs(&fuzz_vfs);
    assert_eq!(app_clamp.wallpaper_audio_volume, 100);

    // Cenário 4: Arquivo truncado no meio de uma linha
    let partial_text = b"# ALADDIN_CONFIG_V1\nMODE=1\nWALLP";
    *fuzz_vfs.payload.borrow_mut() = partial_text.to_vec();
    let mut app = SettingsApp::new();
    let mode = app.load_from_vfs(&fuzz_vfs);
    assert_eq!(mode, Some(CapabilityMode::Balanced));
    assert_eq!(app.wallpaper_id, 0);
}

#[test]
fn test_settings_transactional_save_and_rollback() {
    use std::collections::BTreeMap;
    use aladdin::apps::settings::SettingsApp;
    use aladdin::contracts::system::CapabilityMode;
    use aladdin::contracts::vfs::VfsHost;

    struct FailVfs {
        files: std::cell::RefCell<BTreeMap<String, Vec<u8>>>,
        corrupt_tmp_on_write: bool,
    }
    impl VfsHost for FailVfs {
        fn create_file(&self, path: &str) -> Result<(), ()> {
            self.files.borrow_mut().entry(path.to_string()).or_insert_with(Vec::new);
            Ok(())
        }
        fn readdir(&self, _path: &str, _cb: &mut dyn FnMut(&str, bool, bool, bool)) {}
        fn read_file(&self, path: &str, buf: &mut [u8]) -> Result<usize, ()> {
            if let Some(data) = self.files.borrow().get(path) {
                let len = data.len().min(buf.len());
                buf[..len].copy_from_slice(&data[..len]);
                Ok(len)
            } else {
                Err(())
            }
        }
        fn write_file(&self, path: &str, buf: &[u8]) -> Result<(), ()> {
            let mut payload = buf.to_vec();
            if self.corrupt_tmp_on_write && path == "/disk/SETTINGS.TMP" {
                payload[0] ^= 0xFF; // Corrompe byte intencionalmente para simular erro de gravação/disco
            }
            self.files.borrow_mut().insert(path.to_string(), payload);
            Ok(())
        }
        fn remove_file(&self, path: &str) -> Result<(), ()> {
            self.files.borrow_mut().remove(path);
            Ok(())
        }
        fn rename_file(&self, old_path: &str, new_path: &str) -> Result<(), ()> {
            let data = self.files.borrow_mut().remove(old_path);
            if let Some(d) = data {
                self.files.borrow_mut().insert(new_path.to_string(), d);
                Ok(())
            } else {
                Err(())
            }
        }
    }

    // 1. Gravação bem-sucedida: .TMP validado e promovido para .CFG
    let vfs_ok = FailVfs {
        files: std::cell::RefCell::new(BTreeMap::new()),
        corrupt_tmp_on_write: false,
    };
    let mut app = SettingsApp::new();
    app.wallpaper_id = 9;
    assert!(app.save_to_vfs(&vfs_ok, CapabilityMode::Advanced).is_ok());
    assert!(vfs_ok.files.borrow().contains_key("/disk/SETTINGS.CFG"));
    assert!(!vfs_ok.files.borrow().contains_key("/disk/SETTINGS.TMP")); // .TMP deve ter sido limpo

    // 2. Gravação com falha de integridade em .TMP: aborta e não corrompe .CFG existente
    let initial_cfg = b"# ALADDIN_CONFIG_V1\nMODE=1\nWALLPAPER=2\n";
    let mut files_map = BTreeMap::new();
    files_map.insert("/disk/SETTINGS.CFG".to_string(), initial_cfg.to_vec());

    let vfs_fail = FailVfs {
        files: std::cell::RefCell::new(files_map),
        corrupt_tmp_on_write: true,
    };
    let mut app_fail = SettingsApp::new();
    app_fail.wallpaper_id = 5;
    // save_to_vfs detecta divergência de checksum/bytes e aborta
    assert!(app_fail.save_to_vfs(&vfs_fail, CapabilityMode::Basic).is_err());
    // O arquivo .CFG anterior continua intacto!
    assert_eq!(vfs_fail.files.borrow().get("/disk/SETTINGS.CFG").unwrap(), initial_cfg);

    // 3. Queda de energia simulada durante rename: .CFG ausente/corrompido mas .BAK íntegro
    let mut files_powercut_bak = BTreeMap::new();
    files_powercut_bak.insert("/disk/SETTINGS.CFG".to_string(), b"GARBAGE_ZERO_BYTES".to_vec());
    files_powercut_bak.insert("/disk/SETTINGS.BAK".to_string(), initial_cfg.to_vec());

    let vfs_powercut_bak = FailVfs {
        files: std::cell::RefCell::new(files_powercut_bak),
        corrupt_tmp_on_write: false,
    };
    let mut app_recovered_bak = SettingsApp::new();
    let mode_bak = app_recovered_bak.load_from_vfs(&vfs_powercut_bak);
    assert_eq!(mode_bak, Some(CapabilityMode::Balanced));
    assert_eq!(app_recovered_bak.wallpaper_id, 2);
    // .CFG foi restaurado com sucesso a partir do .BAK
    assert_eq!(vfs_powercut_bak.files.borrow().get("/disk/SETTINGS.CFG").unwrap(), initial_cfg);

    // 4. Queda de energia simulada antes da promoção: .CFG ausente mas .TMP íntegro sobreviveu
    let mut files_powercut_tmp = BTreeMap::new();
    files_powercut_tmp.insert("/disk/SETTINGS.TMP".to_string(), b"# ALADDIN_CONFIG_V1\nMODE=2\nWALLPAPER=8\n".to_vec());

    let vfs_powercut_tmp = FailVfs {
        files: std::cell::RefCell::new(files_powercut_tmp),
        corrupt_tmp_on_write: false,
    };
    let mut app_recovered_tmp = SettingsApp::new();
    let mode_tmp = app_recovered_tmp.load_from_vfs(&vfs_powercut_tmp);
    assert_eq!(mode_tmp, Some(CapabilityMode::Advanced));
    assert_eq!(app_recovered_tmp.wallpaper_id, 8);
    // .CFG foi restaurado a partir do .TMP
    assert!(vfs_powercut_tmp.files.borrow().contains_key("/disk/SETTINGS.CFG"));
}

#[test]
fn test_wallpaper_continuity_and_occlusion_pause() {
    use aladdin::desktop::session::AladdinSession;

    let mut session = AladdinSession::new();
    assert_eq!(session.wallpaper_anim_frame, 0);
    assert_eq!(session.last_user_activity_tick, 0);

    // Seleciona wallpaper de movimento (ID 8 = Matrix Rain)
    session.settings.wallpaper_id = 8;

    // Simula 10 ticks de animação não oclusa
    for _ in 0..10 {
        session.wallpaper_anim_frame = session.wallpaper_anim_frame.wrapping_add(1);
    }
    assert_eq!(session.wallpaper_anim_frame, 10);

    // Abre janela 0 (Monitor) maximizada cobrindo o desktop
    if let Some(w) = session.wm.windows[0].as_mut() {
        w.is_open = true;
        w.is_minimized = false;
        w.is_maximized = true;
    }

    // Verifica detecção de oclusão do desktop
    let is_desktop_occluded = session.wm.windows.iter().any(|w_opt| {
        if let Some(w) = w_opt {
            w.is_open && !w.is_minimized && w.is_maximized
        } else {
            false
        }
    });
    assert!(is_desktop_occluded, "Desktop deve estar marcado como ocluso quando janela maximizada está aberta");

    // Durante oclusão, wallpaper_anim_frame NÃO é incrementado
    // (Simula passagem de 500 ticks com a janela cobrindo a tela)
    let frame_during_occlusion = session.wallpaper_anim_frame;
    assert_eq!(frame_during_occlusion, 10);

    // Des-maximiza a janela
    if let Some(w) = session.wm.windows[0].as_mut() {
        w.is_maximized = false;
    }
    let is_desktop_occluded_after = session.wm.windows.iter().any(|w_opt| {
        if let Some(w) = w_opt {
            w.is_open && !w.is_minimized && w.is_maximized
        } else {
            false
        }
    });
    assert!(!is_desktop_occluded_after, "Desktop não deve estar mais ocluso");

    // Retoma animação a partir do exato frame onde parou (frame 10 -> 11) com ZERO salto de tempo
    session.wallpaper_anim_frame = session.wallpaper_anim_frame.wrapping_add(1);
    assert_eq!(session.wallpaper_anim_frame, 11);
}
