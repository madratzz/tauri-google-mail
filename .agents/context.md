# Active Project Context

Last updated: 2026-05-28

## Project Summary

**Gmail Desktop** is a Tauri 2 cross-platform desktop wrapper for Gmail (mail.google.com). It loads Gmail directly inside a system webview (WebView2 on Windows, WKWebView on macOS) with no JavaScript frontend framework. The entire app logic lives in a single Rust file: `src-tauri/src/lib.rs`.

GitHub: https://github.com/madratzz/tauri-google-mail  
Account: madratzz

## Current Goals

- Maintain and ship stable cross-platform Gmail desktop app.
- Alpha release v1.0.0-alpha.1 just tagged and pushed — GitHub Actions CI building for macOS/Windows/Linux.
- Continue iterating on UX: icon variants, peek overlay, pop-out stability.

## Current Architecture / Structure

```
tauri-google-mail/
  src-tauri/
    src/
      lib.rs          ← entire app logic (Rust)
      main.rs         ← calls gmail_desktop_lib::run()
    icons/
      icon.png / icon.ico / icon.icns   ← main app icon (Gmail color)
      gmail-color.png                   ← icon switcher: color variant
      gmail-dark.png                    ← icon switcher: dark variant
      gmail-white.png                   ← icon switcher: light/white variant
    Cargo.toml
    tauri.conf.json
  .github/
    workflows/
      release.yml     ← builds macOS/Windows/Linux on v* tags
  package.json        ← tauri CLI dev dependency only
  .agents/            ← agent context system (this folder)
  .archive/           ← agent context archives
```

**Key Rust structures:**
- `PeekUrl(Mutex<Option<tauri::Url>>)` — app state storing the real URL before peek webview creation, avoids sentinel-URL race on Windows/WebView2.
- `create_peek_overlay()` — spawns a child webview at 85% window size centered.
- `expand_peek()` — reads URL from `PeekUrl` state, closes peek, opens standalone window.
- `close_peek()` — closes the peek child webview.
- `open_standalone_window()` — creates an independent WebviewWindow, handles recursive new-window interception.

**Sentinel URL pattern:** Peek toolbar buttons navigate to `peek-action.tauri.internal/expand` and `/close`. `on_navigation` catches these and spawns async tasks to act on them.

## Important Decisions

- No JS frontend — Gmail URL loaded directly in webview. Keeps the app minimal and avoids build complexity.
- Safari user-agent — required to pass Google's browser compatibility check for Gmail.
- `tauri::async_runtime::spawn` wrapping all window ops inside `on_navigation` — fixes Windows/WebView2 deadlock where navigation callbacks fire on a background thread.
- `PeekUrl` state — fixes Windows race condition where `peek.url()` returns the sentinel URL by the time `expand_peek` reads it.
- `win.destroy()` via `CloseRequested` + `prevent_close()` — forces WebView2 to release properly; plain `win.close()` was unreliable.
- Icons sourced from dashboardicons.com (selfhst/icons CDN), CC BY 4.0.

## Active Constraints

- Tauri 2 API only — no Tauri v1 patterns.
- Windows builds require MSVC toolchain with specific env vars (`LIB`, `INCLUDE`, `PATH`) pointing to MSVC 14.44.35207 and Windows SDK 10.0.26100.0.
- `crate-type = ["staticlib", "cdylib", "rlib"]` required in Cargo.toml for Tauri 2 lib builds.
- `unstable` feature flag required in Tauri for `add_child()` (peek overlay).
- Do not add a JS frontend or build tool — keep it pure Rust + Tauri.
- Do not commit `.claude/` directory (in .gitignore).

## Current Open Questions

- Should the peek overlay support multiple simultaneous overlays, or always replace the existing one?
- Consider adding a notification badge / unread count in the dock/taskbar.

## Archive Summary

No archived context yet. This is the initial setup on 2026-05-28.

## Archive Pointers

- [Archived Context Index](../.archive/context/INDEX.md)
