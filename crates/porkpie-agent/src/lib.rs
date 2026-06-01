//! Background scheduling primitives for future Porkpie sync workers.

use std::time::{Duration, Instant};

/// Tracks when a periodic background job should run.
#[derive(Debug, Clone)]
pub struct AgentSchedule {
    interval: Duration,
    next_run: Instant,
}

impl AgentSchedule {
    /// Create a new schedule that is immediately due.
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            next_run: Instant::now(),
        }
    }

    /// Return true when the task is due.
    pub fn is_due(&self, now: Instant) -> bool {
        now >= self.next_run
    }

    /// Mark the task as completed and schedule the next run.
    pub fn mark_completed(&mut self, now: Instant) {
        self.next_run = now + self.interval;
    }
}

#[cfg(test)]
mod tests {
    use super::AgentSchedule;
    use std::time::{Duration, Instant};

    #[test]
    fn schedule_advances_after_completion() {
        let now = Instant::now();
        let mut schedule = AgentSchedule::new(Duration::from_secs(30));
        assert!(schedule.is_due(Instant::now() + Duration::from_millis(1)));

        schedule.mark_completed(now);
        assert!(!schedule.is_due(now + Duration::from_secs(1)));
        assert!(schedule.is_due(now + Duration::from_secs(31)));
    }
}
