//! Contrato de Sistema de Arquivos (VFS)

pub trait VfsHost {
    /// Enumera entradas de um diretório chamando o callback para cada entrada: (nome, is_dir, is_elf, is_txt)
    fn readdir(&self, path: &str, cb: &mut dyn FnMut(&str, bool, bool, bool));

    /// Lê o conteúdo de um arquivo para um buffer de memória
    fn read_file(&self, path: &str, buf: &mut [u8]) -> Result<usize, ()>;

    /// Escreve dados em um arquivo
    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), ()>;

    /// Remove um arquivo
    fn remove_file(&self, path: &str) -> Result<(), ()>;
}
