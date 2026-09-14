# Active Project Context

Last updated: 2026-09-14

## Project Summary

Gmail Desktop is a cross-platform Rust/Tauri 2 wrapper for Gmail. It loads `https://mail.google.com/` directly in a native webview; there is no JavaScript frontend. The current manifest version is `1.1.29823021`.

## Current Goals

- Maintain a stable Gmail desktop wrapper across macOS, Windows, and Linux.
- Preserve the established peek-overlay and standalone-window behavior.
- Keep project instructions and active agent context accurate, concise, and free of sensitive data.

## Current Architecture / Structure

```text
src-tauri/src/lib.rs        Main Tauri application logic
src-tauri/src/main.rs       Native entry point
src-tauri/icons/            Application and menu icon assets
src-tauri/tauri.conf.json   Tauri bundle and runtime configuration
.github/workflows/release.yml  Tag-triggered cross-platform releases
docs/                       Platform-specific operational notes
.agents/                    Active agent context
.archive/                   Immutable historical context snapshots
```

The Tauri app uses a Safari user agent, a child-webview peek overlay for new-window links, sentinel navigation for overlay controls, and standalone windows for pop-outs. `PeekUrl` retains the original URL before sentinel navigation.

## Important Decisions

- Keep the application pure Rust/Tauri; do not add a frontend framework without explicit approval.
- Keep Tauri 2 APIs and the `unstable` feature needed by child-webview support.
- Keep WebView2 navigation callbacks free of direct window operations; dispatch those operations asynchronously.
- Maintain active context in `.agents/` and immutable historical context in `.archive/`.

## Active Constraints

- The root [AGENTS.md](../AGENTS.md) is the canonical repository instruction file.
- The repository currently has `master` as its local and remote default branch; no `development` branch was found on 2026-09-14. The documented branch policy therefore needs user direction before future branch or pull-request work.
- Do not store personal data, credentials, internal URLs, or machine-specific environment values in agent context.

## Current Open Questions

- Decide whether multiple simultaneous peek overlays are a desired product behavior.
- Decide whether unread-count or notification-badge support is in scope.

## Archive Summary

The 2026-05-28 bootstrap context, memory, learnings, logs, and default agent profile were archived and redacted on 2026-09-14. They document the original WebView2 fixes and project setup.

## Archive Pointers

- [Archived Context Index](../.archive/context/INDEX.md)
- [Archived Learnings Index](../.archive/learnings/INDEX.md)
