use log::{info, warn};
use rusoto_core::{Region, HttpClient};
use rusoto_credential::StaticProvider;
use rusoto_sesv2::{Destination, EmailContent, RawMessage, SendEmailRequest, SesV2, SesV2Client};
use std::{fs::File, str::FromStr};
use std::io::Read;
use base64::encode;


pub struct Email {
    attachments: Vec<String>,
    from: String,
    to: Vec<String>,
    subject: String,
    credentials: StaticProvider,
    region: Region,
}

impl Email {
    pub fn new(
        attachments: Vec<String>,
        from: String,
        to: Vec<String>,
        subject: &str,
        access_key: &str,
        secret_key: &str,
        region: &str,
    ) -> Self {
        
        let region_object = match Region::from_str(region) {
            Ok(region) => region,
            Err(_) => panic!("Improper AWS region name given"),
        };

        let credentials_provider = StaticProvider::new_minimal(
            access_key.to_string(),
            secret_key.to_string()
        );

        Self {
            attachments: attachments,
            from: from,
            to: to,
            subject: subject.to_string(),
            credentials: credentials_provider,
            region: region_object,
        }
    }

    pub async fn send(&self, unregistered_guests: Vec<Vec<String>>, checkin_issues: Vec<Vec<String>>) {

        // Create the raw email message with multiple attachments
        let mut recipients = String::new();
        for recipient in &self.to {
            recipients.push_str(&format!("{}, ", &recipient));
        }

        // Unregistered Guests Table
        let mut table_data: Vec<Vec<String>> = Vec::new();
        table_data.push(vec!["Listing".to_string(), "Row".to_string(), "Fullname".to_string(), "Check In".to_string(), "Check Out".to_string()]);
        for guest in unregistered_guests {
            table_data.push(guest);
        }

        // Generate HTML unregistered guests table rows
        let mut unreg_guests_table_rows = String::new();
        for row in table_data {
            unreg_guests_table_rows.push_str("<tr>");
            for cell in row {
                unreg_guests_table_rows.push_str(&format!("<td>{}</td>", cell));
            }
            unreg_guests_table_rows.push_str("</tr>");
        }

        // Guests with checkin issues
        let mut table_data: Vec<Vec<String>> = Vec::new();
        table_data.push(vec!["Listing".to_string(), "Row".to_string(), "Fullname".to_string(), "Input Error(s)".to_string()]);
        for guest in checkin_issues {
            table_data.push(guest);
        }

        // Generate HTML checkin issues table rows
        let mut checkin_issues_table_rows = String::new();
        for row in table_data {
            checkin_issues_table_rows.push_str("<tr>");
            for cell in row {
                checkin_issues_table_rows.push_str(&format!("<td>{}</td>", cell));
            }
            checkin_issues_table_rows.push_str("</tr>");
        }

        let inline_image_path = "src/header_image.jpg";

        // HTML content with tables and an image
        let html_content = format!(
            r#"
            <html>
            <body>
                <img src="cid:header_image.jpg" alt="Image" style="width:100%; max-width:600px;">
                <br>
                <br>
                <h2 style="color: #1E90FF;">Guests Available for Checkin</h2>
                <table border="1">
                    {}
                </table>
                <br>
                <h2 style="color: #1E90FF;">Guests with Checkin Issues</h2>
                <table border="1">
                    {}
                </table>
            </body>
            </html>
            "#,
            unreg_guests_table_rows, checkin_issues_table_rows
        );

        // Load the inline image file
        let mut image_file = File::open(inline_image_path).expect("Unable to open image file");
        let mut image_content = Vec::new();
        image_file.read_to_end(&mut image_content).expect("Unable to read image file");
        let encoded_image = encode(&image_content);

        let mut raw_email = String::new();
        raw_email.push_str(
            &format!(
                "From: {}\r\n\
                To: {}\r\n\
                Subject: {}\r\n\
                MIME-Version: 1.0\r\n\
                Content-Type: multipart/mixed; boundary=\"boundary\"\r\n\r\n\
                --boundary\r\n\
                Content-Type: multipart/alternative; boundary=\"subboundary\"\r\n\r\n\
                --subboundary\r\n\
                Content-Type: text/plain; charset=\"UTF-8\"\r\n\
                Content-Transfer-Encoding: 7bit\r\n\r\n\
                This is the plain text version of the email.\r\n\r\n\
                --subboundary\r\n\
                Content-Type: text/html; charset=\"UTF-8\"\r\n\
                Content-Transfer-Encoding: 7bit\r\n\r\n\
                {}\r\n\r\n\
                --subboundary--\r\n\
                --boundary\r\n\
                Content-Type: image/jpeg; name=\"header_image.jpg\"\r\n\
                Content-Transfer-Encoding: base64\r\n\
                Content-Disposition: inline; filename=\"header_image.jpg\"\r\n\
                Content-ID: <header_image.jpg>\r\n\r\n\
                {}\r\n",
                self.from, recipients, self.subject, html_content, encoded_image
            )
        );

        for attachment in &self.attachments {
            
            // Load the attachment file
            info!("Attaching file {} to email", attachment);
            let mut file = File::open(attachment).expect("Unable to open file");
            let mut file_content = Vec::new();
            file.read_to_end(&mut file_content).expect("Unable to read file");
            let encoded_file = encode(&file_content);

            // Get the file name from the path
            let file_name = attachment.split('/').last().unwrap_or("attachment");

            // Append attachment part
            raw_email.push_str(&format!(
                "--boundary\r\n\
                Content-Type: application/octet-stream; name=\"{}\"\r\n\
                Content-Transfer-Encoding: base64\r\n\
                Content-Disposition: attachment; filename=\"{}\"\r\n\r\n\
                {}\r\n",
                file_name, file_name, encoded_file
            ));
        }

        // Close the MIME boundary
        raw_email.push_str("--boundary--");

        // Create the SES client
        let client = SesV2Client::new_with(
            HttpClient::new().expect("Failed to create HTTP client"),
            self.credentials.clone(),
            self.region.clone(),
        );

        let request = SendEmailRequest {
            from_email_address: Some(self.from.clone()),
            destination: Some(Destination {
                to_addresses: Some(self.to.clone()),
                cc_addresses: None,
                bcc_addresses: None,
            }),
            content: EmailContent {
                raw: Some(RawMessage {
                    data: raw_email.into_bytes().into(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        // Send the email
        match client.send_email(request).await {
            Ok(_) => info!("Email sent successfully!"),
            Err(e) => warn!("Error sending email: {:?}", e),
        }
    }
}
