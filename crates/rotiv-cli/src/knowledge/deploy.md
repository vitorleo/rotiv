# Deploy

## Explanation

`rotiv deploy` deploys your compiled Rotiv project to a Linux VPS via SSH and SCP. It copies the server binary, runs pending migrations, and restarts the systemd service — all in one command.

Deploy configuration lives in `.rotiv/deploy.json` (create with `rotiv deploy --init`):

```json
{
  "host": "YOUR_SERVER_IP",
  "user": "root",
  "remote_path": "/opt/rotiv-apps/myapp",
  "service_name": "myapp"
}
```

**Important:** Add `.rotiv/deploy.json` to `.gitignore` if your host/user are sensitive.

### Deploy steps (in order)

1. **Build** — runs `rotiv build` to produce `dist/server` (skip with `--skip-build`)
2. **Upload** — `scp dist/server <user>@<host>:<remote_path>/server`
3. **Migrate + restart** — SSH in, run `./server migrate`, then `sudo systemctl restart <service>`

The `rotiv` binary must be on PATH for step 1. `ssh` and `scp` must be on PATH for steps 2 and 3. Your local SSH key/agent is used automatically.

### Prerequisites on the server

- A systemd service unit file at `/etc/systemd/system/<service_name>.service`
- The `<remote_path>` directory must exist and be writable by the deploy user
- `sudo systemctl restart` must not require a password for the deploy user (sudoers)

### Dry run

Use `--dry-run` to print all commands without executing:

```bash
rotiv deploy --dry-run
```

Output:
```
  run   [upload binary]: scp dist/server root@YOUR_SERVER_IP:/opt/rotiv-apps/myapp/server
  run   [migrate + restart]: ssh root@YOUR_SERVER_IP "..."
```

## Code Example

```bash
# First-time setup: create deploy config
rotiv deploy --init
# Edit .rotiv/deploy.json with your server details

# Deploy (full pipeline: build → upload → migrate → restart)
rotiv deploy

# Deploy without rebuilding (faster if you built recently)
rotiv deploy --skip-build

# Preview steps without executing
rotiv deploy --dry-run --skip-build

# Override config at deploy time
rotiv deploy --host 1.2.3.4 --user deploy --path /srv/app --service myapp

# JSON output (useful for CI pipelines)
rotiv deploy --skip-build --json
# → { "ok": true, "host": "...", "remote_path": "...", "service": "...", "dry_run": false }
```

### Example systemd unit file (`/etc/systemd/system/myapp.service`)

```ini
[Unit]
Description=My Rotiv App
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/rotiv-apps/myapp
ExecStart=/opt/rotiv-apps/myapp/server
Restart=always
RestartSec=5
Environment=PORT=3000
Environment=NODE_ENV=production

[Install]
WantedBy=multi-user.target
```

After creating: `sudo systemctl enable myapp && sudo systemctl start myapp`

## Related

- build, migrate, context
