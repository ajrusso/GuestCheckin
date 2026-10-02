# GuestCheckin

Rust CLI that scans Google Sheets for unregistered guests.

**Modes**

| `submit_mode` | Behavior |
|---------------|----------|
| `unl` (default) | Write UNL files and email them via AWS SES (classic) |
| `soap` | **Blue Glory POC only:** submit via CheckIn `POST /api/ubyport/submit-guests`, mark Sheet column M **only on accept**, SES summary email **without UNL/PDF attachments** |

The soap path is a **temporary POC bridge** (Sheets → CheckIn → UbyPort). It is **not** future product intake. The host register remains the Google Sheet.

## Prerequisites

- Rust toolchain (`cargo`, `rustc`)
- `src/config/config.toml` (copy from `src/config/config.toml.example` and fill in)
- `service_account_key.json` in the project root
- For soap mode: CheckIn running with pilot token + encryption key; `[checkin]` in config

## Development

Run from the project root so config and relative paths resolve:

```powershell
cargo test
cargo run
```

Or build a debug binary:

```powershell
cargo build
```

## Soap mode (Blue Glory POC / #68)

1. Set `submit_mode = "soap"` and fill `[checkin]` in `config.toml` (see example).
2. Ensure CheckIn is up (`GET /api/ubyport/health`) with `UBYPORT_PILOT_TOKEN` matching config.
3. Run once: `cargo run` or Docker (below).
4. Distro email reuses the existing SES HTML template with accepted / rejected / issue tables — **no passport numbers, no UNL attachments**.
5. Encrypted doručenka PDFs land under CheckIn’s host-mounted `pilot-runs/` (not in this agent).

### Docker

```powershell
# Create empty bind targets if missing
New-Item -ItemType File -Force scan_state.json, output.log | Out-Null

docker compose build
docker compose run --rm guestcheckin-agent
```

Daily schedule (host): Task Scheduler or cron → `docker compose run --rm guestcheckin-agent`.

Set `PILOT_RUNS_HOST` if CheckIn’s `pilot-runs` directory is not at `../CheckIn/backend/pilot-runs`.

On Windows Docker Desktop, `host.docker.internal` reaches CheckIn on the host (see `config.toml.example`).

## Windows desktop shortcut (portable install)

Package a release build into `dist\GuestCheckin\` and create/update a Desktop shortcut:

```powershell
.\build.ps1
```

This will:

1. Run `cargo build --release`
2. Copy `guest-checkin.exe`, `config.toml`, and `service_account_key.json` into `dist\GuestCheckin\`
3. Write `run-guest-checkin.cmd` (runs the exe, then `pause` so the window stays open)
4. Create a **GuestCheckin** shortcut on your Desktop pointing at that launcher

Double-click the Desktop icon to run. The console stays open until you press a key.

Optional custom install location:

```powershell
.\build.ps1 -InstallDir "C:\Apps\GuestCheckin"
```

After code changes, run `.\build.ps1` again to refresh the exe and shortcut.
