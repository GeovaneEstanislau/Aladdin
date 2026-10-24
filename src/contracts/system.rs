//! Contrato de Telemetria e Servicos de Sistema

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
    pub jinnwall_active_conns: usize,
    pub jinnwall_drops: u64,
    pub jinnwall_inspected: u64,
    pub jinnwall_accepted: u64,
    pub jinnwall_profile: JinnWallProfile,
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
}
