# Default Agent

Last updated: 2026-09-14

## Agent Role

Maintain the Gmail Desktop codebase and its active context, durable memory, learnings, logs, and archives while following the repository-wide rules in [AGENTS.md](../../AGENTS.md).

## Operating Rules

- Read `context.md` first, then relevant `memory.md`, `learnings.md`, and `logs.md` before meaningful work.
- Treat `AGENTS.md` as authoritative for Git, pull-request, versioning, documentation, sensitive-data, and archive policy.
- Update active context only when it adds durable value. Keep entries concise and distinguish verified facts from assumptions.
- Re-read shared files immediately before edits; merge rather than overwrite concurrent changes.
- Archive detailed older material before trimming it from active files. Verify the file and index link first; never edit verified archive snapshots.
- Use relative links and reverse-chronological ordering for logs and archive indexes.
- Never store credentials, tokens, passwords, private keys, personal data, internal URLs, or machine-specific environment values.

## Project Context

Gmail Desktop is a pure Rust/Tauri 2 Gmail wrapper. `src-tauri/src/lib.rs` contains the primary application logic. Preserve documented WebView2 protections: asynchronous navigation-triggered window operations, `PeekUrl` storage before sentinel navigation, and reliable standalone-window shutdown behavior.

## Responsibilities

- Maintain active context and only the relevant archive indexes.
- Preserve current implementation constraints and update project documentation with meaningful work.
- Validate changed links, filenames, ordering, and obvious sensitive-data redaction before handoff.

## Workflow

1. Read the active files and applicable repository documentation.
2. Perform the scoped task.
3. Update `.agents/logs.md` for meaningful work.
4. Update memory, learnings, and context only when the task warrants it.
5. Archive old material on a date change or soft-size threshold, then verify it.
6. Update affected indexes and documentation.
