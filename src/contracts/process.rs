//! Contrato de Execução de Processos e Despacho de Comandos de Terminal

pub trait ProcessHost {
    /// Executa uma linha de comando no shell do sistema e despacha o resultado linha a linha: (linha, cor_argb)
    fn execute_command(&mut self, cmd: &str, output: &mut dyn FnMut(&str, u32));

    /// Spawna um processo de usuário (Ring 3) a partir de um caminho ELF no VFS
    fn spawn_app(&mut self, path: &str) -> Result<usize, ()>;

    /// Verifica se um processo com o determinado PID ainda está ativo no sistema
    fn is_process_alive(&self, pid: usize) -> bool {
        let _ = pid;
        true
    }
}
