use std::sync::atomic::{AtomicU64, Ordering};

// Zero means no command has been submitted in this PTY session yet.
pub fn record_output(activity: &AtomicU64, now: u64) {
    let _ = activity.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |last| {
        (last > 0).then_some(last.max(now))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_output_stays_idle_until_submission() {
        let activity = AtomicU64::new(0);
        for now in [100, 200, 10_000] {
            record_output(&activity, now);
            assert_eq!(activity.load(Ordering::Relaxed), 0);
        }
        activity.store(10_001, Ordering::Relaxed);
        record_output(&activity, 10_010);
        assert_eq!(activity.load(Ordering::Relaxed), 10_010);
    }

    #[test]
    fn late_output_does_not_move_activity_backwards() {
        let activity = AtomicU64::new(200);
        record_output(&activity, 100);
        assert_eq!(activity.load(Ordering::Relaxed), 200);
    }
}
