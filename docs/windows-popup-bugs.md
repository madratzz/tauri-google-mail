# Windows Popup/Pop-Out Bugs in Tauri 2

All three bugs are Windows-only. Root cause: WebView2 fires `on_navigation` callbacks on a **background thread**, unlike WKWebView on macOS which fires on the main thread.

---

## Bug 1 — Pop-out window is white (blank)

**Cause:** When the peek overlay navigates to a sentinel URL (e.g. `peek-action.tauri.internal/expand`) to signal an action, `on_navigation` cancels it with `return false`. On Windows/WebView2, the navigation cancel is async — by the time the expand handler calls `peek.url()`, WebView2 has already set the current URL to the sentinel instead of the real content URL.

**Fix:** Store the real URL in Tauri app state (`Mutex<Option<tauri::Url>>`) **before** creating the peek webview. The expand handler reads from state instead of calling `peek.url()`.

```rust
struct PeekUrl(Mutex<Option<tauri::Url>>);

// Before creating the webview, store the URL:
if let Ok(mut guard) = app.state::<PeekUrl>().0.lock() {
    *guard = Some(url.clone());
}

// In expand handler, read from state:
let url = {
    let state = app.state::<PeekUrl>();
    let guard = state.0.lock().unwrap();
    match guard.clone() {
        Some(u) => u,
        None => return,
    }
};
```

---

## Bug 2 — Close button on pop-out window does nothing

**Cause:** Calling `win.navigate(url)` inside an `on_new_window` callback to redirect content into an existing window corrupts WebView2 internal state on Windows, leaving the window unresponsive. Also, `win.close()` can be silently intercepted or fail to release the WebView2 process.

**Fix:** Create a brand new `WebviewWindow` via `WebviewWindowBuilder` instead of navigating in-place. For closing, use `prevent_close()` + `win.destroy()` in the `CloseRequested` handler.

```rust
win.on_window_event(move |event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = win_clone.destroy(); // force-releases WebView2
    }
});
```

---

## Bug 3 — Both windows freeze / deadlock (most critical)

**Cause:** `on_navigation` fires on a WebView2 background thread on Windows. Any operation that needs the main thread — `peek.close()`, `WebviewWindowBuilder::new()`, any window op — called directly inside the callback causes a deadlock: the main thread waits for the callback to return, but the callback waits for the main thread.

**Fix:** Wrap **all** window operations inside `on_navigation` in `tauri::async_runtime::spawn`. The callback returns immediately; the async task dispatches safely via Tokio.

```rust
.on_navigation(move |nav_url| {
    if nav_url.host_str() == Some("peek-action.tauri.internal") {
        let app = app_handle.clone();
        match nav_url.path() {
            "/expand" => { tauri::async_runtime::spawn(async move { expand_peek(&app); }); }
            "/close"  => { tauri::async_runtime::spawn(async move { close_peek(&app); }); }
            _ => {}
        }
        return false;
    }
    true
})
```

---

## Summary

| Bug | Trigger | Fix |
|---|---|---|
| White pop-out window | `peek.url()` returns sentinel URL after async cancel | Store real URL in app state before creating webview |
| Close button broken | `win.navigate()` in `on_new_window` corrupts WebView2 | Create new `WebviewWindow` instead; use `destroy()` not `close()` |
| Both windows freeze | Window ops called from WebView2 background thread | Wrap all window ops in `tauri::async_runtime::spawn` |
