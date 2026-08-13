//! A process-wide request to stop proving at the next stage boundary.

use std::sync::atomic::{AtomicU64, Ordering};

/// Bumped once per cancel request. A run captures this at start and compares later, so a
/// request left over from an earlier run reads as stale instead of cancelling this one.
///
/// One prover process proves one thing at a time, so the request needs no job identity.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Whether the current stage should run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum Proceed {
    /// No cancel has been requested since the run started.
    Continue,
    /// A cancel was requested; stop at this boundary.
    Cancelled,
}

impl Proceed {
    #[must_use]
    pub fn is_cancelled(self) -> bool {
        self == Self::Cancelled
    }
}

/// A proving run stopped at a stage boundary because a cancel was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("proving cancelled at stage `{stage}`")]
pub struct Cancelled {
    /// The stage that was about to run.
    pub stage: &'static str,
}

/// Asks the proving threads to stop at their next stage boundary.
pub fn request() {
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

/// The request count a run captures at start. Only meaningful compared against a later
/// reading — an unchanged value means no cancel arrived in between.
#[must_use]
pub fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}
