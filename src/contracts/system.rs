//! Contrato de Telemetria e Servicos de Sistema

use crate::contracts::storage::StorageHost;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JinnWallProfile {
    PublicSecure,
    Development,
}

impl JinnWallProfile {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::PublicSecure => "PublicSecure",
            Self::Development => "Development",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SystemStats {
    pub uptime_secs: usize,
    pub task_count: usize,
    pub sched_ticks: u64,
    pub used_ram_mb: usize,
    pub total_ram_mb: usize,
    pub current_tick: u64,
    pub net_ip: [u8; 4],
    pub net_adapter: &'static str,
    pub net_state_desc: &'static str,
    pub jinnwall_active_conns: usize,
    pub jinnwall_drops: u64,
    pub jinnwall_inspected: u64,
    pub jinnwall_accepted: u64,
    pub jinnwall_profile: JinnWallProfile,
    pub physical_frames_used: usize,
    pub physical_frames_total: usize,
    pub dynamic_pages_allocated: usize,
    pub dynamic_pages_freed: usize,
    pub vfs_open_fds: usize,
    pub vfs_mounted_volumes: usize,
    pub net_tx_packets: u64,
    pub net_rx_packets: u64,
    pub cpu_active_cycles: u64,
    pub cpu_idle_cycles: u64,
    pub syscalls_total: u64,
    pub memory_active_bytes: u64,
    pub memory_idle_bytes: u64,
}

impl Default for SystemStats {
    fn default() -> Self {
        Self {
            uptime_secs: 0,
            task_count: 0,
            sched_ticks: 0,
            used_ram_mb: 0,
            total_ram_mb: 0,
            current_tick: 0,
            net_ip: [0; 4],
            net_adapter: "",
            net_state_desc: "",
            jinnwall_active_conns: 0,
            jinnwall_drops: 0,
            jinnwall_inspected: 0,
            jinnwall_accepted: 0,
            jinnwall_profile: JinnWallProfile::PublicSecure,
            physical_frames_used: 0,
            physical_frames_total: 0,
            dynamic_pages_allocated: 0,
            dynamic_pages_freed: 0,
            vfs_open_fds: 0,
            vfs_mounted_volumes: 0,
            net_tx_packets: 0,
            net_rx_packets: 0,
            cpu_active_cycles: 0,
            cpu_idle_cycles: 0,
            syscalls_total: 0,
            memory_active_bytes: 0,
            memory_idle_bytes: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityMode {
    /// Renderizacao 100% opaca, maxima economia de ciclos de CPU
    Basic,
    /// Alpha blending linear leve, sombras discretas de 1 passo
    Balanced,
    /// Composicao avancada com desfoque e acentuacoes luminescentes
    Advanced,
}

pub trait SystemHost {
    fn stats(&self) -> SystemStats;
    fn yield_cpu(&self);
    fn storage_profile_name(&self) -> &'static str;
    fn cycle_storage_profile(&self);
    fn jinnwall_profile(&self) -> JinnWallProfile;
    fn set_jinnwall_profile(&self, profile: JinnWallProfile);
    fn toggle_jinnwall_profile(&self) -> JinnWallProfile;
    fn reboot(&self) -> !;

    // ── Ciclo de Energia e Atualização Offline no Desligamento ──────────────────
    fn shutdown(&self) -> ! {
        self.reboot();
    }

    fn set_display_power(&self, _on: bool) {}

    fn has_pending_updates(&self) -> bool {
        true
    }

    fn pending_updates_count(&self) -> usize {
        2
    }

    fn stage_update_package(&self, _pkg_name: &str) -> bool {
        true
    }

    fn apply_offline_updates_and_shutdown(&self) -> ! {
        self.set_display_power(false);
        self.shutdown();
    }

    // Métodos opcionais de integração com o subsistema de armazenamento (StorageHost)
    fn storage_disk_count(&self) -> usize {
        crate::contracts::storage::DEFAULT_STORAGE_HOST.disk_count()
    }
    fn storage_get_disk(&self, index: usize) -> Option<crate::contracts::storage::DiskInfo> {
        crate::contracts::storage::DEFAULT_STORAGE_HOST.get_disk(index)
    }
    fn storage_volume_count(&self) -> usize {
        crate::contracts::storage::DEFAULT_STORAGE_HOST.volume_count()
    }
    fn storage_get_volume(&self, index: usize) -> Option<crate::contracts::storage::VolumeDisplayInfo> {
        crate::contracts::storage::DEFAULT_STORAGE_HOST.get_volume(index)
    }
    fn storage_mount_volume(&self, _volume_id: usize, _mount_point: &str) -> Result<(), &'static str> {
        Ok(())
    }
    fn storage_unmount_volume(&self, _volume_id: usize) -> Result<(), &'static str> {
        Ok(())
    }
    fn storage_format_volume(&self, _volume_id: usize, _fs_type: &str) -> Result<(), &'static str> {
        Ok(())
    }
}
