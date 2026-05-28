# Active Learnings

Last updated: 2026-05-28

## Recent Learnings

### Windows/WebView2 threading model
- `on_navigation` callbacks fire on a **background thread** in WebView2, not the main UI thread.
- Any Tauri window operation (`close()`, `destroy()`, `WebviewWindowBuilder::new()`) attempted directly inside `on_navigation` will **deadlock** on Windows — those ops dispatch to the main thread, but the main thread is blocked waiting for the navigation callback to return.
- Fix: wrap all window ops in `tauri::async_runtime::spawn(async move { ... })` inside `on_navigation`. The callback returns immediately; the async task runs on the Tokio runtime and dispatches to main thread safely.

### `peek.url()` race condition on Windows
- After navigating to a sentinel URL (`peek-action.tauri.internal/expand`) and calling `return false` to cancel, WebView2 still briefly holds the sentinel as the current URL.
- Calling `peek.url()` in `expand_peek` (called from the async task) can return the sentinel URL instead of the real content URL.
- Fix: store the real URL in `PeekUrl(Mutex<Option<tauri::Url>>)` app state **before** creating the peek webview. `expand_peek` reads from state, not from `peek.url()`.

### `win.close()` vs `win.destroy()` on Windows
- `win.close()` sends a close request that can be intercepted by `CloseRequested` handlers or JS `beforeunload`. This was causing the standalone pop-out window to become unresponsive.
- Fix: in `on_window_event`, call `api.prevent_close()` then `win.destroy()` on `CloseRequested`. This bypasses all interception and forces the WebView2 process to release.

### `on_new_window` callback restrictions
- On Windows, calling `win.navigate(url)` from inside an `on_new_window` callback (to redirect new-window requests into the existing window) corrupts WebView2 state.
- Fix: create a fresh `WebviewWindow` via `WebviewWindowBuilder` instead of navigating in-place.

### Sentinel URL interception pattern
- Using `peek-action.tauri.internal` as a fake host for toolbar action URLs is clean and reliable — it never collides with real navigation.
- Return `false` from `on_navigation` to cancel the navigation; the webview stays on the previous URL (visually).

### Windows build environment
- `cargo build` fails with `LNK1181: cannot open input file 'dbghelp.lib'` if `LIB` env var doesn't include the Windows SDK lib path.
- Required env vars (PowerShell before building):
  ```powershell
  $env:LIB = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64"
  $env:INCLUDE = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\ucrt;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\um;C:\Program Files (x86)\Windows Kits\10\Include\10.0.26100.0\shared"
  ```

### Tauri icon generation
- `npx tauri icon <source.png>` regenerates all icon formats and sizes including `.ico`, `.icns`, and all Windows APPX tile sizes from a single source PNG.
- Source should be at least 512x512 with transparency.

### Tauri `unstable` feature
- `add_child()` on `Window` requires the `unstable` Tauri feature flag: `tauri = { version = "2", features = ["image-png", "unstable"] }`.
- This API may change in future Tauri 2 releases.

### macOS `cocoa`/`objc` dependencies
- macOS dock icon switching requires `cocoa = "0.25"` and `objc = "0.2.7"` as target-specific dependencies.
- These compile fine on Linux/Windows (they are conditionally compiled with `#[cfg(target_os = "macos")]`).

## Patterns

- All Windows-specific bugs in this app trace back to the WebView2 threading model — the first question when debugging a Windows-only freeze is "are we on the right thread?"
- Sentinel URL pattern is useful for injecting toolbar actions into webviews without a Tauri command IPC bridge.

## Mistakes to Avoid

- Do not call `peek.close()` or any `WebviewWindowBuilder` inside `on_navigation` directly — always spawn.
- Do not call `peek.url()` to retrieve content URL after a sentinel navigation on Windows.
- Do not use `win.close()` for pop-out windows — use `prevent_close()` + `destroy()`.
- Do not navigate in `on_new_window` callbacks on Windows.
- Do not run `npm run dev` (or `cargo build`) from a shell session that doesn't have MSVC env vars set.
- Do not commit `.claude/` — it's in `.gitignore` and contains local settings.

## Archive Summary

No archived learnings yet. Initial setup 2026-05-28.

## Archive Pointers

- [Archived Learnings Index](../.archive/learnings/INDEX.md)
