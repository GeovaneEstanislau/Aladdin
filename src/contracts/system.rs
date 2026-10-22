//! Contrato de Telemetria e Serviços de Sistema

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityMode {
    /// Renderização 100% opaca, máxima economia de ciclos de CPU
    Basic,
    /// Alpha blending linear leve, sombras discretas de 1 passo
    Balanced,
    /// Composição avançada com desfoque e acentuações luminescentes
    Advanced,
}

pub trait SystemHost {
    fn stats(&self) -> SystemStats;
    fn yield_cpu(&self);
    fn storage_profile_name(&self) -> &'static str;
    fn cycle_storage_profile(&self);
    fn reboot(&self) -> !;
}
