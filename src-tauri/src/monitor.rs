use crate::models::{ActiveApp, CompletedSession, NetworkSpeed};
use chrono::Utc;
use parking_lot::Mutex;
use std::time::Instant;
use sysinfo::{Networks, Pid, ProcessesToUpdate, System};

pub struct Monitor {
    inner: Mutex<MonitorState>,
}

struct MonitorState {
    system: System,
    current: ActiveApp,
    active_since: Instant,
    active_start_ts: i64,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MonitorState {
                system: System::new(),
                current: ActiveApp::default(),
                active_since: Instant::now(),
                active_start_ts: Utc::now().timestamp(),
            }),
        }
    }

    pub fn sample_active_app_with_session(&self) -> (ActiveApp, Option<CompletedSession>) {
        let sample = platform::active_window();
        let mut inner = self.inner.lock();

        let mut next = match sample {
            Some(sample) => {
                let pid = Pid::from_u32(sample.pid);
                inner
                    .system
                    .refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
                let app = inner
                    .system
                    .process(pid)
                    .map(|process| process.name().to_string_lossy().to_string())
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or_else(|| format!("pid:{}", sample.pid));

                ActiveApp {
                    app,
                    title: sample.title,
                    elapsed_seconds: 0,
                    pid: Some(sample.pid),
                    sampled_at: Utc::now().timestamp(),
                }
            }
            None => ActiveApp {
                app: "Unknown".to_string(),
                title: "No foreground window detected".to_string(),
                elapsed_seconds: 0,
                pid: None,
                sampled_at: Utc::now().timestamp(),
            },
        };

        let changed = inner.current.pid != next.pid || inner.current.title != next.title;
        let completed_session = if changed {
            let end_ts = Utc::now().timestamp();
            let duration_seconds = (end_ts - inner.active_start_ts).max(0);
            let completed = if inner.current.pid.is_some() && duration_seconds > 0 {
                Some(CompletedSession {
                    app_name: inner.current.app.clone(),
                    window_title: inner.current.title.clone(),
                    start_ts: inner.active_start_ts,
                    end_ts,
                    duration_seconds,
                })
            } else {
                None
            };
            inner.active_since = Instant::now();
            inner.active_start_ts = end_ts;
            completed
        } else {
            next.elapsed_seconds = inner.active_since.elapsed().as_secs() as i64;
            None
        };

        inner.current = next.clone();
        (next, completed_session)
    }
}

pub struct NetworkMonitor {
    inner: Mutex<NetworkMonitorState>,
}

struct NetworkMonitorState {
    networks: Networks,
    last_sample_at: Instant,
    last_speed: NetworkSpeed,
}

impl NetworkMonitor {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(NetworkMonitorState {
                networks: Networks::new_with_refreshed_list(),
                last_sample_at: Instant::now(),
                last_speed: NetworkSpeed::default(),
            }),
        }
    }

    pub fn sample_speed(&self) -> NetworkSpeed {
        let mut inner = self.inner.lock();
        let elapsed_seconds = inner.last_sample_at.elapsed().as_secs_f64();
        if elapsed_seconds < 0.25 {
            return inner.last_speed.clone();
        }

        inner.networks.refresh(true);

        let download_bytes = inner
            .networks
            .iter()
            .map(|(_, data)| data.received())
            .sum::<u64>();
        let upload_bytes = inner
            .networks
            .iter()
            .map(|(_, data)| data.transmitted())
            .sum::<u64>();

        let speed = NetworkSpeed {
            upload_bps: bytes_per_second(upload_bytes, elapsed_seconds),
            download_bps: bytes_per_second(download_bytes, elapsed_seconds),
        };

        inner.last_sample_at = Instant::now();
        inner.last_speed = speed.clone();
        speed
    }
}

fn bytes_per_second(bytes: u64, elapsed_seconds: f64) -> i64 {
    if elapsed_seconds <= 0.0 || !elapsed_seconds.is_finite() {
        return 0;
    }

    ((bytes as f64 / elapsed_seconds).round()).clamp(0.0, i64::MAX as f64) as i64
}

struct ActiveWindowSample {
    pid: u32,
    title: String,
}

#[cfg(target_os = "windows")]
mod platform {
    use super::ActiveWindowSample;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    pub fn active_window() -> Option<ActiveWindowSample> {
        // Win32 foreground-window calls are small and synchronous; keep the unsafe boundary here.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return None;
            }

            let mut buffer = vec![0u16; 512];
            let len = GetWindowTextW(hwnd, &mut buffer);
            let title = if len > 0 {
                String::from_utf16_lossy(&buffer[..len as usize])
            } else {
                "Untitled window".to_string()
            };

            Some(ActiveWindowSample { pid, title })
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::ActiveWindowSample;

    pub fn active_window() -> Option<ActiveWindowSample> {
        None
    }
}
