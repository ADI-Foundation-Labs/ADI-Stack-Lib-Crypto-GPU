//! A process-wide request to stop proving at the next stage boundary.

use std::sync::atomic::{AtomicBool, Ordering};

/// One prover process proves one thing at a time, so the request needs no job identity.
static CANCELLED: AtomicBool = AtomicBool::new(false);

/// Whether the current stage should run.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proceed {
    Continue,
    Cancelled,
}

impl Proceed {
    #[must_use]
    pub fn is_cancelled(self) -> bool {
        self == Self::Cancelled
    }
}

/// Clears any earlier request. Call this before every proving run, or a stale request
/// cancels the next one instantly.
pub fn arm() {
    CANCELLED.store(false, Ordering::Release);
}

/// Asks the proving threads to stop at their next stage boundary.
pub fn request() {
    CANCELLED.store(true, Ordering::Release);
}

/// Whether a stop has been requested.
pub fn requested() -> Proceed {
    if CANCELLED.load(Ordering::Acquire) {
        return Proceed::Cancelled;
    }
    Proceed::Continue
}
