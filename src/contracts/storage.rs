//! Contrato de Gerenciamento de Armazenamento, Discos e Partições (StorageHost)

pub const MAX_DISKS_INFO: usize = 8;
pub const MAX_VOLUMES_INFO: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskBus {
    IdeAta,
    AhciSata,
    NvmePci,
    VirtioBlk,
    RamDisk,
    Unknown,
}

impl DiskBus {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::IdeAta => "IDE / ATA",
            Self::AhciSata => "SATA (AHCI)",
            Self::NvmePci => "NVMe (PCIe)",
            Self::VirtioBlk => "VirtIO Blk",
            Self::RamDisk => "RAM Disk",
            Self::Unknown => "Desconhecido",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DiskInfo {
    pub id: usize,
    pub name: [u8; 32],
    pub name_len: usize,
    pub model: [u8; 40],
    pub model_len: usize,
    pub bus: DiskBus,
    pub capacity_mb: u64,
    pub sector_size: usize,
    pub is_read_only: bool,
    pub is_removable: bool,
    pub is_system_disk: bool,
}

impl DiskInfo {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            name: [0; 32],
            name_len: 0,
            model: [0; 40],
            model_len: 0,
            bus: DiskBus::Unknown,
            capacity_mb: 0,
            sector_size: 512,
            is_read_only: false,
            is_removable: false,
            is_system_disk: false,
        }
    }

    pub fn name_str(&self) -> &str {
        if self.name_len == 0 { "" } else { core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("") }
    }

    pub fn model_str(&self) -> &str {
        if self.model_len == 0 { "" } else { core::str::from_utf8(&self.model[..self.model_len]).unwrap_or("") }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VolumeDisplayInfo {
    pub id: usize,
    pub disk_id: usize,
    pub part_index: usize,
    pub label: [u8; 32],
    pub label_len: usize,
    pub mount_point: [u8; 32],
    pub mount_point_len: usize,
    pub fs_type: [u8; 16],
    pub fs_type_len: usize,
    pub total_mb: u64,
    pub used_mb: u64,
    pub free_mb: u64,
    pub is_mounted: bool,
    pub is_root: bool,
    pub is_busy: bool,
}

impl VolumeDisplayInfo {
    pub const fn empty() -> Self {
        Self {
            id: 0,
            disk_id: 0,
            part_index: 0,
            label: [0; 32],
            label_len: 0,
            mount_point: [0; 32],
            mount_point_len: 0,
            fs_type: [0; 16],
            fs_type_len: 0,
            total_mb: 0,
            used_mb: 0,
            free_mb: 0,
            is_mounted: false,
            is_root: false,
            is_busy: false,
        }
    }

    pub fn label_str(&self) -> &str {
        if self.label_len == 0 { "Sem Rotulo" } else { core::str::from_utf8(&self.label[..self.label_len]).unwrap_or("Sem Rotulo") }
    }

    pub fn mount_point_str(&self) -> &str {
        if self.mount_point_len == 0 { "" } else { core::str::from_utf8(&self.mount_point[..self.mount_point_len]).unwrap_or("") }
    }

    pub fn fs_type_str(&self) -> &str {
        if self.fs_type_len == 0 { "Desconhecido" } else { core::str::from_utf8(&self.fs_type[..self.fs_type_len]).unwrap_or("Desconhecido") }
    }
}

pub trait StorageHost {
    fn disk_count(&self) -> usize;
    fn get_disk(&self, index: usize) -> Option<DiskInfo>;
    fn volume_count(&self) -> usize;
    fn get_volume(&self, index: usize) -> Option<VolumeDisplayInfo>;
    fn mount_volume(&mut self, volume_id: usize, mount_point: &str) -> Result<(), &'static str>;
    fn unmount_volume(&mut self, volume_id: usize) -> Result<(), &'static str>;
    fn format_volume(&mut self, volume_id: usize, fs_type: &str) -> Result<(), &'static str>;
}

/// Implementação padrão com dados de fallback do sistema
pub struct NullStorageHost;

impl StorageHost for NullStorageHost {
    fn disk_count(&self) -> usize { 1 }

    fn get_disk(&self, index: usize) -> Option<DiskInfo> {
        if index == 0 {
            let mut d = DiskInfo::empty();
            let name = b"ide0 (Master)";
            d.name[..name.len()].copy_from_slice(name);
            d.name_len = name.len();
            let m = b"QEMU HARDDISK (ATA)";
            d.model[..m.len()].copy_from_slice(m);
            d.model_len = m.len();
            d.bus = DiskBus::IdeAta;
            d.capacity_mb = 64;
            d.is_system_disk = true;
            Some(d)
        } else {
            None
        }
    }

    fn volume_count(&self) -> usize { 2 }

    fn get_volume(&self, index: usize) -> Option<VolumeDisplayInfo> {
        if index == 0 {
            let mut v = VolumeDisplayInfo::empty();
            v.id = 0;
            let l = b"JINN_SYSTEM";
            v.label[..l.len()].copy_from_slice(l);
            v.label_len = l.len();
            let m = b"/disk";
            v.mount_point[..m.len()].copy_from_slice(m);
            v.mount_point_len = m.len();
            let fs = b"FAT32";
            v.fs_type[..fs.len()].copy_from_slice(fs);
            v.fs_type_len = fs.len();
            v.total_mb = 32;
            v.used_mb = 14;
            v.free_mb = 18;
            v.is_mounted = true;
            v.is_root = true;
            Some(v)
        } else if index == 1 {
            let mut v = VolumeDisplayInfo::empty();
            v.id = 1;
            let l = b"EXT2_DATA";
            v.label[..l.len()].copy_from_slice(l);
            v.label_len = l.len();
            let m = b"/ext2";
            v.mount_point[..m.len()].copy_from_slice(m);
            v.mount_point_len = m.len();
            let fs = b"Ext2";
            v.fs_type[..fs.len()].copy_from_slice(fs);
            v.fs_type_len = fs.len();
            v.total_mb = 32;
            v.used_mb = 2;
            v.free_mb = 30;
            v.is_mounted = true;
            v.is_root = false;
            Some(v)
        } else {
            None
        }
    }

    fn mount_volume(&mut self, _volume_id: usize, _mount_point: &str) -> Result<(), &'static str> { Ok(()) }
    fn unmount_volume(&mut self, _volume_id: usize) -> Result<(), &'static str> { Ok(()) }
    fn format_volume(&mut self, _volume_id: usize, _fs_type: &str) -> Result<(), &'static str> { Ok(()) }
}

pub static DEFAULT_STORAGE_HOST: NullStorageHost = NullStorageHost;

pub struct SystemHostStorageAdapter<'a>(pub &'a dyn crate::contracts::system::SystemHost);

impl<'a> StorageHost for SystemHostStorageAdapter<'a> {
    fn disk_count(&self) -> usize { self.0.storage_disk_count() }
    fn get_disk(&self, index: usize) -> Option<DiskInfo> { self.0.storage_get_disk(index) }
    fn volume_count(&self) -> usize { self.0.storage_volume_count() }
    fn get_volume(&self, index: usize) -> Option<VolumeDisplayInfo> { self.0.storage_get_volume(index) }
    fn mount_volume(&mut self, volume_id: usize, mount_point: &str) -> Result<(), &'static str> {
        self.0.storage_mount_volume(volume_id, mount_point)
    }
    fn unmount_volume(&mut self, volume_id: usize) -> Result<(), &'static str> {
        self.0.storage_unmount_volume(volume_id)
    }
    fn format_volume(&mut self, volume_id: usize, fs_type: &str) -> Result<(), &'static str> {
        self.0.storage_format_volume(volume_id, fs_type)
    }
}
