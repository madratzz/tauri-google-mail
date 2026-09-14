# Active Learnings

Last updated: 2026-09-14

## Recent Learnings

- Windows WebView2 runs `on_navigation` callbacks away from the UI thread; direct window operations there can deadlock. Dispatch them through `tauri::async_runtime::spawn`.
- A cancelled sentinel navigation can make `peek.url()` temporarily report the sentinel address. Preserve the original URL in `PeekUrl` before creating the overlay.
- Opening a fresh standalone webview for nested new-window requests avoids the state corruption associated with navigating an existing WebView2 window in that callback.
- `prevent_close()` followed by `destroy()` is the project’s documented workaround for unreliable standalone-window closing on Windows.
- macOS release validation should inspect the downloaded DMG’s application bundle, not only a local build; see `docs/macos-damaged-release.md`.

## Patterns

- Native Tauri code and injected webview JavaScript coordinate overlay controls through a dedicated sentinel host whose navigation is cancelled after handling.
- Platform-specific operational knowledge belongs in `docs/` and a concise cross-reference belongs here.

## Mistakes to Avoid

- Do not add a frontend framework by default.
- Do not perform window operations directly inside WebView2 navigation callbacks.
- Do not depend on `peek.url()` after sentinel navigation.
- Do not copy sensitive, personal, or machine-specific values from context into new logs or summaries.

## Archive Summary

The detailed initial Windows/WebView2 findings are preserved in a redacted 2026-05-28 archive snapshot.

## Archive Pointers

- [Archived Learnings Index](../.archive/learnings/INDEX.md)
