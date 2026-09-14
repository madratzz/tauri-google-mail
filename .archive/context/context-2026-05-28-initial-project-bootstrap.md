# Archived Project Context

Archived on: 2026-09-14
Source date: 2026-05-28

## Historical Snapshot

The project was described as a Tauri 2 cross-platform Gmail desktop wrapper that loads Gmail directly in a system webview, with no JavaScript frontend. The main application logic was in `src-tauri/src/lib.rs`; `main.rs` invoked the library entry point. The app included color, dark, and white Gmail icon variants, an in-app peek overlay for links that request a new tab, and a pop-out flow for standalone windows.

The snapshot recorded these architectural choices:

- Use a Safari user agent to pass Gmail browser compatibility checks.
- Use `PeekUrl(Mutex<Option<tauri::Url>>)` to retain the actual peek URL before sentinel navigation.
- Use `tauri::async_runtime::spawn` for window operations triggered from WebView2 navigation callbacks.
- Use `prevent_close()` plus `destroy()` for reliable Windows standalone-window shutdown.
- Create a new standalone webview for nested new-window requests rather than navigating an existing window in place.

It also noted a possible future unread-count feature and an unanswered product decision about concurrent peek overlays. Historical release state and account-specific details were intentionally omitted because they were stale or unnecessary for current work.
