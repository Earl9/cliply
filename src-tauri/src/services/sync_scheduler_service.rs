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

            while thread_running.load(Ordering::SeqCst) {
                match remote_sync_service::run_auto_sync_cycle(&app) {
                    Ok(Some(result)) => {
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
                    Ok(None) => {}
                    Err(error) => {
                        logger::error(&app, "auto_sync_cycle_failed", error);
                        let _ = app.emit("remote-sync-status-changed", ());
                    }
                }

                if !sleep_while_running(&thread_running, CHECK_INTERVAL) {
                    break;
                }
            }
        })
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))?;

    Ok(AutoSyncSchedulerShutdown { running })
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
