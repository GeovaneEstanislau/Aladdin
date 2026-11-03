//! Aladdin Design System 0.5 — Paleta de Cores e Tokens de Superfície (ARGB 32-bit)
//! Fusão arquitetural entre a pureza visual e proporções do elementary OS 7
//! e o impacto luminescente neon e gradientes profundos do Garuda Linux (Dr460nized).

// ── Superfícies e Camadas de Profundidade (Elevation Hierarchy) ─────────────
pub const COLOR_SURFACE_L0: u32     = 0x0009_0D14; // Abismo Profundo (Nível 0: Base do Wallpaper)
pub const COLOR_SURFACE_L1: u32     = 0x000F_172A; // Ardósia Espacial (Nível 1: Wingpanel e Dock flutuante)
pub const COLOR_SURFACE_L2: u32     = 0x001E_293B; // Superfície Principal (Nível 2: Janelas, cartões)
pub const COLOR_SURFACE_L3: u32     = 0x0033_4155; // Superfície de Entrada (Nível 3: Inputs, tabelas, popovers)
pub const COLOR_SURFACE_HOVER: u32  = 0x003E_4E66; // Estado Hover de elementos interativos
pub const COLOR_SURFACE_ACTIVE: u32 = 0x0027_3549; // Estado Pressionado / Ativo

// Aliases mantidos para compatibilidade total de código existente
pub const COLOR_BG: u32         = COLOR_SURFACE_L0;
pub const COLOR_CARD: u32       = COLOR_SURFACE_L2;
pub const COLOR_CARD_ALT: u32   = COLOR_SURFACE_L3;
pub const COLOR_TOPPANEL: u32   = 0x000A_0E18;
pub const COLOR_DOCK: u32       = 0x000E_1523;

// ── Bordas e Contornos (Stroke & Focus Tokens) ──────────────────────────────
pub const COLOR_BORDER_SUBTLE: u32 = 0x001E_293B; // Borda de baixo contraste (1px)
pub const COLOR_BORDER: u32        = 0x002D_3748; // Borda padrão neutra
pub const COLOR_BORDER_MUTED: u32  = 0x003E_4A5B; // Borda secundária
pub const COLOR_BORDER_ACT: u32    = 0x0038_BDF8; // Anel de foco ciano (Focus Ring)

// ── Cores de Acento Luminescente (Garuda Neon & elementary Highlights) ───────
pub const COLOR_CYAN: u32       = 0x0038_BDF8; // Ciano Luminescente (Foco principal, Wingpanel timer)
pub const COLOR_CYAN_NEON: u32  = 0x0038_BDF8;
pub const COLOR_PURPLE: u32     = 0x00A8_55F7; // Violeta Garuda Dr460nized (Acentos nobres e FX Glow)
pub const COLOR_VIOLET: u32     = 0x00A8_55F7;
pub const COLOR_EMERALD: u32    = 0x0022_C55E; // Esmeralda / Mint (Status OK, maximizar, telemetria RAM)
pub const COLOR_AMBER: u32      = 0x00F5_9E0B; // Âmbar Solar (Alerta, minimizar, processos em pausa)
pub const COLOR_RUBY: u32       = 0x00EF_4444; // Rubi Carmesim (Fechar janela, perigo, botão sair)
pub const COLOR_BLUE_PRIMARY: u32 = 0x0025_63EB; // Azul Royal (Botões primários, seleções)
pub const COLOR_BLUE_HOVER: u32 = 0x003B_82F6; // Azul Royal Hover

// ── Tipografia e Hierarquia de Texto (WCAG AA Compliance) ───────────────────
pub const COLOR_TEXT_PRI: u32   = 0x00F8_FAFC; // Texto primário de alto contraste (Slate 50)
pub const COLOR_TEXT_SEC: u32   = 0x00CBD5E1; // Texto secundário (Slate 300)
pub const COLOR_TEXT_MUTED: u32 = 0x0094_A3B8; // Texto atenuado, rótulos e atalhos (Slate 400)
pub const COLOR_TEXT_ACCENT: u32= 0x0038_BDF8; // Texto em destaque luminescente

// ── Cores Primitivas e Auxiliares ───────────────────────────────────────────
pub const COLOR_WHITE: u32      = 0x00FF_FFFF;
pub const COLOR_BLACK: u32      = 0x0000_0000;
pub const COLOR_BTN_BG: u32     = COLOR_BLUE_PRIMARY;
pub const COLOR_SHADOW: u32     = 0x0004_060A; // Sombra difusa de janela
