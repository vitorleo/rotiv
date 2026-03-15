# Rotiv v0.2.1 — Patch Plan

**Source:** v0.2.0 AI agent experience report (blog app, 2026-03-15)

---

## Issues Found

### Bug 1 (Critical): Worker temp dir missing `"type": "module"` — NEW in v0.2.0

**Symptom:**
```
Error: Transform failed with 1 error:
.../src/index.ts:68:0: ERROR: Top-level await is currently not supported with the "cjs" output format
```

**Root cause:** `write_embedded_worker()` in `dev.rs` extracts 6 `.ts` files into a temp directory
but writes no `package.json`. Without `"type": "module"`, Node.js defaults to CommonJS for the
directory. The worker uses top-level `await` which is ESM-only.

**Fix:** After writing the 6 source files, write `{"type":"module"}` to `$tmpdir/package.json`.

**Same bug exists in `write_embedded_migrate_script()` in `migrate.rs`** — same fix.

Files: `crates/rotiv-cli/src/commands/dev.rs`, `crates/rotiv-cli/src/commands/migrate.rs`

---

### Bug 2: `@rotiv/*` packages not on npm registry

**Symptom:** `pnpm install` returns 404 for all `@rotiv/*` packages.

**Root cause:** The npm publish workflow (`npm-publish.yml`) requires `NPM_TOKEN` secret to be
configured in GitHub repository settings. This has not been done.

**Fix:** Set `NPM_TOKEN` secret in GitHub → Settings → Secrets → Actions. The workflow itself
is correct.

**Note:** This is an operational step, not a code change.

---

### Bug 3: `rotiv migrate` error swallows subprocess output

**Symptom:** Error message shows `"Migration failed:\n"` with no useful content.

**Root cause:** The drizzle-kit child process was writing its errors to stdout, not stderr.
The migration runner only captured stderr. When stderr is empty, the error message is blank.

**Fix:** Capture both stdout and stderr from the child process. If both are non-empty, join them.
Fall back gracefully with a default message if both are empty.

File: `crates/rotiv-orm/src/migration.rs`

---

## Implementation

All three code fixes are one-to-five line changes. No new abstractions needed.

### Wave 1 — Bug 1 (dev.rs + migrate.rs)

In `write_embedded_worker()`, after the file loop:
```rust
std::fs::write(dir.path().join("package.json"), r#"{"type":"module"}"#)?;
```

Same in `write_embedded_migrate_script()`.

### Wave 2 — Bug 3 (migration.rs)

Replace:
```rust
let stderr = String::from_utf8_lossy(&output.stderr).to_string();
return Err(OrmError::MigrationFailed(stderr));
```

With:
```rust
let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
let detail = match (stderr.is_empty(), stdout.is_empty()) {
    (false, false) => format!("{stderr}\n{stdout}"),
    (false, true)  => stderr,
    (true,  false) => stdout,
    (true,  true)  => "no output from drizzle-kit".to_string(),
};
return Err(OrmError::MigrationFailed(detail));
```

### Wave 3 — Bump version + changelog

- Bump `Cargo.toml` workspace version to `0.2.1`
- Write `docs/changelog/v0.2.1.md`
- Tag and push `v0.2.1`
