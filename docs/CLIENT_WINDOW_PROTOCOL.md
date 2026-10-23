# ESPECIFICAÇÃO NORMATIVA — PROTOCOLO DE JANELAS DE CLIENTE ALADDIN
## Contrato de Integração Gráfica para Aplicações em Ring 3 (Jinn OS & Aladdin Desktop)

**Versão da Norma:** 1.0.0  
**Status:** Oficial / Estável  
**Projetos Afetados:** `aladdin` (Compositor/WM) e `jinn` (Microkernel ABI)  
**Associação:** Issue Jinn [#47](https://github.com/GeovaneEstanislau/Jinn_Project/issues/47) / Aladdin [#3](https://github.com/GeovaneEstanislau/Aladdin/issues/3)  

---

## 1. Visão Geral

O **Aladdin Desktop** opera como o compositor gráfico e servidor de janelas nativo do **Jinn OS**. Aplicações executadas em modo de usuário desprivilegiado (Ring 3) não possuem acesso direto à memória de vídeo física do framebuffer (1024 × 768 × 32 bpp). 

A criação, atualização e encerramento de superfícies gráficas ocorre por meio da interface de chamadas de sistema (`int 0x80`), garantindo isolamento de memória, proteção de fronteiras geométricas e controle estrito de cotas por processo.

---

## 2. Syscalls Gráficas e Convenção de Chamada

As chamadas são efetuadas carregando o número da syscall em `RAX`, os argumentos em `RDI`, `RSI` e `RDX`, e executando `int 0x80`. O valor de retorno é entregue em `RAX` como um inteiro com sinal (`i64`), onde valores `>= 0` indicam sucesso e valores `< 0` indicam código de erro POSIX.

### 2.1. Syscall 20: `sys_window_create`
Cria uma nova janela de cliente gerenciada pelo compositor.

- **Entrada:**
  - `RAX`: `20`
  - `RDI`: Ponteiro virtual (`*const u8`) para a string UTF-8 do título da janela.
  - `RSI`: Comprimento (`usize`) do título em bytes (máximo 64 bytes).
  - `RDX`: Dimensões compactadas da área de cliente: `((width & 0xFFFF) << 16) | (height & 0xFFFF)`.
- **Validações:**
  - Largura (`width`): `1 <= width <= 1024` pixels.
  - Altura (`height`): `1 <= height <= 768` pixels.
  - Cota por processo: O processo proprietário não pode exceder `limits.max_windows` (padrão: 4 janelas).
  - Capacidade global do compositor: `MAX_WINDOWS = 8`.
- **Retorno:**
  - Sucesso: `win_id` (`usize` >= 0) identificador exclusivo da janela.
  - Erros:
    - `-EINVAL` (-22): Dimensões inválidas (largura/altura zero ou excedendo os limites da tela).
    - `-EFAULT` (-14): Ponteiro de título aponta para endereço fora do espaço de usuário ou não mapeado.
    - `-EMFILE` (-24): Cota de janelas do processo esgotada ou capacidade global do compositor cheia.
    - `-ENOSYS` (-38): Subsistema gráfico indisponível (boot em modo texto ou sem framebuffer).

### 2.2. Syscall 21: `sys_window_update`
Submete uma nova superfície de pixels para a janela cliente.

- **Entrada:**
  - `RAX`: `21`
  - `RDI`: Identificador da janela (`win_id`).
  - `RSI`: Ponteiro virtual (`*const u32`) para o buffer linear de pixels em formato `0x00RRGGBB`.
  - `RDX`: Tamanho do buffer em bytes (`width * height * 4`).
- **Validações:**
  - Existência da janela (`win_id < MAX_WINDOWS` e janela ativa).
  - Propriedade: `win.owner_pid == Some(calling_pid)`. Processos de terceiros são estritamente impedidos de alterar superfícies alheias.
  - Integridade dimensional: `RDX == client_width * client_height * 4`.
  - Ponteiro de usuário: Endereço canônico em espaço de usuário, alinhado e mapeado com permissão de leitura.
- **Retorno:**
  - Sucesso: `0`. O compositor marca a janela como danificada (`client_damaged = true`) e agenda a composição para o próximo ciclo de varredura.
  - Erros:
    - `-EBADF` (-9): Janela não encontrada ou já encerrada (`WindowOpError::NotFound`).
    - `-EPERM` (-1): Processo chamador não é o proprietário legítimo da janela (`WindowOpError::PermissionDenied`).
    - `-EINVAL` (-22): Tamanho do buffer incompatível com as dimensões da janela (`WindowOpError::InvalidDimensions`).
    - `-EFAULT` (-14): Buffer de pixels em endereço de memória inválido ou violando proteção de página.

### 2.3. Syscall 22: `sys_window_close`
Encerra a janela e libera os recursos gráficos alocados.

- **Entrada:**
  - `RAX`: `22`
  - `RDI`: Identificador da janela (`win_id`).
  - `RSI`: `0`
  - `RDX`: `0`
- **Validações:**
  - Existência da janela.
  - Propriedade (`owner_pid == calling_pid`).
- **Retorno:**
  - Sucesso: `0`. O buffer do cliente é desalocado e o retângulo envolvente da janela (`bounds()`) é registrado na fila de dirty rects do compositor para redesenho do fundo da área de trabalho. A cota `task.resources.open_windows` do processo é decrementada.
  - Erros:
    - `-EBADF` (-9): Janela inexistente ou handle inválido.
    - `-EPERM` (-1): Violação de permissão cross-process.

---

## 3. Formato de Pixels e Modelo de Renderização

- **Profundidade:** 32 bits por pixel (bpp).
- **Ordem de Bytes (Little-Endian / x86-64):**
  - Bits 0..7: Azul (Blue, 0x00..0xFF)
  - Bits 8..15: Verde (Green, 0x00..0xFF)
  - Bits 16..23: Vermelho (Red, 0x00..0xFF)
  - Bits 24..31: Canal Alpha / Reservado (`0x00` opaco na composição básica)
- **Área do Cliente vs Decoração:**
  - A janela é envolvida automaticamente pela moldura do Aladdin: barra de título (`TITLE_BAR_HEIGHT = 24` pixels) e bordas laterais/inferiores (2 pixels).
  - O buffer fornecido pelo cliente preenche estritamente a área interna útil (`client_width × client_height`).
- **Scissor Clipping:**
  - O compositor aplica recorte retangular de tesoura (`ClipRect`) na área cliente, impedindo que pixels desenhados por clientes maliciosos ou descalibrados vazem para a barra de título, para o dock ou para janelas sobrepostas.

---

## 4. Ciclo de Vida e Limpeza Automática de Recursos

Para assegurar imunidade contra vazamento de recursos e janelas zumbis na área de trabalho:

1. **Encerramento Normal:** A aplicação deve chamar `sys_window_close` para cada janela antes de finalizar via `sys_exit(0)`.
2. **Encerramento Anormal / Crash:** Se uma aplicação sofrer encerramento prematuro (ex: violação de instrução, esgotamento de cotas ou finalização forçada), a rotina do kernel `scheduler::exit_task` aciona automaticamente `gui::close_windows_for_pid(pid)`:
   - Todas as janelas associadas a `owner_pid` são removidas da lista do `WindowManager`.
   - Os buffers de pixels associados são imediatamente desalocados.
   - A região antes ocupada pelas janelas é invalidada e repintada pelo desktop.
   - A flag atômica `task.cleanup_done` garante que a limpeza execute estritamente uma única vez (idempotência).

---

## 5. Mapeamento de Erros e Tabela de Conversão

| Código Aladdin (`WindowOpError`) | Código Kernel (`syscall::*`) | Valor Inteiro | Descrição Semântica |
| :--- | :--- | :--- | :--- |
| `Ok(())` | `0` | `0` | Operação completada com sucesso. |
| `WindowOpError::NotFound` | `EBADF` | `-9` | Handle de janela inválido ou inexistente. |
| `WindowOpError::PermissionDenied` | `EPERM` | `-1` | Operação rejeitada por violação de propriedade entre processos. |
| `WindowOpError::InvalidDimensions`| `EINVAL` | `-22` | Dimensão zero, superior à tela ou buffer com tamanho inconsistente. |
| *(Falha de Alocação de Janela)* | `EMFILE` | `-24` | Cota máxima de janelas por tarefa ou capacidade global esgotada. |
| *(Sub-sistema Indisponível)* | `ENOSYS` | `-38` | Compositor gráfico não inicializado ou operando em modo texto. |
