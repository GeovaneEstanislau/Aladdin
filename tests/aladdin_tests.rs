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
    assert!(per_op_micros < 150.0, "Ciclo de vida de janela muito lento: {:.2} µs", per_op_micros);

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
            jinnwall_active_conns: 3,
            jinnwall_drops: 12,
            jinnwall_inspected: 150,
            jinnwall_accepted: 138,
            jinnwall_profile: prof,
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
                jinnwall_active_conns: 0,
                jinnwall_drops: 0,
                jinnwall_inspected: 0,
                jinnwall_accepted: 0,
                jinnwall_profile: JinnWallProfile::PublicSecure,
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
                jinnwall_active_conns: usize::MAX,
                jinnwall_drops: u64::MAX,
                jinnwall_inspected: u64::MAX,
                jinnwall_accepted: u64::MAX,
                jinnwall_profile: JinnWallProfile::Development,
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
                jinnwall_active_conns: 0,
                jinnwall_drops: 0,
                jinnwall_inspected: 0,
                jinnwall_accepted: 0,
                jinnwall_profile: JinnWallProfile::Development,
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
    session.handle_key(b'k', &mut proc, &sys);
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

    // 2. Outra aplicação abre janela cliente (slot 5)
    let foreign_pid = 42;
    let foreign_win_id = session.create_client_window(foreign_pid, "Editor de Texto", 300, 200)
        .expect("Janela de outra aplicacao criada");
    assert_eq!(foreign_win_id, 5);

    // A janela 5 NÃO deve ser identificada como navegador!
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

    // 4. Processo do navegador cria sua janela
    let browser_win_id = session.create_client_window(105, "Jinn Browser v0.3.0", 300, 180)
        .expect("Janela do browser criada");
    assert_eq!(browser_win_id, 7);

    // Identificação explícita do navegador deve retornar o slot 6, e NÃO o slot 5
    assert_eq!(session.find_browser_window_id(), Some(7));
    assert_eq!(session.browser_win_id, Some(7));

    // 5. Tentativa de novo lançamento não duplica o processo
    proc.spawned_path = None;
    session.launch_or_focus_browser(&mut proc);
    // NÃO deve ter chamado spawn_app novamente
    assert_eq!(proc.spawned_path, None);
    // Janela deve estar no topo
    assert_eq!(session.wm.top_window_id(), 7);

    // 6. Minimização e restauração
    let w = session.wm.windows[7].as_mut().unwrap();
    w.is_minimized = true;
    assert_eq!(session.find_browser_window_id(), Some(7));

    session.launch_or_focus_browser(&mut proc);
    let w_restored = session.wm.windows[7].as_ref().unwrap();
    assert!(!w_restored.is_minimized);
    assert!(w_restored.is_open);

    // 7. Fechamento da janela do navegador
    assert!(session.close_client_window(105, 7).is_ok());
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

    let mut session = AladdinSession::new();
    let mut proc = TestProc;
    let sys = MockSystem::new();

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
    session.handle_key(0x1B, &mut proc, &sys);
    assert_eq!(session.should_exit, false, "ESC nao deve encerrar o desktop quando janela cliente tem foco");

    let ev = session.poll_client_event(300, win_id).expect("poll ok").expect("evento presente");
    assert_eq!(ev.event_type, aladdin::contracts::input::EVENT_KEY_DOWN);
    assert_eq!(ev.button_or_key, 0x1B);

    // 4. Entrega de caracteres ASCII imprimiveis
    session.handle_key(b'x', &mut proc, &sys);
    let ev_char = session.poll_client_event(300, win_id).expect("poll ok").expect("evento presente");
    assert_eq!(ev_char.event_type, aladdin::contracts::input::EVENT_KEY_DOWN);
    assert_eq!(ev_char.button_or_key, b'x' as u32);

    // 5. Fechamento da janela e comportamento subsequente do ESC no console
    assert!(session.close_client_window(300, win_id).is_ok());
    assert_eq!(session.find_browser_window_id(), None);

    // Com foco no terminal, ESC deve sinalizar saida para o console
    session.handle_key(0x1B, &mut proc, &sys);
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
    session.handle_key(b'A', &mut proc, &sys);
    session.handle_key(b'B', &mut proc, &sys);

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
    session.handle_key(b'z', &mut proc, &sys);

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
