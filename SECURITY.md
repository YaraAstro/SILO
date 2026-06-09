# SILO Security Notes

SILO stores user activity data locally by default. The current foundation does not perform privileged system changes.

## Local Data

- SQLite data is stored in the user's local application data directory.
- Exports are written locally under the SILO app data directory.
- No cloud sync is enabled by default.

## Privileged Operations

Website blocking and hard app enforcement are later phases. When implemented:

- Request elevation only for the specific action that needs it.
- Keep the main app unprivileged.
- Back up the hosts file before writing entries.
- Provide a restore path for every hosts or firewall change.
- Protect critical system processes from hard blocking.

## Privacy Boundary

Do not inspect TLS content or capture page contents. Per-site usage should rely on domain-level metadata or explicit proxy/filter opt-in with clear consent.
