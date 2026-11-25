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

/// Erro de verificação criptográfica e estrutural de pacote `.jpkg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JpkgVerifyError {
    TooSmall,
    InvalidHeader,
    OutOfBounds,
    ChecksumMismatch,
}

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Implementação SHA-256 pura `#![no_std]` e zero-alloc para verificação de integridade de pacotes `.jpkg`.
#[derive(Clone, Copy)]
pub struct Sha256Hasher {
    state: [u32; 8],
    buffer: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Sha256Hasher {
    pub const fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            buffer: [0u8; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.total_len += data.len() as u64;
        let mut offset = 0;

        if self.buf_len > 0 {
            let to_copy = (64 - self.buf_len).min(data.len());
            self.buffer[self.buf_len..self.buf_len + to_copy].copy_from_slice(&data[..to_copy]);
            self.buf_len += to_copy;
            offset += to_copy;
            if self.buf_len == 64 {
                let block = self.buffer;
                self.process_block(&block);
                self.buf_len = 0;
            }
        }

        while offset + 64 <= data.len() {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[offset..offset + 64]);
            self.process_block(&block);
            offset += 64;
        }

        if offset < data.len() {
            let rem = data.len() - offset;
            self.buffer[..rem].copy_from_slice(&data[offset..]);
            self.buf_len = rem;
        }
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total_len.wrapping_mul(8);
        self.buffer[self.buf_len] = 0x80;
        self.buf_len += 1;

        if self.buf_len > 56 {
            for b in &mut self.buffer[self.buf_len..64] {
                *b = 0;
            }
            let block = self.buffer;
            self.process_block(&block);
            self.buf_len = 0;
        }

        for b in &mut self.buffer[self.buf_len..56] {
            *b = 0;
        }
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let block = self.buffer;
        self.process_block(&block);

        let mut digest = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        digest
    }

    fn process_block(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

/// Valida o cabeçalho JPKG e o hash SHA-256 (manifesto + payload ELF) antes de retornar a fatia executável.
pub fn verify_and_extract_elf(pkg: &[u8]) -> Result<(JpkgHeader, &[u8]), JpkgVerifyError> {
    if pkg.len() < JPKG_HEADER_SIZE {
        return Err(JpkgVerifyError::TooSmall);
    }
    let header = JpkgHeader::parse(pkg).map_err(|_| JpkgVerifyError::InvalidHeader)?;

    let manifest_start = JPKG_HEADER_SIZE;
    let manifest_end = manifest_start
        .checked_add(header.manifest_size as usize)
        .ok_or(JpkgVerifyError::OutOfBounds)?;
    let elf_offset = header.elf_offset as usize;
    let elf_size = header.elf_size as usize;
    let elf_end = elf_offset
        .checked_add(elf_size)
        .ok_or(JpkgVerifyError::OutOfBounds)?;
    let res_end = elf_end
        .checked_add(header.resources_size as usize)
        .ok_or(JpkgVerifyError::OutOfBounds)?;

    if elf_size == 0
        || manifest_end > pkg.len()
        || elf_offset < manifest_end
        || elf_end > pkg.len()
        || res_end > pkg.len()
    {
        return Err(JpkgVerifyError::OutOfBounds);
    }

    let mut hasher = Sha256Hasher::new();
    hasher.update(&pkg[manifest_start..manifest_end]);
    hasher.update(&pkg[elf_offset..elf_end]);
    let actual = hasher.finalize();

    let mut diff = 0u8;
    for i in 0..32 {
        diff |= actual[i] ^ header.checksum[i];
    }
    if diff != 0 {
        return Err(JpkgVerifyError::ChecksumMismatch);
    }

    Ok((header, &pkg[elf_offset..elf_end]))
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
