# Active Logs

Last updated: 2026-11-09

## Current Session

### 2026-11-09T11:30:00Z — claude/serene-goodall-xyarrc

Implemented the review plan (critique cross-checked by a second model) in one pass, on the user's instruction to "do all".

Changes:

- Speed: release profile (`lto`, `codegen-units = 1`, `panic = "abort"`, `strip`, `opt-level = "s"`), `rlib`-only crate type, hidden-until-loaded main window with Gmail background colour (6 s fallback show).
- Responsive: main minimum 720x500, pop-outs sized to 90% of the monitor (max 1180x820) and centred, peek uses more space on small windows, repositions on `ScaleFactorChanged`, View menu zoom (Ctrl/Cmd `+`/`-`/`0`).
- UX: unread count from the document title (window title, badge, tray tooltip, background notification), tray with close-to-tray, macOS Reopen, `mailto:` deep link + single instance + in-page `mailto:` rewrite, window-state persistence, persisted icon choice, Window menu (Cmd+W), Esc closes peek, theme-aware larger peek toolbar, offline banner, download handler, Compose keeps the current account.
- Security: non-Google new-window links (after unwrapping `google.com/url?q=`) open in the system browser; unknown hosts are titled by host, not "Google Workspace"; mutex `unwrap` removed; `NSImage` leak fixed.
- Windows workarounds (`beforeunload` suppression, `destroy()` close) now compile only on Windows.
- macOS `Info.plist` adds camera/microphone usage strings for Meet.
- Release/CI: new `ci.yml` (fmt, clippy `-D warnings`, tests on ubuntu-22.04); release builds on ubuntu-22.04, adds Intel macOS, tags from `__VERSION__`, adds concurrency; Arch job/PKGBUILD use `--no-bundle`; MIT `LICENSE` added.
- Version scheme changed (user-approved) so `Z` fits MSI's 65535 limit; version set to `1.2.7499`.

Validation: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, and 6 new unit tests pass on Linux; debug build launched under Xvfb for 15 s without errors. Not verified: macOS/Windows builds, real Gmail sign-in, tray/badge/notification behaviour on each OS.

Next steps:

- Verify on macOS and Windows (downloads, Meet permissions, mailto default-handler registration, close-to-tray).
- Consider updater, code signing, and a per-platform user agent.

### 2026-11-09T18:45:00Z — claude/serene-goodall-xyarrc

Release `v1.2.7499` published from `main` via a manual workflow run (tag pushes are blocked from the agent environment). The Arch job built but skipped upload because its steps required a tag ref. `release.yml` now reads the version from `Cargo.toml`, always uploads the Arch package and PKGBUILD to `v<version>`, and has an `arch_only` dispatch input to add the Arch package to an existing release without rebuilding other platforms. Documentation review: this log entry is the documentation update.

## Archive Summary

Earlier sessions (2026-05-28 bootstrap; 2026-09-14 icon refresh and agent-context setup) are archived.

## Archive Pointers

- [logs-2026-09-14-icon-and-agent-context.md](../.archive/logs/logs-2026-09-14-icon-and-agent-context.md)
- [logs-2026-05-28-project-bootstrap.md](../.archive/logs/logs-2026-05-28-project-bootstrap.md)
- [Archived Logs Index](../.archive/logs/INDEX.md)
