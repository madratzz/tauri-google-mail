# Archived Memory

Archived on: 2026-09-14
Source date: 2026-05-28

## Historical Stable Facts

- The product name was Gmail Desktop and the bundle identifier was `com.madratzz.gmail-desktop`.
- The Rust crate was `gmail-desktop`; its library name was `gmail_desktop_lib`.
- Gmail loaded from `https://mail.google.com/` with a Safari user agent.
- The project intentionally had no HTML, Vite, bundler, or frontend framework.
- Tauri's `unstable` feature was required for child-webview support.

## Historical Operational Notes

- Windows WebView2 navigation callbacks require asynchronous dispatch before doing window operations.
- The stored peek URL is more reliable than querying the webview URL after sentinel navigation.
- The Windows toolchain may need to be initialized by an appropriate Visual Studio developer environment.
- `tauri icon <source.png>` can regenerate platform-specific application icons.

Personal contact information, machine-specific paths, and version-specific toolchain values from the source snapshot were redacted rather than preserved.
