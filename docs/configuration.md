# Configuration

GuestCheckin reads a single TOML file per run. Secrets and live listing IDs stay out of git (`config.toml`, `config.smoke.toml`, service account key are gitignored / under `dist/`).

## Load order

1. `--config path\to\file.toml` (or `--config=path`)
2. Environment variable `GUESTCHECKIN_CONFIG`
3. `./config.toml` — portable install (`dist\GuestCheckin\`)
4. `src/config/config.toml` — repo-local development

The process logs `Loading config from …` at startup so you can confirm which file was used.

Implementation: `src/settings.rs` (`resolve_config_path`).

## Profiles

| File | Purpose | In git? |
|------|---------|---------|
| `config.toml` | Everyday / UNL agent (portable dist) | No (local / dist only) |
| `config.smoke.toml` | Soap + TWS211 smoke (fake Sheet tab) | No (gitignored) |
| `src/config/config.toml.example` | Template for normal config | Yes |
| `src/config/config.smoke.toml.example` | Template for smoke profile | Yes |

### Normal (UNL) profile

Copy `src/config/config.toml.example` → `config.toml` (or use `dist\GuestCheckin\config.toml` from `build.ps1`).

Default `submit_mode` is `unl` when omitted.

### Smoke / soap profile

Copy `src/config/config.smoke.toml.example` → `config.smoke.toml` next to the exe (e.g. `dist\GuestCheckin\`).

Required for soap:

```toml
submit_mode = "soap"

[checkin]
base_url = "http://127.0.0.1:5000"
pilot_token = "…"   # must match CheckIn UBYPORT_PILOT_TOKEN
```

Use **one** `[[listing]]` pointed at a Google Sheet tab that contains **fictitious guests only** (PCR Operating Rules / CheckIn `docs/certification.md`: no real PII on the test facility).

Separate scan/log files in the smoke example (`scan_state.smoke.json`, `output.smoke.log`) so smoke runs do not clobber the UNL agent cursor.

### Running smoke without touching prod config

```powershell
cd dist\GuestCheckin
$env:GUESTCHECKIN_CONFIG = "config.smoke.toml"
.\guest-checkin.exe
# or:
.\guest-checkin.exe --config config.smoke.toml
```

Optional: copy `scripts/run-smoke.cmd.example` beside the exe as `run-smoke.cmd`.

## Related settings

| Key | Notes |
|-----|--------|
| `submit_mode` | `unl` (default) or `soap` |
| `checkin.base_url` / `checkin.pilot_token` | Required when `submit_mode = "soap"` |
| `scan_state_filepath` | Watermark / pending rows JSON |
| `service_account_key_filepath` | Google SA JSON; must be Editor on the spreadsheet |
| `[SES].to` | Distro list (same template; soap mode sends **no** UNL/PDF attachments) |

See also [smoke-testing.md](./smoke-testing.md) and the CheckIn Blue Glory agent notes in CheckIn `docs/runbook.md`.
