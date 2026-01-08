#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy)]
pub struct SharingConfig {
    pub uf_to_dl: bool,
    pub dl_to_uf: bool,
}

impl Default for SharingConfig {
    fn default() -> Self {
        Self { uf_to_dl: true, dl_to_uf: true }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DebugEqSharing {
    pub enabled: bool,
    pub max_reason_lits: usize,
    pub log_imports: bool,
    pub log_exports: bool,
    pub log_shared_stats: bool,
}

impl Default for DebugEqSharing {
    fn default() -> Self {
        Self {
            enabled: false,
            max_reason_lits: 8,
            log_imports: true,
            log_exports: true,
            log_shared_stats: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EngineConfig {
    pub sharing: SharingConfig,
    pub debug_eq: DebugEqSharing,
}
