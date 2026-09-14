# Repository Instructions

## Scope and precedence

Follow instructions in this order: explicit user request; this file and other repository guidance; then agent defaults. Keep changes scoped, preserve unrelated work, and never store secrets, credentials, private keys, tokens, passwords, personal data, internal URLs, or customer data in repository context. Redact uncertain sensitive values with placeholders.

## Project

Gmail Desktop is a Rust/Tauri 2 desktop wrapper for Gmail. It has no JavaScript frontend; application behavior is primarily in `src-tauri/src/lib.rs`. Treat the repository README, package manifests, Tauri configuration, workflow files, and `.agents/` active context as authoritative sources before making project changes.

## Git, pull requests, and versions

- Start implementation work from `development`, then create one `feature/<short-kebab-case-description>` branch per task unless repository-enforced policy says otherwise.
- Make small, cohesive commits. Include relevant documentation and agent-context updates in the same commit.
- Open a pull request for each merge. Merge feature branches into `development` only through that pull request; do not directly merge or squash merge. Update `main` only through a release pull request from `development`.
- Do not commit unrelated changes, rewrite shared history, force-push, or change Git configuration without explicit user approval.
- If `development` does not exist or the policy conflicts with enforced repository rules, stop and report the conflict before creating a branch or pull request.
- Assign versions as `X.Y.Z`: increase `X` only for intentional breaking releases and `Y` only for backward-compatible features. Set `Z` to `floor(unix_timestamp_seconds / 60)` in UTC; never reuse or decrease it. Record the version, UTC assignment time, and major/minor reason in release documentation.

## Documentation and naming

- Every completed task needs a substantive documentation update. If none is warranted, add a precise documentation-review note to `.agents/logs.md`.
- Use Tauri 2 APIs; do not introduce a JavaScript frontend or build tool without explicit approval.
- Keep Rust names idiomatic (`snake_case` for functions). For any Unity C# introduced later, private `[SerializeField]` fields use PascalCase, for example `[SerializeField] private float MovementSpeed;`.

## Agent context and archives

- `.agents/` contains concise, active context; `.archive/` contains dated immutable snapshots. Both are version-controlled and must not be added to `.gitignore`.
- Read `.agents/context.md` first, then relevant `memory.md`, `learnings.md`, `logs.md`, and the default agent profile before meaningful work.
- Re-read a shared context file immediately before editing it and merge safely. Include a session identifier in logs when multiple agents may be working.
- Update `logs.md` after meaningful work. Update the other active files only for durable facts, learnings, goals, architecture, or constraints that changed.
- On a date change or when an active file exceeds its soft size limit, create and verify a unique dated archive file first, update its category index in newest-first order, retain a concise active summary and pointer, and only then trim active material. Never edit a verified archive or delete history without explicit user approval.
- Keep archive filenames `type-YYYY-MM-DD[-short-gist][-NN].md`, with lowercase hyphenated slugs. Update only indexes affected by a change.

## Technical constraints

- Gmail is loaded directly in the native webview. The Safari user agent, peek overlay, and pop-out flow have Windows/WebView2-specific behavior documented in `docs/windows-popup-bugs.md`.
- In WebView2 `on_navigation` handlers, dispatch window operations via `tauri::async_runtime::spawn`; do not call them directly. Preserve `PeekUrl` state handling for sentinel navigation and forced `destroy()` close handling unless a tested replacement is supplied.
- Keep sensitive values out of source, context, logs, docs, archives, commits, and command output. Do not copy existing sensitive data into summaries.

## Before handoff

Validate changed files, links, archive indexes, ordering, and obvious sensitive-data redaction. State any unresolved assumptions or branch-policy conflicts clearly.
