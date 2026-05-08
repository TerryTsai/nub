//! Engine wire shapes for `GET /containers/{id}/stats?stream=true`. Each
//! NDJSON line decodes into `RawStats`; we compute deltas in `stats.rs`.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub(in crate::ops::containers) struct RawStats {
    #[serde(default)]
    pub cpu_stats: RawCpu,
    #[serde(default)]
    pub precpu_stats: RawCpu,
    #[serde(default)]
    pub memory_stats: RawMem,
    #[serde(default)]
    pub networks: Option<HashMap<String, RawNet>>,
}

#[derive(Default, Deserialize)]
pub(in crate::ops::containers) struct RawCpu {
    #[serde(default)]
    pub cpu_usage: RawCpuUsage,
    #[serde(default)]
    pub system_cpu_usage: u64,
    #[serde(default)]
    pub online_cpus: u64,
}

#[derive(Default, Deserialize)]
pub(in crate::ops::containers) struct RawCpuUsage {
    #[serde(default)]
    pub total_usage: u64,
    #[serde(default)]
    pub percpu_usage: Option<Vec<u64>>,
}

#[derive(Default, Deserialize)]
pub(in crate::ops::containers) struct RawMem {
    #[serde(default)]
    pub usage: u64,
    #[serde(default)]
    pub limit: u64,
}

#[derive(Default, Deserialize)]
pub(in crate::ops::containers) struct RawNet {
    #[serde(default)]
    pub rx_bytes: u64,
    #[serde(default)]
    pub tx_bytes: u64,
}
