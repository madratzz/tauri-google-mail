# Default Agent

Last updated: 2026-05-28

## Agent Role

This agent maintains and extends the Gmail Desktop Tauri 2 app. It handles feature development, bug fixes, icon/asset updates, CI/release management, and agent context maintenance.

## Operating Rules

- Read `.agents/context.md` first.
- Read `.agents/memory.md` second.
- Read `.agents/learnings.md` third.
- Read `.agents/logs.md` fourth.
- Update active files after meaningful work.
- Archive older files when the date changes or active files become too large.
- Never delete historical context without archiving it first.
- Keep summaries concise but useful.
- Use relative links.
- Do not store secrets, API keys, passwords, tokens, private keys, or credentials.

## Project Context

Gmail Desktop is a Tauri 2 cross-platform desktop app that wraps `mail.google.com` in a system webview. The entire app logic is in `src-tauri/src/lib.rs`. There is no JS frontend. It uses a Safari user-agent, a peek overlay for in-app link previews, a pop-out button to open links in standalone windows, and an icon switcher menu.

**Critical Windows behaviors to remember:**
- `on_navigation` fires on a WebView2 background thread — always use `tauri::async_runtime::spawn` for window ops inside it.
- `peek.url()` may return the sentinel URL after navigation cancel — read URL from `PeekUrl` app state instead.
- Use `prevent_close()` + `win.destroy()` for reliable window closing on Windows.

## Responsibilities

- Maintain context, logs, memory, learnings, archives.
- Keep archive indexes updated.
- Implement features and bug fixes in `src-tauri/src/lib.rs`.
- Manage releases via git tags triggering GitHub Actions CI.

## Workflow

1. Read active context files (context → memory → learnings → logs).
2. Perform the requested task.
3. Update `.agents/logs.md`.
4. Update `.agents/memory.md` if stable facts were discovered.
5. Update `.agents/learnings.md` if new lessons were learned.
6. Update `.agents/context.md` if project direction, structure, or goals changed.
7. Archive old material when needed.
8. Update all relevant indexes.

## Agent Maintenance Checklist

- [ ] Updated `.agents/logs.md`.
- [ ] Updated `.agents/context.md` if goals, architecture, structure, or constraints changed.
- [ ] Updated `.agents/memory.md` if stable facts were discovered.
- [ ] Updated `.agents/learnings.md` if useful lessons were learned.
- [ ] Archived old material if the date changed or files became too large.
- [ ] Updated `.agents/INDEX.md`.
- [ ] Updated relevant `.archive/*/INDEX.md` files.
- [ ] Redacted sensitive data.
- [ ] Preserved useful summaries in active files.
