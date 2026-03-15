# Rotiv v0.2.2 — Patch Plan

**Source:** v0.2.1 AI agent experience report (weather app, 2026-03-15)

---

## Issues Found

### Bug 1: Table name pluralization produces incorrect names

**Examples:** `Search→searchs`, `Category→categorys`, `Query→querys`, `Match→matchs`, `Status→statuss`

**Root cause:** `to_snake_plural()` in `add.rs` only handled `s/x/z → es` and defaulted to `+s`.
It did not handle:
- `ch/sh` endings → `es` (search→searches, match→matches)
- consonant+`y` endings → `ies` (category→categories, query→queries)

**Fix:** Add a proper `pluralize()` helper with rules for `ch/sh → es`, `consonant+y → ies`, `s/x/z → es`, default `+s`.

File: `crates/rotiv-cli/src/commands/add.rs`

---

### Bug 2: `rotiv migrate` error body still empty when tsx fails to load

**Symptom:**
```
error [E_UNKNOWN] migration error: Migration failed: {"error":"Error: drizzle-kit generate failed:\n"}
```

**Root cause:** Two issues in `runner.ts`:
1. `result.stderr ?? result.stdout` — `??` does not fall through when `stderr` is an empty string (not null/undefined). drizzle-kit's error output ends up in stdout but `stderr` is `""` so stdout is never reached.
2. `result.error` (the Node.js spawn error, set when the child process couldn't start at all) was never checked. When tsx fails to resolve `@rotiv/orm`, Node crashes before drizzle-kit runs, setting `result.error` but leaving `stderr` and `stdout` empty.

**Fix:**
- Check `result.error` first and surface its message
- Use `[stderr, stdout].filter(Boolean).join()` instead of `??` to capture whichever has content

File: `packages/@rotiv/migrate-script/src/runner.ts`

---

### Operational: `@rotiv/*` packages not on npm

Same as previous reports. Requires setting `NPM_TOKEN` secret in GitHub repository settings.
No code change needed. The publish workflow is correct.

---

## Implementation

### Wave 1 — Pluralization fix (`add.rs`)

Replace `to_snake_plural` with one that calls a new `pluralize()` helper covering:
- `word.ends_with("ch") || ends_with("sh")` → `es`
- `consonant + y` → `ies`
- `s/x/z` → `es`
- default → `s`

Add test cases: Search→searches, Match→matches, Category→categories, Query→queries, Status→statuses, Box→boxes.

### Wave 2 — Migrate error capture (`runner.ts`)

In both `generateMigrations` and `applyMigrations`:
1. Check `result.error` first → throw with `result.error.message`
2. Replace `result.stderr ?? result.stdout` with `[result.stderr, result.stdout].filter(Boolean).join("\n") || "no output"`

### Wave 3 — Version bump + changelog + tag

- Bump `Cargo.toml` workspace version to `0.2.2`
- Rebuild migrate-script `dist/` so updated runner is embedded in the binary
- Write `docs/changelog/v0.2.2.md`
- Tag `v0.2.2`
