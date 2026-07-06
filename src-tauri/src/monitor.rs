use crate::models::{ActiveApp, CompletedSession, NetworkSpeed};
use chrono::Utc;
use parking_lot::Mutex;
use std::time::Instant;
use sysinfo::{Networks, Pid, ProcessesToUpdate, System};
use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static APP_INFO_CACHE: Lazy<Mutex<HashMap<String, (String, Option<String>)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

                let site = extract_browser_site(&app, &sample.title);

                let mut app_display_name = Some(app.clone());
                let mut app_icon = None;
                if let Some(process) = inner.system.process(pid) {
                    if let Some(exe_path) = process.exe() {
                        let (display, icon) = platform::get_app_info(exe_path);
                        app_display_name = Some(display);
                        app_icon = icon;
                    }
                }
                if let Some(ref name) = app_display_name {
                    if name.to_lowercase().ends_with(".exe") {
                        app_display_name = Some(name[..name.len() - 4].to_string());
                    }
                }

                ActiveApp {
                    app,
                    title: sample.title,
                    elapsed_seconds: 0,
                    pid: Some(sample.pid),
                    sampled_at: Utc::now().timestamp(),
                    site,
                    is_fullscreen: sample.is_fullscreen,
                    app_display_name,
                    app_icon,
                }
            }
            None => ActiveApp {
                app: "Unknown".to_string(),
                title: "No foreground window detected".to_string(),
                elapsed_seconds: 0,
                pid: None,
                sampled_at: Utc::now().timestamp(),
                site: None,
                is_fullscreen: false,
                app_display_name: Some("Unknown".to_string()),
                app_icon: None,
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
            .values()
            .map(|data| data.received())
            .sum::<u64>();
        let upload_bytes = inner
            .networks
            .values()
            .map(|data| data.transmitted())
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
    is_fullscreen: bool,
}

#[cfg(target_os = "windows")]
mod platform {
    use super::ActiveWindowSample;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, HICON,
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

            let mut is_fullscreen = false;
            let mut rect = windows::Win32::Foundation::RECT::default();
            if windows::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut rect).is_ok() {
                let monitor = windows::Win32::Graphics::Gdi::MonitorFromWindow(
                    hwnd,
                    windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTOPRIMARY,
                );
                let mut mi = windows::Win32::Graphics::Gdi::MONITORINFO {
                    cbSize: std::mem::size_of::<windows::Win32::Graphics::Gdi::MONITORINFO>()
                        as u32,
                    ..Default::default()
                };
                if windows::Win32::Graphics::Gdi::GetMonitorInfoW(monitor, &mut mi).as_bool() {
                    let mut client_rect = windows::Win32::Foundation::RECT::default();
                    if windows::Win32::UI::WindowsAndMessaging::GetClientRect(
                        hwnd,
                        &mut client_rect,
                    )
                    .is_ok()
                    {
                        let mut client_point = windows::Win32::Foundation::POINT { x: 0, y: 0 };
                        if windows::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut client_point)
                            .as_bool()
                        {
                            let client_left = client_point.x;
                            let client_top = client_point.y;
                            let client_right = client_left + client_rect.right;
                            let client_bottom = client_top + client_rect.bottom;

                            if client_left <= mi.rcMonitor.left
                                && client_top <= mi.rcMonitor.top
                                && client_right >= mi.rcMonitor.right
                                && client_bottom >= mi.rcMonitor.bottom
                            {
                                is_fullscreen = true;
                            }
                        }
                    }
                }
            }

            Some(ActiveWindowSample {
                pid,
                title,
                is_fullscreen,
            })
        }
    }

    unsafe fn hicon_to_bmp_bytes(hicon: HICON) -> Option<Vec<u8>> {
        use windows::Win32::Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, SelectObject,
            BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HGDIOBJ,
        };
        use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

        let mut icon_info = ICONINFO::default();
        if GetIconInfo(hicon, &mut icon_info).is_err() {
            return None;
        }

        let hbm_color = icon_info.hbmColor;
        let hbm_mask = icon_info.hbmMask;
        
        let result = (|| {
            if hbm_color.is_invalid() {
                return None;
            }

            let hdc = CreateCompatibleDC(None);
            if hdc.is_invalid() {
                return None;
            }

            let mut bmp = BITMAP::default();
            let get_obj_res = GetObjectW(
                HGDIOBJ(hbm_color.0),
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bmp as *mut _ as *mut _),
            );
            if get_obj_res == 0 {
                let _ = DeleteDC(hdc);
                return None;
            }

            let width = bmp.bmWidth;
            let height = bmp.bmHeight;
            
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // negative height for top-down DIB
                    biPlanes: 1,
                    biBitCount: 32, // 32-bit RGBA
                    biCompression: 0, // BI_RGB
                    biSizeImage: (width * height * 4) as u32,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut buffer = vec![0u8; (width * height * 4) as usize];
            let old_obj = SelectObject(hdc, HGDIOBJ(hbm_color.0));
            
            let success = GetDIBits(
                hdc,
                hbm_color,
                0,
                height as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            SelectObject(hdc, old_obj);
            let _ = DeleteDC(hdc);

            if success == 0 {
                return None;
            }

            let file_header_size = 14;
            let info_header_size = 40;
            let total_size = file_header_size + info_header_size + buffer.len();

            let mut bmp_file = Vec::with_capacity(total_size);
            
            // 1. File Header
            bmp_file.extend_from_slice(b"BM");
            bmp_file.extend_from_slice(&(total_size as u32).to_le_bytes());
            bmp_file.extend_from_slice(&[0, 0, 0, 0]);
            bmp_file.extend_from_slice(&((file_header_size + info_header_size) as u32).to_le_bytes());

            // 2. Info Header
            bmp_file.extend_from_slice(&(info_header_size as u32).to_le_bytes());
            bmp_file.extend_from_slice(&width.to_le_bytes());
            bmp_file.extend_from_slice(&(-height).to_le_bytes());
            bmp_file.extend_from_slice(&1u16.to_le_bytes());
            bmp_file.extend_from_slice(&32u16.to_le_bytes());
            bmp_file.extend_from_slice(&0u32.to_le_bytes());
            bmp_file.extend_from_slice(&(buffer.len() as u32).to_le_bytes());
            bmp_file.extend_from_slice(&0i32.to_le_bytes());
            bmp_file.extend_from_slice(&0i32.to_le_bytes());
            bmp_file.extend_from_slice(&0u32.to_le_bytes());
            bmp_file.extend_from_slice(&0u32.to_le_bytes());

            // 3. Pixel data
            bmp_file.extend_from_slice(&buffer);

            Some(bmp_file)
        })();

        if !hbm_color.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(hbm_color.0));
        }
        if !hbm_mask.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(hbm_mask.0));
        }

        result
    }

    pub fn get_app_info(exe_path: &std::path::Path) -> (String, Option<String>) {
        use windows::core::PCWSTR;
        use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_DISPLAYNAME};
        use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
        use std::mem::size_of;
        use std::os::windows::ffi::OsStrExt;

        let exe_str = exe_path.to_string_lossy().to_string();
        let cache_key = exe_str.clone();

        if let Some(cached) = super::APP_INFO_CACHE.lock().get(&cache_key) {
            return cached.clone();
        }

        let mut path_u16: Vec<u16> = exe_path.as_os_str().encode_wide().collect();
        path_u16.push(0);

        let mut shfi = SHFILEINFOW::default();
        let result = unsafe {
            SHGetFileInfoW(
                PCWSTR(path_u16.as_ptr()),
                Default::default(),
                Some(&mut shfi as *mut _),
                size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_DISPLAYNAME,
            )
        };

        let mut friendly_name = exe_path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| exe_str.clone());

        let mut base64_icon = None;

        if result != 0 {
            let len = shfi.szDisplayName.iter().position(|&c| c == 0).unwrap_or(shfi.szDisplayName.len());
            if len > 0 {
                let name = String::from_utf16_lossy(&shfi.szDisplayName[..len]);
                if !name.trim().is_empty() {
                    friendly_name = name;
                }
            }

            if !shfi.hIcon.is_invalid() {
                if let Some(bmp_bytes) = unsafe { hicon_to_bmp_bytes(shfi.hIcon) } {
                    use base64::Engine;
                    base64_icon = Some(base64::prelude::BASE64_STANDARD.encode(&bmp_bytes));
                }
                unsafe {
                    let _ = DestroyIcon(shfi.hIcon);
                }
            }
        }

        if friendly_name.to_lowercase().ends_with(".exe") {
            friendly_name = friendly_name[..friendly_name.len() - 4].to_string();
        }

        let info = (friendly_name, base64_icon);
        super::APP_INFO_CACHE.lock().insert(cache_key, info.clone());
        info
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::ActiveWindowSample;

    pub fn active_window() -> Option<ActiveWindowSample> {
        None
    }

    pub fn get_app_info(exe_path: &std::path::Path) -> (String, Option<String>) {
        let name = exe_path.file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let friendly_name = if name.to_lowercase().ends_with(".exe") {
            name[..name.len() - 4].to_string()
        } else {
            name
        };
        (friendly_name, None)
    }
}

pub fn resolve_app_info_from_name(name: &str) -> (String, Option<String>) {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for process in sys.processes().values() {
        let exe_name = process.name().to_string_lossy().to_string();
        if exe_name.eq_ignore_ascii_case(name) {
            if let Some(exe_path) = process.exe() {
                return platform::get_app_info(exe_path);
            }
        }
    }

    // Try looking up in the cache by the executable name as a suffix
    {
        let cache = APP_INFO_CACHE.lock();
        for (path, (friendly, icon)) in cache.iter() {
            let path_lower = path.to_lowercase();
            let name_lower = name.to_lowercase();
            if path_lower == name_lower 
                || path_lower.ends_with(&format!("\\{}", name_lower)) 
                || path_lower.ends_with(&format!("/{}", name_lower)) 
            {
                return (friendly.clone(), icon.clone());
            }
        }
    }

    // Fallback: strip .exe and return no icon
    let friendly = if name.to_lowercase().ends_with(".exe") {
        name[..name.len() - 4].to_string()
    } else {
        name.to_string()
    };
    (friendly, None)
}

pub fn normalize_domain(input: &str) -> String {
    let mut cleaned = input.trim().to_lowercase();

    // Strip protocol
    if cleaned.starts_with("https://") {
        cleaned = cleaned["https://".len()..].to_string();
    } else if cleaned.starts_with("http://") {
        cleaned = cleaned["http://".len()..].to_string();
    }

    // Strip credentials and path/query/fragment
    if let Some(idx) = cleaned.find('/') {
        cleaned = cleaned[..idx].to_string();
    }

    // Strip port
    if let Some(idx) = cleaned.find(':') {
        cleaned = cleaned[..idx].to_string();
    }

    // Strip www. prefix
    if cleaned.starts_with("www.") {
        cleaned = cleaned["www.".len()..].to_string();
    }

    if cleaned.contains("youtube") {
        "youtube.com".to_string()
    } else if cleaned.contains("github") {
        "github.com".to_string()
    } else if cleaned.contains("google search") || cleaned == "google" {
        "google.com".to_string()
    } else if cleaned.contains("gmail") {
        "gmail.com".to_string()
    } else if cleaned.contains("facebook") {
        "facebook.com".to_string()
    } else if cleaned.contains("twitter") || cleaned == "x" {
        "x.com".to_string()
    } else if cleaned.contains("reddit") {
        "reddit.com".to_string()
    } else if cleaned.contains("netflix") {
        "netflix.com".to_string()
    } else if cleaned.contains("linkedin") {
        "linkedin.com".to_string()
    } else if cleaned.contains("stackoverflow") {
        "stackoverflow.com".to_string()
    } else if cleaned.contains("wikipedia") {
        "wikipedia.org".to_string()
    } else if cleaned.contains("amazon") {
        "amazon.com".to_string()
    } else {
        let filtered: String = cleaned
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
            .collect();
        if filtered.contains('.') {
            filtered
        } else if !filtered.is_empty() {
            format!("{}.com", filtered)
        } else {
            "unknown.com".to_string()
        }
    }
}

pub fn extract_browser_site(app_name: &str, window_title: &str) -> Option<String> {
    let app_lower = app_name.to_lowercase();
    let is_browser = app_lower.contains("chrome")
        || app_lower.contains("firefox")
        || app_lower.contains("msedge")
        || app_lower.contains("brave")
        || app_lower.contains("opera")
        || app_lower.contains("iexplore")
        || app_lower.contains("safari");

    if !is_browser {
        return None;
    }

    let mut title = window_title.trim().to_string();
    let browser_suffixes = [
        " - Google Chrome",
        " - Microsoft Edge",
        " - Mozilla Firefox",
        " - Brave",
        " - Opera",
        " - Google Chrome (Incognito)",
        " - Brave (Private)",
    ];
    for suffix in browser_suffixes {
        if title.ends_with(suffix) {
            title = title[..title.len() - suffix.len()].trim().to_string();
            break;
        }
    }

    let separators = [" - ", " | ", " · "];
    let mut site_name = title.clone();
    for sep in separators {
        if let Some(last_part) = title.rsplit(sep).next() {
            let trimmed = last_part.trim();
            if !trimmed.is_empty() && trimmed.len() < 30 {
                site_name = trimmed.to_string();
                break;
            }
        }
    }

    Some(normalize_domain(&site_name))
}

pub fn get_app_info(exe_path: &std::path::Path) -> (String, Option<String>) {
    platform::get_app_info(exe_path)
}
