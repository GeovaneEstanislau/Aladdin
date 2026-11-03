# ALADDIN FILE MANAGER 0.1.0 — MANUAL TÉCNICO E DE OPERAÇÃO

**Aplicação:** Aladdin File Manager (Aladdin Files)  
**Versão:** 0.1.0  
**Ambiente:** Aladdin Desktop OS  
**Data:** Novembro de 2026  
**Status:** Implementado, Integrado e Validado  

---

## 1. Visão Geral da Aplicação

O **Aladdin File Manager 0.1.0** é o gerenciador de arquivos oficial do ambiente desktop Aladdin. Desenvolvido em Rust `#![no_std]` sem dependências externas, o aplicativo conecta a interface gráfica moderna aos serviços reais de armazenamento do microkernel Jinn.

Diferente de demonstrações visuais mockadas, o Aladdin Files opera com transações reais de entrada/saída sobre os volumes montados, refletindo o espaço físico em tempo real e validando permissões de leitura e gravação em cada operação.

### Características Principais:
- **Design System Aladdin:** Estética corporativa e moderna fundamentada em paleta escura (Slate/Navy) com realces em Emerald, Cyan e Amber.
- **Barra Lateral de Volumes:** Exibição clara e seleção instantânea entre múltiplos volumes (ex.: `/disk` FAT32 Leitura/Escrita e `/ext2` Somente-Leitura).
- **Navegação e Breadcrumb:** Rastreamento do caminho atual com botão `< Voltar` e proteção contra estouro de limite na raiz.
- **Barra de Ferramentas Funcional:** Botões de ação rápida: `+ Pasta`, `+ Arquivo`, `Copiar`, `Renomear`, `Excluir` e `Atualizar`.
- **Lista de Arquivos e Metadados:** Exibição com tags tipadas (`[DIR]`, `[ELF]`, `[TXT]`, `[ARQ]`) e ações rápidas (`> Rodar` para executáveis Ring 3 e visualizador rápido para textos).
- **Telemetria de Armazenamento ao Vivo:** Rodapé com capacidade total, espaço livre e modo de acesso atualizados dinamicamente via `vfs::statvfs`.
- **Diálogos Modais Integrados:** Janelas de diálogo para confirmação de exclusão, nomeação de novas pastas/arquivos e renomeação com suporte a teclado.

---

## 2. Integração com os Contratos do Kernel

O aplicativo não acessa ponteiros ou estruturas privadas do Ring 0. Toda a comunicação ocorre por meio de interfaces contratuais formais definidas em `aladdin::contracts`:

### 2.1. Trait `VfsHost`
```rust
pub trait VfsHost {
    fn readdir(&self, path: &str, cb: &mut dyn FnMut(&str, bool, bool, bool));
    fn read_file(&self, path: &str, dest: &mut [u8]) -> Result<usize, ()>;
    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), ()>;
    fn create_file(&self, path: &str) -> Result<(), ()>;
    fn create_dir(&self, path: &str) -> Result<(), ()>;
    fn remove_file(&self, path: &str) -> Result<(), ()>;
    fn rename_file(&self, old_path: &str, new_path: &str) -> Result<(), ()>;
    fn copy_file(&self, src_path: &str, dst_path: &str) -> Result<(), ()>;
    fn file_size(&self, path: &str) -> Result<usize, ()>;
}
```

### 2.2. Trait `StorageHost`
Utilizada para consulta dos volumes do sistema (`storage_volume_count` e `storage_get_volume`), informando capacidade em MB, ponto de montagem e modo de permissão.

---

## 3. Manual de Operação do Usuário

### 3.1. Como Abrir o Gerenciador de Arquivos
1. **Pelo Dock Plank:** Clique no terceiro ícone inferior do dock: `[~] Arquivos`.
2. **Pelo Terminal Aladdin:** Digite o comando `files` e pressione Enter.
3. **Pelo Menu Iniciar:** Selecione a opção "Gerenciador de Arquivos".

### 3.2. Navegação em Pastas e Volumes
- **Alternar de Volume:** Clique no volume desejado na barra lateral esquerda (`[D] /disk (RW)` ou `[E] /ext2 (RO)`).
- **Entrar em uma Pasta:** Clique na linha da pasta com a tag `[DIR]`.
- **Subir de Diretório:** Clique no botão azul `< Voltar` localizado na barra de endereço superior.

### 3.3. Criação de Pastas e Arquivos
1. Clique em **`+ Pasta`** ou **`+ Arquivo`** na barra de ferramentas superior.
2. O diálogo modal solicitará o nome do novo item.
3. Digite o nome desejado no teclado (caracteres ASCII padrão 8.3).
4. Pressione **Enter** ou clique em **Criar**. Para cancelar, pressione **ESC** ou clique em **Cancelar**.

### 3.4. Renomear e Excluir Itens
1. Clique sobre o arquivo ou diretório desejado na lista para selecioná-lo.
2. Para renomear: clique no botão **`Renomear`**, informe o novo nome e confirme.
3. Para excluir: clique no botão **`Excluir`** e confirme no diálogo de segurança.

---

## 4. Matriz de Testes e Validação Técnica

| Caso de Teste | Módulo | Descrição | Resultado |
| :--- | :--- | :--- | :--- |
| **Proteção de Volume RO** | `aladdin_tests.rs` | Tentar criar/renomear/excluir no volume `/ext2` | **Aprovado (Rejeitado com erro)** |
| **Navegação de Diretórios** | `aladdin_tests.rs` | Navegação `into` e `up` com limite seguro na raiz | **Aprovado (100%)** |
| **CRUD e Diálogos** | `aladdin_tests.rs` | Fluxo completo de criação, renomeação, exclusão e input | **Aprovado (100%)** |
| **Live GUI no QEMU** | QEMU Live Test | Abertura da janela, renderização de breadcrumbs e itens reais | **Aprovado (Captura confirmada)** |
| **Persistência Cold-Reboot**| `test_persistence.ps1`| Gravação em FAT32, flush físico e reinicialização a frio | **Aprovado (100%)** |

---

## 6. Matriz de Compatibilidade do Ecossistema

| Recurso | Implementado | Testado | Validado |
| :--- | :---: | :---: | :---: |
| **IDE/ATA** | Sim | Sim | Sim |
| **AHCI** | Parcial | Sim | Parcial |
| **NVMe** | Parcial | Sim | Parcial |
| **VirtIO-Blk** | Sim | Sim | Sim |
| **MBR** | Sim | Sim | Sim |
| **GPT** | Sim | Sim | Sim |
| **FAT32** | Sim | Sim | Sim |
| **Ext2** | Sim | Sim | Sim |
| **VFS** | Sim | Sim | Sim |
| **Ring 3 Storage API** | Sim | Sim | Sim |
| **Aladdin File Manager** | Sim | Sim | Sim |

