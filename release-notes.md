# SILO Pre-release v0.2.0-preview

Welcome to **SILO v0.2.0-preview**! This release marks a major milestone, introducing system tray integration for background monitoring and deep-dive network metrics dashboards. 

Below is a summary of the new features, user experience updates, and configuration details for this pre-release.

---

## 🚀 Key Highlights

### 1. System Tray & Seamless Background Processing
SILO now runs as a persistent background service to continuously monitor activity and enforce rules without occupying space on your taskbar.
* **Hide-on-Close Behavior**: Clicking the close window button (`[X]`) now hides the window to the system tray instead of closing the application.
* **Left-click Toggle**: Click the system tray icon to quickly toggle the window between visible/focused and hidden states.
* **Right-click Context Menu**:
  * **Show SILO**: Restores and focuses the main window.
  * **Add Rule**: Instantly focuses the window and navigates directly to the **Rules & Limits** section.
  * **Start / Stop Focus Mode**: A context-aware toggle allowing you to activate/deactivate Focus Mode directly from the tray.
  * **Quit**: Fully exits the application and shuts down background tasks.
* **Bidirectional Sync**: Real-time listeners automatically update the tray menu item label (e.g., changing from "Start Focus Mode" to "Stop Focus Mode") when toggled from the in-app dashboard.

### 2. Advanced Network Analytics & Detail Sub-screens
To prevent dashboard clutter, the main network tab has been optimized, and new dedicated analytics views have been added.
* **5-Item Widget Limits**: The "Top Apps" and "Top Sites" widgets on the Network tab now display the top 5 consumers, keeping the main dashboard clean.
* **Dedicated Analytics Screens**: Interactive, full-page sub-screens are now accessible via **More** buttons for both top apps and top sites.
* **Multi-period Time Ranges**: View bandwidth metrics filtered by **Today** (1-day query), **Last 7 Days**, or **Last 30 Days**.
* **Bandwidth Metrics Grid**: Comprehensive overview cards showing *Total*, *Download*, and *Upload* data usage for the selected period.
* **Visual Usage Gradients**: Rich progress bars with color gradients representing the percentage share of each item's bandwidth consumption relative to the top consumer.
* **Real-time Filter**: An instant search input to quickly isolate specific applications or website domains.

---

## 🛠️ Code Changes & Affected Files

The implementation spans both the Rust/Tauri backend and the Svelte frontend:

| Component | File Path | Description |
| :--- | :--- | :--- |
| **Backend Config** | [`Cargo.toml`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/Cargo.toml) | Enabled `tray-icon` and `image-png` features for Tauri. |
| **Backend Core** | [`src/lib.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/lib.rs) | Created system tray icon, set up context menu handlers, registered close-to-tray window events, and synced focus mode events. |
| **Storage Layer** | [`src/storage.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/storage.rs) | Updated SQLite data queries to accurately map "Today" to current-day metrics. |
| **Frontend UI** | [`src/routes/+page.svelte`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/routes/+page.svelte) | Designed Svelte state handlers, sub-screen layouts, search filter logics, and range toggle APIs. Added `"navigate"` listener to handle tray shortcuts. |

---

## 📝 How to Test the Pre-release

To run and verify the new features in your local environment, follow these steps:

1. **Launch the Development Server**:
   ```bash
   pnpm tauri dev
   ```
2. **Verify System Tray Backgrounding**:
   * Click the close (`[X]`) button. The window should hide.
   * Verify that the SILO icon is active in your Windows System Tray.
   * Left-click the tray icon to restore the window.
3. **Verify Context Shortcuts**:
   * Right-click the tray icon and select **Add Rule**. The app window should restore and show the **Rules** tab.
   * Right-click the tray icon and select **Start Focus Mode**. Toggle the window open and verify that the Focus button on the dashboard updates accordingly.
4. **Inspect Network Sub-screens**:
   * Open the **Network** dashboard and select **More Top Apps** or **More Top Sites**.
   * Test the search input and date filter tabs (**Today**, **7 Days**, **30 Days**) to verify SQLite database query integration.
