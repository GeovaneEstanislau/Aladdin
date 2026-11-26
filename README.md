# Aladdin — Out-of-Kernel Ring 3 Compositor & Desktop Shell

> **EN:** [README.en.md](README.en.md) | **Ecosystem:** [Jinn_Project](https://github.com/GeovaneEstanislau/Jinn_Project) · [jhcs-rs](https://github.com/GeovaneEstanislau/jhcs-rs)

**Aladdin** é um **compositor gráfico e desktop shell executado inteiramente fora do kernel (Ring 3)**, projetado para demonstrar serviços gráficos isolados sobre contratos formais de plataforma e baixa sobrecarga em hardware x86_64 sem dependência obrigatória de GPU discreta.

---

## Princípios de Arquitetura

- **Serviço Gráfico Isolado fora do Kernel (Ring 3):** O microkernel nunca executa código de janelas, fontes ou widgets em Ring 0. O servidor gráfico comunica-se com o kernel por `AladdinSharedRings` (anéis SPSC *lock-free* / *zero-copy* para eventos e superfície de vídeo) e chamadas canônicas do sistema.
- **Compositor Orientado a Dano (`DirtyRect`) e Zero-GPU:** Acumula e compõe somente regiões modificadas da tela, usando efeito de vidro fosco (*frosted glass*) acelerado por SIMD em CPU (`box_blur_rgba_simd` / `jhcs-backend-simd`) ou fallback escalar.
- **Consumo Ocioso Zero (*Zero-Idle*):** O loop de composição suspende o processo com `yield`/`wait_on` na fila de eventos quando não há entrada nem janela suja, evitando *busy-waiting* em repouso.
- **Desacoplamento por Contratos:** Interfaces explícitas (`FramebufferDevice`, `InputEvent`, `SystemHost`, `VfsHost`, `ProcessHost`) em `src/contracts/` permitem testar toda a lógica de desktop tanto em `#![no_std]` bare-metal quanto com `--features std` no host (Windows/Linux).

## Módulos Principais

- `src/contracts/`: Traits e estruturas de barramento de entrada (`EVENT_MOUSE_*`, `EVENT_KEY_DOWN`, `EVENT_SCROLL`), framebuffer, VFS, processos e diagnóstico DNS/Rotas.
- `src/compositor/`: Double buffering, rastreamento de dano (`DirtyRect`), primitivas 2D e blending alfa ARGB.
- `src/wm/`: Gerenciador de janelas, foco, Z-order, redimensionamento e encapsulamento de eventos para clientes Ring 3 (`ClientWindowEvent`).
- `src/desktop/`: Painel superior, dock, menu de aplicações, alternador de janelas e despacho de sessão (`session.rs`).
- `src/apps/`:
  - **Terminal Interativo** (`sysinfo`, `vfs`, `net`, `ps`, `mem`);
  - **Gerenciador de Arquivos** (FAT32 + Ext2 com histórico e visualização rápida);
  - **Monitor de Sistema & Rede/DNS** (`network_dns.rs` — inspeção de tabela DNS local em 3 camadas, *hosts* estáticos e `HotRouteTable` com abertura direta no Jinn Browser).

## Validação & Uso como Workspace

```bash
# Verificação de build bare-metal no_std
cargo check --target x86_64-unknown-none

# Execução da suíte de testes unitários no host
cargo test --features std
```

Em conjunto com o microkernel [`Jinn_Project`](https://github.com/GeovaneEstanislau/Jinn_Project), o binário `aladdin-server` empacota o workspace `Aladdin` dentro da imagem bootável (`scripts/run.ps1`).

## Licença

Licenciamento dual permissivo: **`MIT OR Apache-2.0`** ([`Cargo.toml`](Cargo.toml)), facilitando integração em pesquisas acadêmicas, ferramentas embarcadas e serviços isolados em espaço de usuário.
