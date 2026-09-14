# Archived Learnings

Archived on: 2026-09-14
Source date: 2026-05-28

## Windows/WebView2 Behavior

- `on_navigation` executes on a WebView2 background thread. Direct window operations there can deadlock because the UI thread is waiting for the callback to return. Dispatch the work through `tauri::async_runtime::spawn` instead.
- After cancelled sentinel navigation, a peek webview can momentarily report the sentinel address rather than the content address. Persist the content URL before creating the peek overlay and use that stored value for pop-out.
- Redirecting a new-window request by navigating the existing window can corrupt WebView2 state. Open a new standalone webview instead.
- A normal close request can be intercepted or fail to release the WebView2 process. The established workaround is to prevent the close request and destroy the window.

## General Patterns

- A dedicated sentinel host can communicate toolbar actions from injected webview JavaScript without adding an IPC bridge; cancel its navigation after handling.
- Tauri child-webview functionality depended on the unstable API surface, so future dependency upgrades need targeted testing.

This snapshot was redacted of environment-specific paths and values.
