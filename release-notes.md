# SILO Release Notes v1.1.2 (Window Resizing & Real-Time Network Monitor)

Welcome to **SILO v1.1.2**! This release fixes window resizing/expansion freezes and delivers a silky-smooth, real-time live network monitor with industry-standard Mbps & Kbps metrics.

---

## 🚀 Key Highlights

### 1. Window Expansion & Resizing Freeze Fix
* **Eliminated Layout Thrashing**: Resolved an issue where expanding or resizing the application window caused Chart.js to enter an infinite resize observation loop within flex/grid layouts.
* **Bounded Containers & Debounced Redraws**: Chart canvases are now safely contained with `overflow: hidden`, and dataset updates happen in-place using `chart.update('none')` with a 60ms debounce.

### 2. Smooth 1-Second Real-Time Network Stream with EMA
* **Real-time Event Streaming**: The monitoring engine now streams live network transfer rates to the frontend every 1 second over Tauri IPC (`network_speed_update`).
* **Exponential Moving Average (EMA)**: Implemented an EMA smoothing algorithm to filter out bursty UDP/TCP packet jitter, providing a smooth real-time bandwidth graph.

### 3. Standard Mbps & Kbps Bandwidth Formatting
* **Industry Standard Units**: Network rates are now converted and formatted into standard bits-per-second units (**Mbps**, **Kbps**, **Gbps**) matching Ookla Speedtest and network monitoring tools, while total cumulative volume remains in **GB / MB / KB**.

---

# SILO Release Notes v1.1.1 (Windows Startup & Background Execution Refactor)

Welcome to **SILO v1.1.1**! This release introduces a refactored Windows startup and window lifecycle architecture, ensuring completely headless background execution during Windows boot, on-demand GUI instantiation, single-instance communication, and responsive window handling.

---

## 🚀 Key Highlights

### 1. Headless Background Startup on Windows Boot
* **Zero UI at Boot**: When Windows boots or when started with background flags (`--minimized`, `--background`, `--autostart`, `--silent`, `-b`), SILO runs entirely in the background without creating a visible window, splash screen, or initializing the Edge WebView2 runtime.
* **Minimal Startup Footprint**: Core background tasks (active app tracking, network speed monitoring, SQLite database, and system tray management) initialize in microseconds with zero CPU overhead and negligible memory usage.

### 2. On-Demand GUI Loading & Single-Instance IPC
* **Dynamic Window Creation**: The main WebView window and dashboard UI are created only when explicitly requested by the user.
* **Single-Instance Re-use**: Manually launching SILO from a desktop shortcut or Start menu while the background instance is active detects the running process via IPC, forwards CLI arguments, and immediately reveals the GUI without spawning duplicate processes.
* **Instant Window Restoration**: Closing the window hides it smoothly to keep background monitoring active, and subsequent reopenings from the shortcut or system tray are instantaneous.

### 3. Responsive Closing & Clean Termination
* **Eliminated "Not Responding" Dialogs**: Separating heavy frontend and SQLite initialization from startup eliminates window message pump starvation when users close the window during initial launch.
* **Clean Process Exit**: Selecting "Quit" from the tray menu or PIN modal terminates the background process completely.

---

## 🛠️ Code Changes & Affected Files

| Component | File Path | Description |
| :--- | :--- | :--- |
| **Workspace Manifest** | [`package.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/package.json) | Bumped version to `1.1.1`. |
| **Backend Manifest** | [`Cargo.toml`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/Cargo.toml) | Bumped package version to `1.1.1`. |
| **Tauri Core Configuration** | [`tauri.conf.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/tauri.conf.json) | Bumped configuration build version to `1.1.1` and removed static window creation (`"windows": []`). |
| **Rust Core & Lifecycle** | [`src-tauri/src/lib.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/lib.rs) | Implemented `show_or_create_main_window`, CLI background flag detection, on-demand single-instance/tray callbacks, and clean toast string formatting. |

---

## 📝 How to Verify the Release

1. **Verify Backend Build and Manifest Sync**:
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
2. **Build Static Frontend Assets**:
   ```bash
   pnpm build
   ```
3. **Test Headless Startup & GUI on Demand**:
   * Run `cargo run --manifest-path src-tauri/Cargo.toml -- --minimized` (or launch via autostart).
   * Confirm process runs quietly in background with tray icon and NO visible window.
   * Double-click application shortcut or click tray icon to verify GUI opens promptly.
   * Close window and verify process remains alive in tray.
   * Click "Quit" from tray menu and confirm process terminates completely.

---

# SILO Release Notes v1.1.0 (PIN-Authorized Settings Protection)

Welcome to **SILO v1.1.0**! This release introduces critical settings security, requiring PIN authorization before modifying configurations.

---

## 🚀 Key Highlights

### 1. PIN-Authorized Settings Protection
We have added a security guard to the Settings panel:
* **Authorization Dialog**: When clicking "Save Settings", a new high-fidelity authentication modal is presented.
* **PIN Validation**: You must enter today's local date-based PIN (`ddmmyy`) to authorize and write configurations to the SQLite database.
* **Improved UX**: Prevents unauthorized modifications by third parties or accidental edits during focus sessions.

---

## 🛠️ Code Changes & Affected Files

| Component | File Path | Description |
| :--- | :--- | :--- |
| **Workspace Manifests** | [`package.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/package.json) | Bumped version to `1.1.0`. |
| **Backend Manifests** | [`Cargo.toml`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/Cargo.toml) | Bumped package version to `1.1.0`. |
| **Tauri Core Configuration** | [`tauri.conf.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/tauri.conf.json) | Bumped configuration build version to `1.1.0`. |
| **Settings view** | [`src/lib/components/views/SettingsView.svelte`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/lib/components/views/SettingsView.svelte) | Implemented PIN save overlay check, imports, state variables, and callbacks. |

---

## 📝 How to Verify the Release

1. **Verify Cargo Build and Package Sync**:
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
2. **Launch Application in Development**:
   ```bash
   pnpm tauri dev
   ```
3. **Test PIN Save Settings**:
   * Navigate to the **Settings** view.
   * Modify any option (e.g. toggle "Notifications" or "Keyboard Shortcuts").
   * Click **Save Settings**.
   * Verify that the **Authorize Settings Update** PIN modal overlays the screen.
   * Attempt to cancel or enter an incorrect PIN; confirm changes are not saved and error toast displays.
   * Enter today's PIN (in `ddmmyy` format) and confirm settings are successfully saved with a success toast.

---

# SILO Release Notes v1.0.0 (Official Release)

Welcome to the **initial official release of SILO (v1.0.0)**! SILO is a local-first, privacy-focused productivity and digital wellness platform designed to help you monitor screen time, control distractions, and analyze internet usage in real-time.

This release represents our graduation to a stable production-ready state, introducing critical security enhancements, file export versatility, dynamic configurations, and extensive test coverage.

---

## 🚀 Key Highlights

### 1. PIN-Gated Tray Exit Security
To guarantee uninterrupted productivity monitoring and prevent accidental or unauthorized closure of the SILO agent, exiting the application from the tray menu is now secure:
* **Tray Quit Restriction**: Attempting to "Quit" from the system tray menu now pops up a sleek, high-fidelity security lock overlay.
* **Authentication Challenge**: Users must enter today's local date-based PIN (`ddmmyy`) to finalize the shutdown process.
* **Refined Cancelling UX**: Cancelling the verification safely hides the main window back to the tray, maintaining active tracking and focus enforcement.

### 2. Native Custom Export Directory Selection
We have transitioned from hardcoded export paths to custom user-chosen locations:
* **Native Directory Dialogs**: Exporting usage reports or network logs now triggers the system-native save dialog.
* **Direct Choice**: Users can select exactly where to write CSV or JSON reports, improving backup management and usability.

### 3. Dynamic App Versioning
* Hardcoded version strings inside the settings and about interfaces have been completely eliminated.
* The frontend now dynamically retrieves the current version string directly from the Tauri back-end runtime, ensuring visual accuracy across updates.

### 4. Enterprise-Grade Automated Test Suite
* Added complete integration and unit tests covering Svelte view navigation, database connections, statistics processing, and focus mode state transitions to ensure stable execution.

---

## 🛠️ Code Changes & Affected Files

| Component | File Path | Description |
| :--- | :--- | :--- |
| **Workspace Manifests** | [`package.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/package.json) | Bumped version to `1.0.0`. |
| **Backend Manifests** | [`Cargo.toml`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/Cargo.toml) | Bumped package version to `1.0.0`. |
| **Tauri Core Configuration** | [`tauri.conf.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/tauri.conf.json) | Bumped configuration build version to `1.0.0`. |
| **Tauri Command Layer** | [`src-tauri/src/api.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/api.rs)<br>[`src-tauri/src/lib.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/lib.rs) | Created the `exit_app` command, updated API registrations, and modified system tray event handling. |
| **API Client Typings** | [`src/lib/siloApi.ts`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/lib/siloApi.ts) | Implemented client invoker bindings for `exitApp`. |
| **Main View Layout** | [`src/routes/+page.svelte`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/routes/+page.svelte) | Implemented modal routing for quitting application and state listeners. |
| **Settings view** | [`src/lib/components/views/SettingsView.svelte`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/lib/components/views/SettingsView.svelte) | Changed static version displays to bind dynamically to `boot.version`. |

---

## 📝 How to Verify the Release

1. **Verify Cargo Build and Package Sync**:
   Verify everything compiles cleanly:
   ```bash
   cargo check --manifest-path src-tauri/Cargo.toml
   ```
2. **Launch Application in Development**:
   ```bash
   pnpm tauri dev
   ```
3. **Test PIN Gated Exit**:
   * Right-click the system tray icon and select **Quit**.
   * Verify the "Quit Application" PIN prompt modal appears.
   * Attempt to cancel or enter an incorrect PIN; confirm the window hides/denies exit.
   * Enter today's PIN (in `ddmmyy` format) and confirm the application cleanly shuts down.
4. **Confirm Dynamic Version Badge**:
   * Navigate to the **Settings** view and check the **Updates** and **About** panels.
   * Confirm they display the dynamic `v1.0.0` tag.
