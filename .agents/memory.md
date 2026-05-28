# Active Memory

Last updated: 2026-05-28

## Stable Project Facts

- App name: **Gmail Desktop**, identifier: `com.madratzz.gmail-desktop`.
- Crate name: `gmail-desktop`, lib name: `gmail_desktop_lib`.
- Version: `1.0.0` (tauri.conf.json + Cargo.toml). Latest tag: `v1.0.0-alpha.1`.
- Main URL loaded: `https://mail.google.com`.
- User-agent constant: Safari 17.x on macOS — required for Google to allow Gmail to load.
- All app logic in one file: `src-tauri/src/lib.rs`.
- No frontend framework. No `index.html`. No Vite. No bundler.
- Tauri feature `unstable` is required — used for `window.add_child()` (peek overlay).

## User Preferences

- Keep responses short and concise.
- No emoji unless explicitly asked.
- No trailing summaries — user can read the diff.
- User tests manually on Windows; macOS works reliably.
- User's GitHub account: **madratzz**.
- User's email: raxashafique@gmail.com.

## Naming Conventions

- Rust functions: `snake_case`.
- Icon files: `gmail-color.png`, `gmail-dark.png`, `gmail-white.png`.
- Child webview label: `"peek"`.
- Archive filenames: `type-DD-MM-YY-slug.md`.
- Git tags: `vMAJOR.MINOR.PATCH` or `vMAJOR.MINOR.PATCH-alpha.N`.

## Important Entities

| Entity | Details |
|---|---|
| GitHub repo | https://github.com/madratzz/tauri-google-mail |
| Local path | E:\GitProjects\tauri-google-mail |
| CI workflow | .github/workflows/release.yml — triggers on `v*` tags |
| Icon source | https://dashboardicons.com/icons/external/gmail (selfhst CDN, CC BY 4.0) |
| Tauri CLI | `@tauri-apps/cli ^2.0.0` (devDependency, invoked via `npx tauri`) |
| Rust toolchain | stable-msvc on Windows |
| MSVC version | 14.44.35207 |
| Windows SDK | 10.0.26100.0 |

## Do Not Forget

- On Windows, `on_navigation` fires on a WebView2 background thread — **always** use `tauri::async_runtime::spawn` for any window op inside that callback.
- `peek.url()` is unreliable on Windows after sentinel navigation — always read URL from `PeekUrl` app state instead.
- `win.close()` on Windows may not fully release the WebView2 process — use `prevent_close()` + `win.destroy()` in `CloseRequested` handler.
- Windows builds fail without manually setting `LIB` and `INCLUDE` env vars if not running from VS Developer Command Prompt.
- `tauri icon <source.png>` regenerates all icon sizes and formats including `.ico`, `.icns`, and all Windows APPX sizes.

## Archive Summary

No archived memory yet. Initial setup 2026-05-28.

## Archive Pointers

- [Archived Memory Index](../.archive/memory/INDEX.md)
