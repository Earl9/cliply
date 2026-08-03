use crate::error::CliplyError;
use crate::logger;
use crate::services::remote_sync_service;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const INITIAL_DELAY: Duration = Duration::from_secs(20);
const CHECK_INTERVAL: Duration = Duration::from_secs(60);
// Consecutive failures double the wait (2m, 4m, ... up to 30m) so an
// unreachable provider does not burn a connection attempt and an error log
// line every single minute.
const MAX_BACKOFF: Duration = Duration::from_secs(30 * 60);

pub struct AutoSyncSchedulerShutdown {
    running: Arc<AtomicBool>,
}

impl Drop for AutoSyncSchedulerShutdown {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub fn start_auto_sync_scheduler(app: AppHandle) -> Result<AutoSyncSchedulerShutdown, CliplyError> {
    let running = Arc::new(AtomicBool::new(true));
    let thread_running = Arc::clone(&running);
    thread::Builder::new()
        .name("cliply-auto-sync".to_string())
        .spawn(move || {
            if !sleep_while_running(&thread_running, INITIAL_DELAY) {
                return;
            }

            let mut consecutive_failures = 0u32;
            let mut last_error_message = String::new();

            while thread_running.load(Ordering::SeqCst) {
                match remote_sync_service::run_auto_sync_cycle(&app) {
                    Ok(Some(result)) => {
                        consecutive_failures = 0;
                        last_error_message.clear();
                        logger::info(
                            &app,
                            "auto_sync_cycle",
                            format!(
                                "exported={} imported={} updated={} deleted={} conflicted={} snapshots={}",
                                result.exported_count,
                                result.imported_count,
                                result.updated_count,
                                result.deleted_count,
                                result.conflicted_count,
                                result.snapshot_count
                            ),
                        );
                        let _ = app.emit("clipboard-items-changed", ());
                        let _ = app.emit("remote-sync-status-changed", ());
                    }
                    Ok(None) => {
                        consecutive_failures = 0;
                        last_error_message.clear();
                    }
                    Err(error) => {
                        consecutive_failures = consecutive_failures.saturating_add(1);
                        let message = error.to_string();
                        // Only log when the error changes or once every 10th
                        // repeat, so a dead provider cannot flood the log.
                        if message != last_error_message || consecutive_failures % 10 == 1 {
                            logger::error(
                                &app,
                                "auto_sync_cycle_failed",
                                format!("consecutive_failures={consecutive_failures} {message}"),
                            );
                            last_error_message = message;
                        }
                        let _ = app.emit("remote-sync-status-changed", ());
                    }
                }

                if !sleep_while_running(&thread_running, next_interval(consecutive_failures)) {
                    break;
                }
            }
        })
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))?;

    Ok(AutoSyncSchedulerShutdown { running })
}

fn next_interval(consecutive_failures: u32) -> Duration {
    if consecutive_failures == 0 {
        return CHECK_INTERVAL;
    }

    let exponent = consecutive_failures.min(5);
    let backoff = CHECK_INTERVAL.saturating_mul(2u32.saturating_pow(exponent));
    backoff.min(MAX_BACKOFF)
}

fn sleep_while_running(running: &Arc<AtomicBool>, duration: Duration) -> bool {
    let mut slept = Duration::ZERO;
    while slept < duration {
        if !running.load(Ordering::SeqCst) {
            return false;
        }
        let step = Duration::from_millis(250).min(duration - slept);
        thread::sleep(step);
        slept += step;
    }
    running.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::{next_interval, CHECK_INTERVAL, MAX_BACKOFF};
    use std::time::Duration;

    #[test]
    fn backoff_doubles_and_caps() {
        assert_eq!(next_interval(0), CHECK_INTERVAL);
        assert_eq!(next_interval(1), Duration::from_secs(120));
        assert_eq!(next_interval(2), Duration::from_secs(240));
        assert_eq!(next_interval(5), MAX_BACKOFF.min(Duration::from_secs(1920)));
        assert_eq!(next_interval(20), next_interval(5));
        assert!(next_interval(20) <= MAX_BACKOFF);
    }
}
