//! AladdinSharedRings — Lock-Free SPSC Ring Buffers (Ring 3 / Ring 0 Zero-Copy)
//!
//! Provides ultra-low latency, zero-copy shared memory communication between
//! user-space (Aladdin Desktop compositor) and kernel-space (Jinn microkernel).
//!
//! Memory layout (exactly 4096 bytes, page-aligned):
//! - Offset 0:    rx_header (64 bytes, cache-aligned) - Input Events Ring Header
//! - Offset 64:   tx_header (64 bytes, cache-aligned) - Graphics Commands Ring Header
//! - Offset 128:  rx_events (1024 bytes) - 64 SharedInputEvent slots (16 bytes each)
//! - Offset 1152: tx_cmds   (2048 bytes) - 64 SharedGfxCmd slots (32 bytes each)
//! - Offset 3200: _padding  (896 bytes)  - Padding to fill exactly 4096 bytes

use core::sync::atomic::{AtomicU32, Ordering};

pub const INPUT_RING_CAP: usize = 64;
pub const CMD_RING_CAP:   usize = 64;

// ── SharedInputEvent Kinds ──────────────────────────────────────────────────
pub const INPUT_KIND_NONE:       u16 = 0;
pub const INPUT_KIND_MOUSE_MOVE: u16 = 1;
pub const INPUT_KIND_MOUSE_BTN:  u16 = 2;
pub const INPUT_KIND_KEY_DOWN:   u16 = 3;
pub const INPUT_KIND_KEY_UP:     u16 = 4;

// ── SharedGfxCmd Opcodes ────────────────────────────────────────────────────
pub const GFX_OP_NONE:               u32 = 0;
pub const GFX_OP_PRESENT_DIRTY_RECT: u32 = 1;
pub const GFX_OP_MOVE_CURSOR:        u32 = 2;
pub const GFX_OP_SET_DISPLAY_POWER:  u32 = 3;

// ── SharedGfxCmd Flags ──────────────────────────────────────────────────────
pub const GFX_FLAG_SUB_BUF: u32 = 1 << 0;

// ── RingHeader Flags ────────────────────────────────────────────────────────
/// Indicates that the consumer (aladdin-server) is sleeping / blocked waiting for events.
/// The producer (kernel ISRs) must trigger a scheduler wakeup when this bit is set.
pub const CONSUMER_SLEEPING: u32 = 1 << 0;

// ── Doorbell Flags (Syscall 90) ─────────────────────────────────────────────
pub const DOORBELL_FLUSH_GFX:  u32 = 1 << 0;
pub const DOORBELL_WAIT_INPUT: u32 = 1 << 1;

/// Evento de entrada compartilhado (16 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedInputEvent {
    pub kind: u16,
    pub modifiers: u16,
    pub x: i32,
    pub y: i32,
    pub code: u32,
}

impl SharedInputEvent {
    pub const fn zero() -> Self {
        Self {
            kind: INPUT_KIND_NONE,
            modifiers: 0,
            x: 0,
            y: 0,
            code: 0,
        }
    }
}

/// Comando gráfico compartilhado (32 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SharedGfxCmd {
    pub opcode: u32,
    pub flags: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub src_offset: u32,
    pub reserved: u32,
}

impl SharedGfxCmd {
    pub const fn zero() -> Self {
        Self {
            opcode: GFX_OP_NONE,
            flags: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            src_offset: 0,
            reserved: 0,
        }
    }
}

/// Cabeçalho de anel SPSC alinhado a 64 bytes (1 cache line para evitar False Sharing).
#[repr(C, align(64))]
pub struct RingHeader {
    pub head: AtomicU32,
    pub tail: AtomicU32,
    pub flags: AtomicU32,
    pub _pad: [u8; 52],
}

impl RingHeader {
    pub const fn new() -> Self {
        Self {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            flags: AtomicU32::new(0),
            _pad: [0u8; 52],
        }
    }
}

/// Página compartilhada de 4096 bytes contendo anéis SPSC de input e gráficos.
#[repr(C, align(4096))]
pub struct AladdinSharedRings {
    pub rx_header: RingHeader,
    pub tx_header: RingHeader,
    pub rx_events: [SharedInputEvent; INPUT_RING_CAP],
    pub tx_cmds:   [SharedGfxCmd; CMD_RING_CAP],
    pub _padding:  [u8; 896],
}

// ── Static Layout Assertions ────────────────────────────────────────────────
const _: () = assert!(core::mem::size_of::<SharedInputEvent>() == 16);
const _: () = assert!(core::mem::size_of::<SharedGfxCmd>() == 32);
const _: () = assert!(core::mem::size_of::<RingHeader>() == 64);
const _: () = assert!(core::mem::align_of::<RingHeader>() == 64);
const _: () = assert!(core::mem::size_of::<AladdinSharedRings>() == 4096);
const _: () = assert!(core::mem::align_of::<AladdinSharedRings>() == 4096);

impl AladdinSharedRings {
    pub const fn new() -> Self {
        Self {
            rx_header: RingHeader::new(),
            tx_header: RingHeader::new(),
            rx_events: [SharedInputEvent::zero(); INPUT_RING_CAP],
            tx_cmds:   [SharedGfxCmd::zero(); CMD_RING_CAP],
            _padding:  [0u8; 896],
        }
    }

    // ── Métodos para o Anel RX (Input: Kernel produz, Ring 3 consome) ────────

    /// Consome o próximo evento de entrada do anel RX (usado pelo Ring 3).
    #[inline(always)]
    pub fn pop_rx_event(&self) -> Option<SharedInputEvent> {
        let head = self.rx_header.head.load(Ordering::Relaxed);
        let tail = self.rx_header.tail.load(Ordering::Acquire);
        if head == tail {
            return None;
        }
        let idx = (head as usize) & (INPUT_RING_CAP - 1);
        let ev = unsafe { core::ptr::read_volatile(&self.rx_events[idx]) };
        self.rx_header.head.store(head.wrapping_add(1), Ordering::Release);
        Some(ev)
    }

    /// Publica um evento de entrada no anel RX (usado pelo Kernel nas ISRs).
    #[inline(always)]
    pub fn push_rx_event(&self, ev: SharedInputEvent) -> bool {
        let tail = self.rx_header.tail.load(Ordering::Relaxed);
        let head = self.rx_header.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) >= INPUT_RING_CAP as u32 {
            return false; // Fila cheia
        }
        let idx = (tail as usize) & (INPUT_RING_CAP - 1);
        unsafe {
            let slot = self.rx_events.as_ptr().add(idx) as *mut SharedInputEvent;
            core::ptr::write_volatile(slot, ev);
        }
        self.rx_header.tail.store(tail.wrapping_add(1), Ordering::Release);
        true
    }

    #[inline(always)]
    pub fn is_rx_empty(&self) -> bool {
        self.rx_header.head.load(Ordering::Relaxed) == self.rx_header.tail.load(Ordering::Acquire)
    }

    // ── Métodos para o Anel TX (Comandos: Ring 3 produz, Kernel consome) ──────

    /// Publica um comando gráfico no anel TX (usado pelo Ring 3).
    #[inline(always)]
    pub fn push_tx_cmd(&self, cmd: SharedGfxCmd) -> bool {
        let tail = self.tx_header.tail.load(Ordering::Relaxed);
        let head = self.tx_header.head.load(Ordering::Acquire);
        if tail.wrapping_sub(head) >= CMD_RING_CAP as u32 {
            return false; // Fila cheia
        }
        let idx = (tail as usize) & (CMD_RING_CAP - 1);
        unsafe {
            let slot = self.tx_cmds.as_ptr().add(idx) as *mut SharedGfxCmd;
            core::ptr::write_volatile(slot, cmd);
        }
        self.tx_header.tail.store(tail.wrapping_add(1), Ordering::Release);
        true
    }

    /// Consome o próximo comando gráfico do anel TX (usado pelo Kernel).
    #[inline(always)]
    pub fn pop_tx_cmd(&self) -> Option<SharedGfxCmd> {
        let head = self.tx_header.head.load(Ordering::Relaxed);
        let tail = self.tx_header.tail.load(Ordering::Acquire);
        if head == tail {
            return None;
        }
        let idx = (head as usize) & (CMD_RING_CAP - 1);
        let cmd = unsafe { core::ptr::read_volatile(&self.tx_cmds[idx]) };
        self.tx_header.head.store(head.wrapping_add(1), Ordering::Release);
        Some(cmd)
    }

    #[inline(always)]
    pub fn is_tx_empty(&self) -> bool {
        self.tx_header.head.load(Ordering::Relaxed) == self.tx_header.tail.load(Ordering::Acquire)
    }
}
