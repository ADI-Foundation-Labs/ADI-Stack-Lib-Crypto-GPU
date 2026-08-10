//! Per-stage progress reporting for GPU proving, emitted as `tracing` events.

use std::time::Instant;

/// Reports each proving stage as it is entered, carrying the previous stage's duration.
///
/// Entry rather than exit, so the last line written names the stage a stalled prover is in.
pub struct StageTimer {
    started: Instant,
    entered: Instant,
    current: Option<&'static str>,
    enabled: bool,
}

impl StageTimer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            entered: now,
            current: None,
            enabled: true,
        }
    }

    /// A timer that reports nothing — for shivini's dry run pass, which proves no real trace.
    pub fn silent() -> Self {
        Self {
            enabled: false,
            ..Self::new()
        }
    }

    /// Reports entry into `stage`, closing out whichever stage was running.
    pub fn enter(&mut self, stage: &'static str) {
        let now = Instant::now();
        if self.enabled {
            tracing::info!(
                stage,
                prev = self.current,
                prev_ms = self.current.map(|_| millis(self.entered, now)),
                "gpu prover stage"
            );
        }
        self.current = Some(stage);
        self.entered = now;
    }

    /// Reports the last stage closing and the total elapsed.
    pub fn finish(self) {
        if !self.enabled {
            return;
        }
        let now = Instant::now();
        tracing::info!(
            prev = self.current,
            prev_ms = self.current.map(|_| millis(self.entered, now)),
            total_ms = millis(self.started, now),
            "gpu prover finished"
        );
    }
}

impl Default for StageTimer {
    fn default() -> Self {
        Self::new()
    }
}

fn millis(from: Instant, to: Instant) -> u64 {
    u64::try_from(to.duration_since(from).as_millis()).unwrap_or(u64::MAX)
}
