# SILO Build And Release

## Local Production Build

```bash
pnpm tauri build
```

## Release Checklist

- Run `pnpm check` and `pnpm build`.
- Run `cargo fmt`, `cargo test`, and `cargo clippy` from `src-tauri`.
- Verify install and uninstall behavior on Windows.
- Sign Windows binaries before public distribution.
- Document any privileged actions before enabling hosts or firewall blocking.

## Future CI Jobs

- Frontend install, check, and build.
- Rust format, clippy, and tests.
- Tauri bundle build on Windows runners.
- Sandbox integration tests for blocking and export flows.
