//! Per-frame stream chunks — log/exec output, stats samples, pull/build
//! progress, and stream-end markers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    Log {
        stderr: bool,
        data: String,
    },
    Stats {
        cpu_pct: f64,
        mem_used: u64,
        mem_limit: u64,
        net_rx: u64,
        net_tx: u64,
    },
    Lagging {
        dropped: u32,
    },
    Stdin {
        data: String,
    },
    StdinClose,
    PullProgress {
        id: String,
        status: String,
        current: u64,
        total: u64,
    },
    BuildProgress {
        /// Engine output line. Newline-terminated; UI concatenates as-is.
        stream: String,
        /// Final image ID (sha256:…) on the last `aux` chunk, otherwise None.
        image_id: Option<String>,
    },
    End {
        ok: bool,
        err: Option<String>,
    },
}
