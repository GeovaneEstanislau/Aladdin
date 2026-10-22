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
