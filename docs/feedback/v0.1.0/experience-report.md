# Rotiv v0.1.0 — AI Agent Experience Report

**Date:** 2026-03-14
**Agent:** Claude Sonnet 4.6 (Claude Code CLI)
**Task:** Build a todo app from scratch using the Rotiv framework

---

## Summary

The framework's **design vision is excellent** for AI-agent-driven development. The conventions are predictable, the CLI output is structured, and the self-describing error format is well-suited for agent consumption. However, the v0.1.0 standalone release has **three blocking issues** that prevent any project from running end-to-end outside of the Rotiv monorepo.

---

## What Worked Well

### 1. CLI scaffolding (`rotiv new`, `rotiv add`)
`rotiv new todo-app` instantly produced a coherent project with the right structure. `rotiv add model Todo` and `rotiv add route todos/[id]` generated heavily-annotated boilerplate that made the framework conventions immediately clear. As an agent, I didn't need to guess — the comments in generated files served as inline documentation.

### 2. Validation and spec tools
`rotiv validate` gave a clean pass with zero noise. `rotiv spec-sync` correctly detected 2 routes and 2 models after edits. `rotiv context-regen` produced a well-structured `.rotiv/context.md` that I could use as a project summary. `rotiv diff-impact` correctly identified both routes as affected when the `todo.ts` model changed — this is a genuinely useful capability for impact analysis.

### 3. Error message quality
The structured error format (code, message, file, line, expected, got, suggestion) is the right design. Even where `suggestion` and `corrected_code` were null, the format itself is agent-friendly. The `--json` flag on all commands is excellent.

### 4. `rotiv explain`
`rotiv explain migrate` gave accurate, helpful documentation inline. This is a strong feature for agents operating without internet access to external docs.

### 5. Conventions are consistent and predictable
File-system routing (`[id].tsx` → `:id`), the `defineRoute()` / `defineModel()` API, and the three-export pattern (loader / action / component) are all obvious and unambiguous. I wrote the full todo app (model + 2 routes) without any trial and error.

---

## Blocking Issues

| # | Issue | Severity |
|---|-------|----------|
| 1 | `rotiv dev` fails: `@rotiv/route-worker` not bundled in release binary | Blocker |
| 2 | `@rotiv/*` npm packages not published — `pnpm install` fails | Blocker |
| 3 | `rotiv migrate` fails outside monorepo — internal script path not bundled | Blocker |

All three issues stem from the same root cause: **the standalone binary assumes it's running inside the Rotiv monorepo**. The binary needs to either bundle or self-contain the TypeScript worker, migration runner, and npm packages.

---

## Minor Issues

| # | Issue | Severity |
|---|-------|----------|
| 4 | `rotiv dev` startup banner shows wrong path for dynamic routes (`[id].tsx` instead of `todos/[id].tsx`) | Minor |
| 5 | `rotiv add model` error doesn't auto-suggest the PascalCase correction in `corrected_code` field | Enhancement |

---

## Recommendations

1. **For the v0.1.0 release to be usable:** Bundle `@rotiv/route-worker` compiled JS and the migration runner script as embedded assets in the binary. Publish `@rotiv/*` packages to npm (even as `0.1.0-alpha`).

2. **README gap:** The README shows `pnpm install` and `rotiv dev` as the immediate next steps after `rotiv new`. These both fail. Add a note about the current monorepo-only limitation.

3. **`rotiv explain`** should be updated to mention the standalone binary limitation for `migrate` and `dev`.

4. **The `corrected_code` field** in error JSON is a great idea — fill it in wherever the framework can compute it (e.g., PascalCase model names, path normalization).

---

## Completed App

The todo app code is at `c:/Users/Vitor/Documents/codebase/todo-app/` and is structurally complete:
- `app/models/todo.ts` — Todo model with title, status (pending/done), createdAt
- `app/routes/index.tsx` — list all todos, add new todo via form
- `app/routes/todos/[id].tsx` — view todo detail, toggle status

`rotiv validate` passes with no issues. The app would work end-to-end once the blocking release issues are resolved.
