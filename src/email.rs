use log::{info, warn};
use rusoto_core::{HttpClient, Region};
use rusoto_credential::StaticProvider;
use rusoto_sesv2::{Destination, EmailContent, RawMessage, SendEmailRequest, SesV2, SesV2Client};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

use guest_checkin::email_html::{build_soap_email_html, build_unl_email_html};
use base64::encode;

fn resolve_header_image_path() -> Option<&'static str> {
    // Portable install copies the image next to the exe; repo/dev keeps it under src/.
    const CANDIDATES: &[&str] = &["header_image.jpg", "src/header_image.jpg"];
    CANDIDATES.into_iter().copied().find(|path| Path::new(path).exists())
}

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

        let credentials_provider =
            StaticProvider::new_minimal(access_key.to_string(), secret_key.to_string());

        Self {
            attachments,
            from,
            to,
            subject: subject.to_string(),
            credentials: credentials_provider,
            region: region_object,
        }
    }

    pub async fn send(
        &self,
        unregistered_guests: Vec<Vec<String>>,
        checkin_issues: Vec<Vec<String>>,
    ) {
        let header_image = load_header_image();
        let image_html = header_image_html(header_image.is_some());
        let html_content = build_unl_email_html(&image_html, &unregistered_guests, &checkin_issues);
        self.send_raw_html(html_content, header_image, true).await;
    }

    /// Soap / CheckIn mode: outcome tables, **no UNL/PDF attachments**, no travel doc numbers in rows.
    pub async fn send_soap_summary(
        &self,
        accepted: Vec<Vec<String>>,
        rejected: Vec<Vec<String>>,
        checkin_issues: Vec<Vec<String>>,
    ) {
        let header_image = load_header_image();
        let image_html = header_image_html(header_image.is_some());
        let html_content = build_soap_email_html(&image_html, &accepted, &rejected, &checkin_issues);
        self.send_raw_html(html_content, header_image, false).await;
    }

    async fn send_raw_html(
        &self,
        html_content: String,
        header_image: Option<String>,
        include_attachments: bool,
    ) {
        let mut recipients = String::new();
        for recipient in &self.to {
            recipients.push_str(&format!("{}, ", &recipient));
        }

        let mut raw_email = String::new();
        raw_email.push_str(&format!(
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
                --subboundary--\r\n",
            self.from, recipients, self.subject, html_content
        ));

        if let Some(encoded_image) = header_image {
            raw_email.push_str(&format!(
                "--boundary\r\n\
                Content-Type: image/jpeg; name=\"header_image.jpg\"\r\n\
                Content-Transfer-Encoding: base64\r\n\
                Content-Disposition: inline; filename=\"header_image.jpg\"\r\n\
                Content-ID: <header_image.jpg>\r\n\r\n\
                {}\r\n",
                encoded_image
            ));
        }

        if include_attachments {
            for attachment in &self.attachments {
                info!("Attaching file {} to email", attachment);
                let mut file = match File::open(attachment) {
                    Ok(file) => file,
                    Err(e) => {
                        warn!("Unable to open attachment {}: {}", attachment, e);
                        continue;
                    }
                };
                let mut file_content = Vec::new();
                if let Err(e) = file.read_to_end(&mut file_content) {
                    warn!("Unable to read attachment {}: {}", attachment, e);
                    continue;
                }
                let encoded_file = encode(&file_content);

                let file_name = Path::new(attachment)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("attachment");

                raw_email.push_str(&format!(
                    "--boundary\r\n\
                Content-Type: application/octet-stream; name=\"{}\"\r\n\
                Content-Transfer-Encoding: base64\r\n\
                Content-Disposition: attachment; filename=\"{}\"\r\n\r\n\
                {}\r\n",
                    file_name, file_name, encoded_file
                ));
            }
        }

        raw_email.push_str("--boundary--");

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

        match client.send_email(request).await {
            Ok(_) => info!("Email sent successfully!"),
            Err(e) => warn!("Error sending email: {:?}", e),
        }
    }
}

fn header_image_html(has_image: bool) -> String {
    if has_image {
        r#"<img src="cid:header_image.jpg" alt="Image" style="width:100%; max-width:600px;"><br><br>"#
            .to_string()
    } else {
        String::new()
    }
}

fn load_header_image() -> Option<String> {
    let header_image = resolve_header_image_path().and_then(|path| match File::open(path) {
        Ok(mut image_file) => {
            let mut image_content = Vec::new();
            match image_file.read_to_end(&mut image_content) {
                Ok(_) => {
                    info!("Using email header image {}", path);
                    Some(encode(&image_content))
                }
                Err(e) => {
                    warn!("Unable to read header image {}: {}", path, e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Unable to open header image {}: {}", path, e);
            None
        }
    });
    if header_image.is_none() {
        warn!(
            "Email header image not found (looked for header_image.jpg and src/header_image.jpg); sending without it"
        );
    }
    header_image
}
