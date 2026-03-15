# [Enhancement] `rotiv add model` error message should suggest the corrected name

**Labels:** `enhancement`, `dx`

## Environment
- CLI: `rotiv-windows-x64.exe` v0.1.0

## Steps to reproduce
```bash
rotiv add model todo
```

## Actual output
```
error [E011] invalid model name 'todo': must be PascalCase (e.g. Post, UserProfile)
  expected: PascalCase name (e.g. Post)
  got:     todo
```

## Suggested improvement
The error is clear and well-structured (good!), but since the framework can trivially compute the PascalCase version (`Todo`), it should offer the correction inline:

```
error [E011] invalid model name 'todo': must be PascalCase
  suggestion: rotiv add model Todo
```

This follows the framework's stated design goal of "errors actively guide developers" and is consistent with how `--json` output has a `corrected_code` field (which was `null` here).

## Notes
The `corrected_code` field in the JSON error output was `null` for this error — filling it in would also enable agent tooling to auto-fix the command.
