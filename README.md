# Aladdin Desktop — Ambiente Gráfico para Jinn OS

O **Aladdin** é o ambiente de desktop gráfico do **Jinn OS**, combinando:
- **Design:** Clareza e intencionalidade de layout inspiradas no **elementary OS 7** (Wingpanel, Plank Dock, botões semáforo) com a personalidade escura e contrastes luminescentes néon do **Garuda Linux (Dr460nized)**.
- **Eficiência de Recursos:** Compositor com *Damage Tracking* estrito (retângulos sujos), motor de cursor rápido (< 5 µs), zero espera ocupada (`yield`), e cache de I/O de arquivos.
- **Modularidade:** Totalmente desacoplado do kernel do Jinn OS através de contratos formais de plataforma (`FramebufferDevice`, `InputEvent`, `SystemHost`, `VfsHost`, `ProcessHost`).
- **Suporte no_std:** Executa em bare-metal no target `x86_64-unknown-none` com alocação contígua e pode ser compilado com a feature `std` para testes unitários no host (Windows/Linux).

## Estrutura de Módulos

- `contracts`: Traits e estruturas de dados de integração com a plataforma subjacente.
- `compositor`: Double buffering, rastreamento de dano (`DirtyRect`), primitivas 2D e paleta de cores ARGB.
- `font`: Renderização de texto bitmap 8x8 monocromático.
- `wm`: Gerenciamento de janelas, Z-order, controles (minimizar, maximizar, fechar) e arrasto.
- `desktop`: Wingpanel (barra superior), Plank Dock (barra inferior), menu iniciar e loop de sessão.
- `apps`: Terminal interativo integrado ao shell do sistema, Gerenciador de arquivos multi-volume (FAT32/Ext2) com Quick Look, Monitor de sistema e Sobre o Jinn OS.
