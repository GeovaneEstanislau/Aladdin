# Aladdin — Out-of-Kernel Ring 3 Compositor & Desktop Shell

> **PT-BR:** [README.md](README.md) | **Ecosystem:** [Jinn_Project](https://github.com/GeovaneEstanislau/Jinn_Project) · [jhcs-rs](https://github.com/GeovaneEstanislau/jhcs-rs)

**Aladdin** is an **out-of-kernel Ring 3 compositor and desktop shell demonstrating isolated graphical services** built over explicit platform contracts with minimal memory overhead and zero mandatory GPU dependency.

---

## Core Architectural Pillars

- **Strict Ring 3 Graphics Isolation:** Window management, UI rendering, font blitting, and compositing execute strictly in user space—never inside the microkernel. Communication uses `AladdinSharedRings` (lock-free, zero-copy SPSC queues for framebuffer updates and input events).
- **Damage-Driven Compositing (`DirtyRect`) & Zero-GPU Acrylics:** Updates only invalidated screen regions and renders frosted-glass translucency via CPU SIMD blur (`box_blur_rgba_simd` via `jhcs-backend-simd`) with scalar fallback.
- **Event-Driven Zero-Idle Execution:** The compositor blocks via `wait_on` / `yield` whenever no input events or damaged surfaces are pending, preventing background polling loops.
- **Platform-Agnostic Trait Contracts:** Clean interfaces (`FramebufferDevice`, `InputEvent`, `SystemHost`, `VfsHost`, `ProcessHost`) in `src/contracts/` enable compilation both as bare-metal `#![no_std]` (`x86_64-unknown-none`) and under `--features std` for host unit testing.

## Module Layout

- `src/contracts/`: Typed contracts for input (`EVENT_MOUSE_*`, `EVENT_KEY_DOWN`, `EVENT_SCROLL`), framebuffer output, VFS, process lifecycle, and DNS/Hot-Route host telemetry.
- `src/compositor/`: Double-buffered ARGB rasterization, dirty rectangle aggregation, rounded borders, and alpha compositing.
- `src/wm/`: Z-order window manager, dragging/resizing, focus arbitration, and client event forwarding (`ClientWindowEvent`).
- `src/desktop/`: Top bar, dock, application launcher, quick settings, and compositor session loop (`session.rs`).
- `src/apps/`:
  - **Interactive Shell**;
  - **Multi-Volume File Manager** (FAT32 / Ext2);
  - **System Monitor & Network/DNS Inspector** (`network_dns.rs` — 3-layer local DNS table, static `/etc/hosts` editor, and `HotRouteTable` one-click browser launch).

## Verification & Integration

```bash
# Bare-metal #![no_std] check
cargo check --target x86_64-unknown-none

# Host unit tests
cargo test --features std
```

When built inside [`Jinn_Project`](https://github.com/GeovaneEstanislau/Jinn_Project), `apps/aladdin-server` links this repository directly as the Ring 3 display server (`scripts/build.ps1`).

## License

Dual-licensed under **`MIT OR Apache-2.0`** ([`Cargo.toml`](Cargo.toml)).
