# [Bug] `rotiv dev` output shows wrong file path for dynamic route

**Labels:** `bug`

## Environment
- OS: Windows 11 Pro (x64)
- CLI: `rotiv-windows-x64.exe` v0.1.0

## Steps to reproduce
```bash
rotiv new todo-app
cd todo-app
rotiv add route todos/[id]
rotiv dev
```

## Actual output
```
  GET  /todos/:id  →  app/routes/[id].tsx
```

## Expected output
```
  GET  /todos/:id  →  app/routes/todos/[id].tsx
```

## Notes
The route resolves and the path is correct (`/todos/:id`), but the displayed file path in the startup banner drops the `todos/` subdirectory prefix. Minor cosmetic bug, but confusing when navigating to the correct file.
