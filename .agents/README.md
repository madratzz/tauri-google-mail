# Agent Context System

Schema version: 1

This folder stores active AI-agent context for this project. It enables future agents to understand the current state, preserve useful discoveries, and avoid repeating work without loading all historical detail.

## Purpose

- Store active project context, stable memory, recent learnings, and work logs.
- Maintain agent-specific operating guidance.
- Point to older immutable material in `.archive/`.

## Active Files

- `context.md` — concise current project context.
- `memory.md` — durable project facts and verified preferences.
- `learnings.md` — discoveries, gotchas, and repeatable lessons.
- `logs.md` — recent meaningful work in newest-first order.
- `INDEX.md` — table of contents and freshness metadata.
- `agents/default-agent.md` — default operating profile.

## Archive

Older material lives in `../.archive/`. Do not delete useful historical context unless the user explicitly asks; archive and verify it before trimming active files.
