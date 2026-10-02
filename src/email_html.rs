//! Pure HTML builders for SES emails (testable; no passport numbers in soap mode).

/// Build HTML body for classic UNL mode (guests available + check-in issues).
pub fn build_unl_email_html(
    image_html: &str,
    unreg_guests: &[Vec<String>],
    checkin_issues: &[Vec<String>],
) -> String {
    let mut unreg_rows = String::new();
    unreg_rows.push_str("<tr><td>Listing</td><td>Row</td><td>Fullname</td><td>Check In</td><td>Check Out</td></tr>");
    for row in unreg_guests {
        unreg_rows.push_str(&html_table_row(row));
    }

    let mut issue_rows = String::new();
    issue_rows.push_str("<tr><td>Listing</td><td>Row</td><td>Fullname</td><td>Input Error(s)</td></tr>");
    for row in checkin_issues {
        issue_rows.push_str(&html_table_row(row));
    }

    format!(
        r#"
            <html>
            <body>
                {image_html}
                <h2 style="color: #1E90FF;">Guests Available for Checkin</h2>
                <table border="1">
                    {unreg_rows}
                </table>
                <br>
                <h2 style="color: #1E90FF;">Guests with Checkin Issues</h2>
                <table border="1">
                    {issue_rows}
                </table>
            </body>
            </html>
            "#
    )
}

/// Build HTML body for soap / CheckIn submit outcomes (no UNL, no travel doc numbers).
pub fn build_soap_email_html(
    image_html: &str,
    accepted: &[Vec<String>],
    rejected: &[Vec<String>],
    checkin_issues: &[Vec<String>],
) -> String {
    let mut accepted_rows = String::new();
    accepted_rows.push_str(
        "<tr><td>Listing</td><td>Row</td><td>Fullname</td><td>Check In</td><td>Check Out</td><td>Status</td></tr>",
    );
    for row in accepted {
        accepted_rows.push_str(&html_table_row(row));
    }

    let mut rejected_rows = String::new();
    rejected_rows.push_str(
        "<tr><td>Listing</td><td>Row</td><td>Fullname</td><td>Status</td><td>Codes</td></tr>",
    );
    for row in rejected {
        rejected_rows.push_str(&html_table_row(row));
    }

    let mut issue_rows = String::new();
    issue_rows.push_str("<tr><td>Listing</td><td>Row</td><td>Fullname</td><td>Input Error(s)</td></tr>");
    for row in checkin_issues {
        issue_rows.push_str(&html_table_row(row));
    }

    format!(
        r#"
            <html>
            <body>
                {image_html}
                <h2 style="color: #1E90FF;">Accepted (UbyPort / CheckIn)</h2>
                <table border="1">
                    {accepted_rows}
                </table>
                <br>
                <h2 style="color: #1E90FF;">Rejected / Duplicate</h2>
                <table border="1">
                    {rejected_rows}
                </table>
                <br>
                <h2 style="color: #1E90FF;">Guests with Checkin Issues</h2>
                <table border="1">
                    {issue_rows}
                </table>
            </body>
            </html>
            "#
    )
}

fn html_table_row(cells: &[String]) -> String {
    let mut row = String::from("<tr>");
    for cell in cells {
        row.push_str(&format!("<td>{}</td>", html_escape(cell)));
    }
    row.push_str("</tr>");
    row
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soap_email_has_no_passport_numbers() {
        let html = build_soap_email_html(
            "",
            &[vec![
                "Blue Glory".into(),
                "12".into(),
                "Jan Novak".into(),
                "01.10.2026".into(),
                "05.10.2026".into(),
                "accepted".into(),
            ]],
            &[vec![
                "Blue Glory".into(),
                "13".into(),
                "Eva Novak".into(),
                "rejected".into(),
                "105".into(),
            ]],
            &[],
        );
        assert!(html.contains("Accepted (UbyPort / CheckIn)"));
        assert!(html.contains("Rejected / Duplicate"));
        assert!(!html.contains("AB1234567"));
        assert!(!html.to_lowercase().contains("passport"));
    }
}
