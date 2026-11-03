//! Contrato de Sistema de Arquivos (VFS)

pub trait VfsHost {
    /// Enumera entradas de um diretório chamando o callback para cada entrada: (nome, is_dir, is_elf, is_txt)
    fn readdir(&self, path: &str, cb: &mut dyn FnMut(&str, bool, bool, bool));

    /// Lê o conteúdo de um arquivo para um buffer de memória
    fn read_file(&self, path: &str, buf: &mut [u8]) -> Result<usize, ()>;

    /// Escreve dados em um arquivo
    fn write_file(&self, path: &str, data: &[u8]) -> Result<(), ()>;

    /// Cria um novo arquivo vazio
    fn create_file(&self, _path: &str) -> Result<(), ()> {
        Ok(())
    }

    /// Cria um novo diretório
    fn create_dir(&self, _path: &str) -> Result<(), ()> {
        Ok(())
    }

    /// Remove um arquivo ou diretório
    fn remove_file(&self, path: &str) -> Result<(), ()>;

    /// Renomeia ou move um arquivo
    fn rename_file(&self, _old_path: &str, _new_path: &str) -> Result<(), ()> {
        Ok(())
    }

    /// Copia um arquivo
    fn copy_file(&self, _src_path: &str, _dst_path: &str) -> Result<(), ()> {
        Ok(())
    }

    /// Obtém o tamanho em bytes de um arquivo
    fn file_size(&self, _path: &str) -> Result<usize, ()> {
        Ok(0)
    }
}
