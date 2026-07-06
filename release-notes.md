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
