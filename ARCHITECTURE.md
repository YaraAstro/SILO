# SILO Architecture

SILO is a local-first desktop app built with Tauri, Svelte, Rust, and SQLite.

## Runtime Shape

```text
Svelte UI
  -> Tauri IPC commands and events
Rust backend
  -> storage, rules, monitoring, metering, blocker, export services
SQLite
  -> local settings, rules, sessions, usage, samples, logs
Windows integration
  -> foreground window APIs, network counters, notifications, optional elevated actions
```

## Current Foundation

- `src/routes/+page.svelte` provides the initial app shell and core screens.
- `src/lib/siloApi.ts` centralizes typed frontend calls to Tauri commands.
- `src-tauri/src/api.rs` exposes the IPC command surface.
- `src-tauri/src/monitor.rs` samples the Windows foreground window, resolves the process name, emits active-app updates, and closes sessions when focus changes.
- `src-tauri/src/storage.rs` owns SQLite setup, migrations, settings, rules, usage queries, and exports.
- `src-tauri/schema/001_initial.sql` is the first migration and matches `DB_SCHEMA.sql`.

## Implementation Order

1. Foundation, IPC, SQLite, settings, and rules CRUD.
2. Active window monitoring and session persistence.
3. Dashboard aggregates and usage charts.
4. Rules evaluation and focus mode lifecycle.
5. App blocking, then website blocking with explicit privilege handling.
6. Live network speed, per-app attribution, and per-site attribution.
7. Retention, exports, backup prompts, tray controls, logs, tests, and release packaging.
