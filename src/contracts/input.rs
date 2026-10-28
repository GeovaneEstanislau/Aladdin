//! Contrato de Eventos de Entrada de Teclado e Mouse

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left = 1,
    Right = 2,
    Middle = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Escape,
    Enter,
    Backspace,
    Tab,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Char(u8),
    RawScancode(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    MouseMove { x: usize, y: usize },
    MouseDown { button: MouseButton, x: usize, y: usize },
    MouseUp { button: MouseButton, x: usize, y: usize },
    KeyDown { key: KeyCode, ascii: Option<u8> },
    KeyUp { key: KeyCode },
}

// ── Eventos de Janela Cliente Ring 3 (Syscall 24 / Aladdin ABI) ───────────────

pub const EVENT_NONE: u32 = 0;
pub const EVENT_MOUSE_DOWN: u32 = 1;
pub const EVENT_MOUSE_UP: u32 = 2;
pub const EVENT_MOUSE_MOVE: u32 = 3;
pub const EVENT_KEY_DOWN: u32 = 4;
pub const EVENT_KEY_UP: u32 = 5;
pub const EVENT_SCROLL: u32 = 6;
pub const EVENT_WINDOW_CLOSE: u32 = 7;

/// Estrutura de evento transmitida entre o compositor e processos de usuário (32 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientWindowEvent {
    pub event_type: u32,
    pub win_id: u32,
    pub x: i32,
    pub y: i32,
    pub button_or_key: u32,
    pub modifiers: u32,
    pub timestamp: u64,
}

impl ClientWindowEvent {
    pub const fn empty() -> Self {
        Self {
            event_type: EVENT_NONE,
            win_id: 0,
            x: 0,
            y: 0,
            button_or_key: 0,
            modifiers: 0,
            timestamp: 0,
        }
    }
}

pub const MAX_CLIENT_EVENTS: usize = 16;

/// Fila circular determinística de eventos por janela cliente
#[derive(Debug, Clone, Copy)]
pub struct ClientEventQueue {
    buf: [ClientWindowEvent; MAX_CLIENT_EVENTS],
    head: usize,
    tail: usize,
    count: usize,
}

impl ClientEventQueue {
    pub const fn new() -> Self {
        Self {
            buf: [ClientWindowEvent::empty(); MAX_CLIENT_EVENTS],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, event: ClientWindowEvent) -> bool {
        if self.count >= MAX_CLIENT_EVENTS {
            // Descarta evento mais antigo se a fila estiver cheia
            self.head = (self.head + 1) % MAX_CLIENT_EVENTS;
            self.count -= 1;
        }
        self.buf[self.tail] = event;
        self.tail = (self.tail + 1) % MAX_CLIENT_EVENTS;
        self.count += 1;
        true
    }

    pub fn pop(&mut self) -> Option<ClientWindowEvent> {
        if self.count == 0 {
            return None;
        }
        let ev = self.buf[self.head];
        self.head = (self.head + 1) % MAX_CLIENT_EVENTS;
        self.count -= 1;
        Some(ev)
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.count = 0;
    }
}
