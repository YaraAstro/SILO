# SILO Release Notes v0.4.0

Welcome to **SILO v0.4.0**! This major release delivers a premium dashboard UI/UX overhaul, deep range-based screen time and network analytics, list pagination, and a global keyboard shortcut system for fluid navigation.

Below is a summary of the new features, visual improvements, and configuration details for this release.

---

## 🚀 Key Highlights

### 1. Immersive Dashboard UI/UX Redesign
The primary dashboard has been redesigned to feel highly interactive, responsive, and visual:
* **Time-of-day Greetings & Clock Header**: Dynamically greets you (e.g., "Good Morning", "Good Evening") based on the system hour, featuring a formatted local date and version sub-badge.
* **Reactive Focus Shield state**: The header glassmorphism border and background glow transition smoothly between teal/purple (monitoring suspended) and rose/red (monitoring active) with a pulsing power toggle button.
* **Dynamic Live Focus Score SVG**: Shows your live focus score inside a circular SVG ring driven by a color-coded gradient (Teal for Optimal, Blue for Good, Amber for Moderate, Rose for Distracted) accompanied by a neon glow filter.
* **Speed Shimmer Indicators**: Shimmering flow-lines track network download/upload speeds in real time, animating active traffic.
* **Visual Progress Bars**: App and website lists on the dashboard now embed progress meters indicating the remaining focus budget (green to red) or percentage share of total daily screen time.

### 2. Deep Usage & Screen Time Insights (Stats Tab)
The **Stats** view has been upgraded into a comprehensive usage hub:
* **Interactive Range Queries**: Query screen time and network volumes grouped by application or site domain over multiple time ranges (**Today**, **7 Days**, **30 Days**, **90 Days**).
* **Productivity Patterns & Habits**: Features focus streak counters, average focused hours, best-tracked days, and dual visual heatmaps tracking historical screen time and network volume patterns.
* **Quick Rule Setup**: Click the `+` action button on any highly-used app or site card to immediately slide open the rule limit drawer with target names pre-filled.

### 3. Global In-App Keyboard Shortcuts
You can now navigate and control SILO entirely via keyboard hotkeys:
* **Hotkey Navigation mappings**: 
  * `D` or `1`: Go to **Dashboard**
  * `R` or `2`: Go to **Rules**
  * `S` or `3`: Go to **Stats**
  * `N` or `4`: Go to **Network**
  * `P` or `5` or `,`: Go to **Settings**
* **Quick Actions**: 
  * `Spacebar`: Toggle Focus Shield active state
  * `Escape`: Instantly exit drawers, overlay menus, or remove text input focus.
* **Intelligent Focus Detection**: Smart filters ignore navigation inputs while typing inside rule forms, searches, or other input controls.
* **Keyboard Settings Toggle**: Enable or disable shortcuts via Settings, featuring a keycap visual dashboard panel summarizing active hotkeys.

### 4. Smart List Pagination
To prevent scrolling lag and optimize visual structure, pagination controls have been added to:
* **Focus Breakdown** (Stats tab)
* **Attributed Traffic** (Stats tab)
* Page size is capped at **5 rows** per page with dynamic total page indicators and disabled control boundaries.

---

## 🛠️ Code Changes & Affected Files

| Component | File Path | Description |
| :--- | :--- | :--- |
| **Workspace Manifests** | [`package.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/package.json) | Bumped package version to `0.4.0`. |
| **Backend Manifests** | [`Cargo.toml`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/Cargo.toml) | Bumped package version to `0.4.0`. |
| **Tauri Core Configurations**| [`tauri.conf.json`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/tauri.conf.json) | Bumped application build version to `0.4.0`. |
| **Storage layer** | [`src-tauri/src/storage.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/storage.rs) | Created `usage_range_report` to query and sum range stats; added `shortcuts_enabled` columns database persistence. |
| **Tauri Handlers** | [`src-tauri/src/api.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/api.rs)<br>[`src-tauri/src/lib.rs`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src-tauri/src/lib.rs) | Exposed the `get_usage_range` handler command to frontend clients. |
| **API Typings** | [`src/lib/siloApi.ts`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/lib/siloApi.ts) | Implemented client invoker wrappers for `getUsageRange` and shortcuts settings attributes. |
| **Global Typography Styles** | [`src/app.css`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/app.css) | Embedded `animate-shimmer`, `animate-radar`, and `animate-pulse-glow` micro-interaction classes. |
| **Dashboard & Layout** | [`src/routes/+page.svelte`](file:///d:/Devs/project_silo/attempt_VI/silo/silo/src/routes/+page.svelte) | Implemented Svelte 5 derived state logic, global key handlers, pagination systems, custom speedometer SVG graphics, time greetings, and stats sub-screens. |

---

## 📝 How to Test the Release

1. **Start development app**:
   ```bash
   pnpm tauri dev
   ```
2. **Test Dashboard UI/UX**:
   * Verify time greetings change based on time (e.g., Good Morning/Afternoon/Evening).
   * Toggle Focus Mode and observe colors changing between teal (suspended) and pulsing red (active).
   * Verify the circular focus meter updates dynamically relative to remaining rule limits.
3. **Verify Keyboard Navigation**:
   * Navigate between tabs using keys `1`, `2`, `3`, `4`, `5`.
   * Open the Rule Form drawer and ensure pressing `1`, `2`, etc., inputs numbers without triggering navigation shortcuts.
   * Press `Escape` to close drawers.
4. **Inspect Stats range updates**:
   * Open **Stats** and verify that pagination controls (Page size = 5) operate correctly.
   * Select different range bounds (**Today**, **7 Days**, **30 Days**, **90 Days**) and verify total focus metrics update correctly.
