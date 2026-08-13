//! Per-stage progress reporting for GPU proving, emitted as `tracing` events.

use std::time::Instant;

use crate::cancel::{self, Cancelled, Proceed};

/// Reports each proving stage as it is entered, carrying the previous stage's duration.
///
/// Entry rather than exit, so the last line written names the stage a stalled prover is in.
/// The cancel baseline is captured at construction, so a request from an earlier run
/// cannot cancel this one.
pub struct StageTimer {
    started: Instant,
    entered: Instant,
    current: Option<&'static str>,
    enabled: bool,
    /// Cancel generation at construction; `None` never cancels.
    baseline: Option<u64>,
    finished: bool,
}

impl StageTimer {
    /// Reports every stage and stops at the first boundary after a cancel is requested.
    pub fn new() -> Self {
        Self::build(true, Some(cancel::generation()))
    }

    /// Reports every stage but never cancels — for callers whose return type cannot say
    /// "stopped early", so a cancel would have nowhere to go.
    pub fn uncancellable() -> Self {
        Self::build(true, None)
    }

    /// Neither reports nor cancels — for shivini's dry run pass, which sizes allocations
    /// rather than proving.
    pub fn silent() -> Self {
        Self::build(false, None)
    }

    fn build(enabled: bool, baseline: Option<u64>) -> Self {
        let now = Instant::now();
        Self {
            started: now,
            entered: now,
            current: None,
            enabled,
            baseline,
            finished: false,
        }
    }

    /// Whether a cancel arrived since this timer was built, without opening a new stage —
    /// for checkpoints inside a stage that runs long.
    pub fn check(&self) -> Proceed {
        let Some(baseline) = self.baseline else {
            return Proceed::Continue;
        };
        if cancel::generation() == baseline {
            return Proceed::Continue;
        }
        Proceed::Cancelled
    }

    /// Reports entry into `stage`, closing out whichever stage was running, and answers
    /// whether proving should go on. Every boundary is both a report and a checkpoint.
    pub fn enter(&mut self, stage: &'static str) -> Proceed {
        let now = Instant::now();
        let prev = self.current.replace(stage);
        let prev_ms = prev.map(|_| millis(self.entered, now));
        self.entered = now;

        if self.enabled {
            tracing::info!(stage, prev, prev_ms, "prover stage");
        }

        let proceed = self.check();
        if proceed.is_cancelled() && self.enabled {
            tracing::info!(stage, "prover cancelled at a stage boundary");
        }
        proceed
    }

    /// Reports entry into `stage`; the `?`-friendly form of [`StageTimer::enter`], naming
    /// the stage in the error.
    pub fn step(&mut self, stage: &'static str) -> Result<(), Cancelled> {
        if self.enter(stage).is_cancelled() {
            return Err(Cancelled { stage });
        }
        Ok(())
    }

    /// Reports the last stage closing and the total elapsed. Idempotent, so a cancel path
    /// can close the run out on its way past.
    pub fn finish(&mut self) {
        if !self.enabled || self.finished {
            return;
        }
        self.finished = true;
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

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, MutexGuard};

    use super::*;

    /// `request()` moves process-wide state, so the tests that call it take turns.
    static SERIAL: Mutex<()> = Mutex::new(());

    fn serial() -> MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn cancel_during_a_run_stops_the_next_stage() {
        let _guard = serial();
        let mut stages = StageTimer::new();

        assert_eq!(stages.enter("first"), Proceed::Continue);
        cancel::request();

        assert_eq!(stages.enter("second"), Proceed::Cancelled);
    }

    #[test]
    fn cancel_before_a_run_does_not_touch_it() {
        let _guard = serial();
        cancel::request();
        let mut stages = StageTimer::new();

        assert_eq!(stages.enter("first"), Proceed::Continue);
        assert_eq!(stages.check(), Proceed::Continue);
    }

    #[test]
    fn uncancellable_and_silent_timers_ignore_a_request() {
        let _guard = serial();
        let mut uncancellable = StageTimer::uncancellable();
        let mut silent = StageTimer::silent();
        cancel::request();

        assert_eq!(uncancellable.enter("stage"), Proceed::Continue);
        assert_eq!(silent.enter("stage"), Proceed::Continue);
    }

    #[test]
    fn step_names_the_stage_it_stopped_at() {
        let _guard = serial();
        let mut stages = StageTimer::new();
        cancel::request();

        assert_eq!(
            stages.step("quotient"),
            Err(Cancelled { stage: "quotient" })
        );
    }

    #[test]
    fn check_sees_a_cancel_without_opening_a_stage() {
        let _guard = serial();
        let stages = StageTimer::new();
        assert_eq!(stages.check(), Proceed::Continue);

        cancel::request();

        assert_eq!(stages.check(), Proceed::Cancelled);
    }
}
