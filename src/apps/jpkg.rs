//! Jinn Package (.jpkg) Container Format Specification and Parser
//!
//! Compact, zero-heap container for verifiable software packages in Ring 3.

pub const JPKG_MAGIC: [u8; 4] = [0x4A, 0x50, 0x4B, 0x47]; // b"JPKG"
pub const JPKG_HEADER_SIZE: usize = 128;
pub const JPKG_FORMAT_VERSION: u16 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JpkgHeader {
    pub magic: [u8; 4],
    pub format_version: u16,
    pub flags: u16,
    pub name: [u8; 32],
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub _reserved: u16,
    pub category: [u8; 32],
    pub manifest_size: u32,
    pub elf_size: u32,
    pub resources_size: u32,
    pub elf_offset: u32,
    pub checksum: [u8; 32],
}

const _: () = assert!(core::mem::size_of::<JpkgHeader>() == JPKG_HEADER_SIZE);

impl JpkgHeader {
    pub fn empty() -> Self {
        Self {
            magic: JPKG_MAGIC,
            format_version: JPKG_FORMAT_VERSION,
            flags: 0,
            name: [0u8; 32],
            version_major: 1,
            version_minor: 0,
            version_patch: 0,
            _reserved: 0,
            category: [0u8; 32],
            manifest_size: 0,
            elf_size: 0,
            resources_size: 0,
            elf_offset: JPKG_HEADER_SIZE as u32,
            checksum: [0u8; 32],
        }
    }

    pub fn parse(buf: &[u8]) -> Result<Self, &'static str> {
        if buf.len() < JPKG_HEADER_SIZE {
            return Err("Buffer menor que o cabecalho JPKG");
        }
        let header = unsafe { core::ptr::read_unaligned(buf.as_ptr() as *const JpkgHeader) };
        if header.magic != JPKG_MAGIC {
            return Err("Magic bytes invalidos (esperado b'JPKG')");
        }
        if header.format_version != JPKG_FORMAT_VERSION {
            return Err("Versao de formato incompativel");
        }
        Ok(header)
    }

    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&c| c == 0).unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("app")
    }

    pub fn category_str(&self) -> &str {
        let len = self.category.iter().position(|&c| c == 0).unwrap_or(self.category.len());
        core::str::from_utf8(&self.category[..len]).unwrap_or("Geral")
    }
}

/// Representa um registro no catálogo `/disk/packages.db`
#[derive(Clone, Copy, Debug)]
pub struct CatalogEntry {
    pub name: &'static str,
    pub category: &'static str,
    pub version: &'static str,
    pub target_ver: &'static str,
    pub size_kb: usize,
    pub is_installed: bool,
    pub is_staged: bool,
}
