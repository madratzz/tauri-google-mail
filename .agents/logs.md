# Active Logs

Last updated: 2026-05-28

## Current Session

### 2026-05-28

**Summary:** Full project bootstrap — created Tauri 2 Gmail desktop app from scratch, fixed three Windows-specific bugs, updated icons, and pushed alpha release v1.0.0-alpha.1.

**Files touched:**
- `src-tauri/src/lib.rs` — entire app logic, multiple iterations
- `src-tauri/Cargo.toml` — dependencies, crate config
- `src-tauri/tauri.conf.json` — app config, CSP null
- `.github/workflows/release.yml` — CI release workflow
- `src-tauri/icons/gmail-color.png` — replaced with dashboardicons.com version
- `src-tauri/icons/gmail-dark.png` — replaced with dashboardicons.com version
- `src-tauri/icons/gmail-white.png` — replaced with dashboardicons.com light variant
- `src-tauri/icons/icon.png / icon.ico / icon.icns` — regenerated from gmail-color.png
- All APPX tile PNGs — regenerated via `npx tauri icon`
- `package.json` — Tauri CLI devDependency
- `src-tauri/Cargo.lock` — committed
- `package-lock.json` — committed
- `.gitignore` — added `.claude/`
- `.agents/*` — created agent context system
- `.archive/*` — created archive structure

**Decisions made:**
- No JS frontend — pure Rust + Tauri 2 webview wrapper.
- Safari user-agent to pass Google browser checks.
- Peek overlay uses `add_child()` with `unstable` Tauri feature.
- Sentinel URL `peek-action.tauri.internal` for toolbar actions.
- `tauri::async_runtime::spawn` for all window ops inside `on_navigation` (deadlock fix).
- `PeekUrl` Mutex app state to reliably pass URL to `expand_peek` (race condition fix).
- `prevent_close()` + `destroy()` for standalone window close (WebView2 release fix).
- Icons sourced from dashboardicons.com / selfhst CDN (CC BY 4.0).

**Issues found and fixed:**
1. **White pop-out window** — `peek.url()` returned sentinel URL on Windows. Fixed with `PeekUrl` state.
2. **Close button broken on pop-out** — `win.navigate()` inside `on_new_window` corrupted WebView2. Fixed with `open_standalone_window()` helper.
3. **Both windows freeze/deadlock** — `expand_peek` called from `on_navigation` background thread tried to do main-thread ops. Fixed with `tauri::async_runtime::spawn`.
4. **LNK1181 linker error** — missing `LIB` env var for Windows SDK. Fixed by setting MSVC env vars before build.
5. **`.claude/` accidentally staged** — removed with `git rm --cached`, added to `.gitignore`.

**Next steps:**
- Monitor GitHub Actions build for v1.0.0-alpha.1 at https://github.com/madratzz/tauri-google-mail/actions
- Publish the draft release at https://github.com/madratzz/tauri-google-mail/releases when builds complete.
- Consider unread badge / notification count feature.
- Consider multiple simultaneous peek overlay support.

## Recent Previous Sessions

No previous sessions recorded yet.

## Archive Summary

No archived logs yet. Initial setup 2026-05-28.

## Archive Pointers

- [Archived Logs Index](../.archive/logs/INDEX.md)
