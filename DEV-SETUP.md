# SILO Development Setup

## Requirements

- Rust stable
- Node.js 18 or newer
- pnpm
- Tauri Windows prerequisites, including WebView2 Runtime and Visual Studio C++ build tools

## Install

```bash
pnpm install
```

## Run The Frontend

```bash
pnpm dev
```

This starts the Svelte dev server only. Backend commands require Tauri.

## Run The Desktop App

```bash
pnpm tauri dev
```

The app creates a local SQLite database under the user's local application data directory in `SILO/silo.sqlite`.

## Backend Checks

```bash
cd src-tauri
cargo fmt
cargo test
```

## Frontend Checks

```bash
pnpm check
pnpm build
```
