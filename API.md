# SILO IPC API

Frontend code calls these commands through `src/lib/siloApi.ts`.

## Boot And State

- `handshake()` returns app version, database status, focus state, and settings.
- `get_app_state()` returns focus state, the current active app, today's tracked seconds, rule counts, and network speed placeholder.

Events:

- `app_ready` with backend initialization status.
- `update_active_app` with the current foreground app, title, PID, sample time, and elapsed foreground seconds.
- `usage_update` with the current app and today's tracked seconds.

## Focus Mode

- `start_focus_mode()`
- `stop_focus_mode()`
- `toggle_focus_mode()` returns the new enabled state.

Events:

- `focus_mode_changed` with `{ enabled }`.

## Rules

- `get_rules()` returns all app and site rules.
- `save_rule(rule)` creates or updates a rule.
- `delete_rule(id)` removes a rule.

Events:

- `rules_changed` with the saved rule or deleted rule id.

## Usage And Network

- `get_usage(date)` returns app usage totals for an ISO date.
- `get_usage_90d()` returns daily totals for the last 90 days.
- `get_network_speed()` returns `{ uploadBps, downloadBps }`.
- `get_data_usage(range)` accepts `7d`, `30d`, or `90d`.

Active-window monitoring and session writes are implemented for Windows. Network speed and data attribution currently expose stable command contracts; OS network sampling is a later implementation phase.

## Settings And Backup

- `get_settings()`
- `save_settings(settings)`
- `mark_backup_complete()`

Events:

- `settings_changed` with saved settings.

## Exports

- `export_usage_data(range)` writes a JSON export into the local SILO data directory.
- `export_logs(range)` writes a scaffolded JSON log export into the local SILO data directory.
