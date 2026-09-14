# Active Memory

Last updated: 2026-09-14

## Stable Project Facts

- Product name: Gmail Desktop. Bundle identifier: `com.madratzz.gmail-desktop`.
- Rust package: `gmail-desktop`; library: `gmail_desktop_lib`.
- Current manifest version: `1.1.29823021` in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` (assigned 2026-09-14T10:21:14Z for the backward-compatible icon update).
- Gmail is loaded directly from `https://mail.google.com/` with a Safari user agent.
- Main application behavior is in `src-tauri/src/lib.rs`; no frontend framework, HTML entry point, or bundler is present.
- The release workflow runs on tags matching `v*` and builds macOS, Windows, Ubuntu/Debian, and Arch artifacts.

## User Preferences

- No active user preferences have been independently verified in this session.

## Naming Conventions

- Rust functions use `snake_case`.
- Gmail menu-icon assets use `gmail-color.png`, `gmail-dark.png`, and `gmail-white.png`.
- Archive filenames use `type-YYYY-MM-DD[-short-gist][-NN].md`.
- The repository policy defines release versions as `X.Y.Z`, with epoch minutes as `Z`; confirm any transition from existing versioning before publishing.

## Important Entities

| Entity | Details |
|---|---|
| Tauri runtime | Tauri 2 with `image-png` and `unstable` features |
| Main URL | `https://mail.google.com/` |
| Release workflow | `.github/workflows/release.yml` |
| Windows behavior guide | `docs/windows-popup-bugs.md` |
| macOS signing guide | `docs/macos-damaged-release.md` |

## Do Not Forget

- In Windows WebView2 navigation callbacks, use `tauri::async_runtime::spawn` before doing window operations.
- Read the stored `PeekUrl` rather than querying `peek.url()` after sentinel navigation.
- Preserve the established forced `destroy()` close behavior for standalone Windows windows unless it is replaced with targeted testing.
- Keep `.agents/` and `.archive/` in version control; do not add them to `.gitignore`.

## Archive Summary

The original bootstrap memory is preserved in a redacted 2026-05-28 snapshot. It included machine-specific environment guidance that is intentionally no longer active.

## Archive Pointers

- [Archived Memory Index](../.archive/memory/INDEX.md)
