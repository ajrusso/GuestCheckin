"""Populate SmokeTest with fictitious guests; copy headers only from Form Responses 1."""
from pathlib import Path

from google.oauth2 import service_account
from googleapiclient.discovery import build

SPREADSHEET_ID = "1DFAr_PyXNE6-fU8QkItAQ7Xo_3Dfk36gOeuQfUy14MY"
SOURCE_SHEET = "Form Responses 1"
TARGET_SHEET = "SmokeTest"
KEY = Path(r"C:\Users\ajrus\projects\GuestCheckin\dist\GuestCheckin\service_account_key.json")
SCOPES = ["https://www.googleapis.com/auth/spreadsheets"]


def main() -> None:
    creds = service_account.Credentials.from_service_account_file(KEY, scopes=SCOPES)
    svc = build("sheets", "v4", credentials=creds)
    sheets = svc.spreadsheets()

    meta = sheets.get(spreadsheetId=SPREADSHEET_ID).execute()
    titles = {s["properties"]["title"]: s["properties"]["sheetId"] for s in meta.get("sheets", [])}
    print("tabs:", sorted(titles))

    header_resp = (
        sheets.values()
        .get(spreadsheetId=SPREADSHEET_ID, range=f"'{SOURCE_SHEET}'!1:1")
        .execute()
    )
    headers = list(header_resp.get("values", [[]])[0])
    print("header_cols:", len(headers))
    print("headers:", headers)

    if TARGET_SHEET not in titles:
        sheets.batchUpdate(
            spreadsheetId=SPREADSHEET_ID,
            body={"requests": [{"addSheet": {"properties": {"title": TARGET_SHEET}}}]},
        ).execute()
        print("created sheet", TARGET_SHEET)
    else:
        print("sheet exists", TARGET_SHEET)

    while len(headers) < 13:
        headers.append("")
    if len(headers) >= 13 and not str(headers[12]).strip():
        headers[12] = "Registered With Authorities"
    elif len(headers) == 12:
        headers.append("Registered With Authorities")

    # Fictitious guests only — never copy real Form Responses body rows.
    fake_rows = [
        [
            "02.10.2026 10:00:00",
            "01 Tourism",
            "01.10.2026",
            "05.10.2026",
            "Testovic",
            "Jan",
            "15.03.1990",
            "SVK Slovakia",
            "SMOKEDOC001",
            "",
            "Bratislava Fake Street 1",
            "Jan Testovic",
            "",
        ],
        [
            "02.10.2026 10:05:00",
            "01 Tourism",
            "02.10.2026",
            "06.10.2026",
            "Fiktivna",
            "Eva",
            "20.07.1988",
            "CZE Czechia",
            "SMOKEDOC002",
            "",
            "Praha Fake Address 2",
            "Eva Fiktivna",
            "",
        ],
        [
            "02.10.2026 10:10:00",
            "01 Tourism",
            "03.10.2026",
            "07.10.2026",
            "Dummy",
            "Petr",
            "01.01.1985",
            "POL Poland",
            "SMOKEDOC003",
            "VISAFAKE01",
            "Warsaw Test 3",
            "Petr Dummy",
            "",
        ],
    ]

    width = len(headers)
    values = [headers[:width]]
    for row in fake_rows:
        r = list(row)
        if len(r) < width:
            r.extend([""] * (width - len(r)))
        values.append(r[:width])

    sheets.values().clear(spreadsheetId=SPREADSHEET_ID, range=f"'{TARGET_SHEET}'").execute()
    sheets.values().update(
        spreadsheetId=SPREADSHEET_ID,
        range=f"'{TARGET_SHEET}'!A1",
        valueInputOption="USER_ENTERED",
        body={"values": values},
    ).execute()

    print(f"wrote {len(fake_rows)} fictitious guests to {TARGET_SHEET}")
    print("surnames:", [r[4] for r in fake_rows])


if __name__ == "__main__":
    main()
