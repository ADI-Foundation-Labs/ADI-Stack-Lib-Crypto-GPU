//! Per-stage progress reporting for GPU proving, emitted as `tracing` events.

use std::time::Instant;

use crate::cancel::{self, Proceed};

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

    /// A timer that neither reports nor cancels — for shivini's dry run pass, which sizes
    /// allocations rather than proving. Cancelling it would buy nothing and would strand a
    /// `None` in the non-cancellable callers that drive it.
    pub fn silent() -> Self {
        Self {
            enabled: false,
            ..Self::new()
        }
    }

    /// Reports entry into `stage`, closing out whichever stage was running, and answers
    /// whether proving should go on. Every boundary is both a report and a checkpoint.
    pub fn enter(&mut self, stage: &'static str) -> Proceed {
        let now = Instant::now();
        let prev = self.current.replace(stage);
        let prev_ms = prev.map(|_| millis(self.entered, now));
        self.entered = now;

        if !self.enabled {
            return Proceed::Continue;
        }

        tracing::info!(stage, prev, prev_ms, "prover stage");

        let proceed = cancel::requested();
        if proceed.is_cancelled() {
            tracing::info!(stage, "prover cancelled at a stage boundary");
        }
        proceed
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
            "prover stage total"
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
