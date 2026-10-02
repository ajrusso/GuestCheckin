# Smoke testing (TWS211)

Until a production robotic account is approved, live SOAP smoke uses the **PCR test facility (TWS211)** only — not production `ws_uby`.

## Rule: no real PII

CheckIn certification / Operating Rules require **synthetic/fictitious** guests — no real personal data on the test WS (`synthetic/fictitious` checklist in CheckIn `docs/certification.md`).

Do **not** point soap smoke at a live guest register. Use:

- CheckIn synthetic submit (`POST /api/ubyport/submit` or `pilot:submit` with made-up JSON), and/or  
- A **throwaway Google Sheet tab** (or spreadsheet) filled with fake rows, via GuestCheckin `config.smoke.toml`

Encrypting real Sheet fields does **not** satisfy the rule.

## Path A — CheckIn API only (no Sheets)

1. Configure CheckIn `.env` for TWS211 + `UBYPORT_PILOT_TOKEN` + encryption key.
2. Start CheckIn; `GET /api/ubyport/health`.
3. Submit fictitious guests via `npm run pilot:submit` or `POST /api/ubyport/submit-guests`.

**Pass:** per-guest results; encrypted receipt under `pilot-runs/`; decrypt works; no full travel-doc numbers in logs.

## Path B — GuestCheckin soap agent

1. Create a smoke tab with the same columns as the form; enter **fake** guests only.
2. Share the spreadsheet with the service account as **Editor**.
3. Copy `src/config/config.smoke.toml.example` → `config.smoke.toml` (see [configuration.md](./configuration.md)).
4. Set that listing’s `google_spreadsheet_id` / `google_sheet_name` to the smoke tab.
5. Point `[checkin]` at a running CheckIn (TWS211).
6. Run with `--config config.smoke.toml` or `GUESTCHECKIN_CONFIG`.

**Pass:** accepted rows get column M = TRUE; rejects stay unmarked; SES summary has outcome tables, **no UNL/PDF attachments**, no passport numbers; PDFs only under CheckIn `pilot-runs/`.

Use check-in/out dates PCR accepts on TWS211 (invalid check-in → chyby **101**). Prefer today / near-term stay dates for smoke rows.

## After robotic account (#66)

Same flows against production `ws_uby` / real facility credentials — only when intentionally going live.

## Agent notes from first smoke

- Blank column M cells are omitted by the Sheets API; GuestCheckin treats a fully empty M column (or trailing blanks) as unregistered.
- CheckIn PARTIAL_SUCCESS / validation failures put per-guest esults on envelope `meta`; the agent reads those to mark column M only for `accepted`.

## Docker smoke

Build with `docker compose build`, mount `config.smoke.toml` (with `base_url = http://host.docker.internal:5000` on Desktop), then `docker compose run --rm guestcheckin-agent`.

