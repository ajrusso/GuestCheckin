//! HTTP client for CheckIn pilot `submit-guests` (POC / #68).

use log::{info, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde_json::json;

use crate::guest_input::GuestInput;

#[derive(Debug, Clone)]
pub struct CheckInClient {
    base_url: String,
    pilot_token: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GuestSubmitResult {
    pub index: usize,
    pub travel_document_number_redacted: String,
    pub status: String,
    #[serde(default)]
    pub chyby_codes: Vec<i64>,
    pub pseudo_razitko: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitGuestsData {
    pub client_batch_id: Option<String>,
    pub batch_count: Option<u32>,
    pub results: Vec<GuestSubmitResult>,
    #[serde(default)]
    pub receipt_ids: Vec<String>,
    pub has_guest_record_errors: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct SubmitGuestsOutcome {
    pub http_status: u16,
    pub ok: bool,
    pub data: Option<SubmitGuestsData>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

impl CheckInClient {
    pub fn new(base_url: &str, pilot_token: &str) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {e}"))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            pilot_token: pilot_token.to_string(),
            http,
        })
    }

    pub async fn submit_guests(
        &self,
        client_batch_id: &str,
        guests: &[GuestInput],
    ) -> Result<SubmitGuestsOutcome, String> {
        let url = format!("{}/api/ubyport/submit-guests", self.base_url);
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Pilot-Token",
            HeaderValue::from_str(&self.pilot_token)
                .map_err(|e| format!("Invalid pilot token header: {e}"))?,
        );

        let body = json!({
            "clientBatchId": client_batch_id,
            "guests": guests,
        });

        info!(
            "Submitting {} guest(s) to CheckIn (batchId={})",
            guests.len(),
            client_batch_id
        );

        let response = self
            .http
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("CheckIn submit-guests request failed: {e}"))?;

        let http_status = response.status().as_u16();
        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("CheckIn response JSON parse failed: {e}"))?;

        let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        let mut data = value
            .get("data")
            .cloned()
            .and_then(|d| serde_json::from_value::<SubmitGuestsData>(d).ok());

        // Partial / validation failures put per-guest results on `meta`, not `data`.
        if data.as_ref().map(|d| d.results.is_empty()).unwrap_or(true) {
            if let Some(meta) = value.get("meta") {
                if let Some(results) = meta.get("results") {
                    if let Ok(parsed) = serde_json::from_value::<Vec<GuestSubmitResult>>(results.clone())
                    {
                        if !parsed.is_empty() {
                            data = Some(SubmitGuestsData {
                                client_batch_id: meta
                                    .get("clientBatchId")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                batch_count: meta
                                    .get("batchCount")
                                    .and_then(|v| v.as_u64())
                                    .map(|n| n as u32),
                                results: parsed,
                                receipt_ids: meta
                                    .get("receiptIds")
                                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                                    .unwrap_or_default(),
                                has_guest_record_errors: Some(true),
                            });
                        }
                    }
                }
            }
        }
        let error_code = value
            .pointer("/error/code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let error_message = value
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if !ok {
            warn!(
                "CheckIn submit-guests returned ok=false status={} code={:?} msg={:?}",
                http_status, error_code, error_message
            );
        }

        Ok(SubmitGuestsOutcome {
            http_status,
            ok,
            data,
            error_code,
            error_message,
        })
    }
}

/// Whether Sheet column M should be marked for this per-guest status.
pub fn should_mark_column_m(status: &str) -> bool {
    status.eq_ignore_ascii_case("accepted")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_accepted_marks_column_m() {
        assert!(should_mark_column_m("accepted"));
        assert!(should_mark_column_m("Accepted"));
        assert!(!should_mark_column_m("rejected"));
        assert!(!should_mark_column_m("duplicate"));
        assert!(!should_mark_column_m(""));
    }
}
