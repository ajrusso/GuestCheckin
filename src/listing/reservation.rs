use std::collections::{HashMap, HashSet};

use google_sheets4::oauth2::{read_service_account_key, ServiceAccountAuthenticator};
use google_sheets4::{hyper::client::HttpConnector, hyper_rustls::HttpsConnector};
use google_sheets4::{Sheets, hyper, hyper_rustls};
use google_sheets4::api::ValueRange;
use serde_json::json;
use log::{debug, info, warn, error};
use tokio::time::sleep;

use crate::listing::guest::Guest;
use guest_checkin::transliteration::decode_html_entities;

const SHEETS_RETRY_ATTEMPTS: u32 = 5;
const SHEETS_RETRY_BASE_MS: u64 = 1000;
const SHEETS_QUOTA_RETRY_SECS: u64 = 60;
const BATCH_GET_CHUNK_SIZE: usize = 50;

fn is_retryable_sheets_error(err: &str) -> bool {
    err.contains("503")
        || err.contains("UNAVAILABLE")
        || err.contains("429")
        || err.contains("RATE_LIMIT_EXCEEDED")
        || err.contains("rateLimit")
        || err.contains("quota")
        || err.contains("backendError")
}

fn retry_delay(err: &str, attempt: u32) -> std::time::Duration {
    if err.contains("RATE_LIMIT_EXCEEDED") || err.contains("quota") {
        std::time::Duration::from_secs(SHEETS_QUOTA_RETRY_SECS)
    } else {
        std::time::Duration::from_millis(SHEETS_RETRY_BASE_MS * 2u64.pow(attempt.saturating_sub(1)))
    }
}


#[derive(Clone)]
pub struct Reservation {
    spreadsheet_id: String,
    sheet_name: String,
    hub: Option<Sheets<HttpsConnector<HttpConnector>>>,
}

/// Result of scanning for unregistered guests, including cursor updates for the caller.
pub struct UnregisteredScanResult {
    pub guests: Vec<Guest>,
    pub last_seen_row: u32,
    /// All candidate row numbers considered this run (pending ∪ newly unregistered).
    pub candidate_rows: Vec<u32>,
}

impl Reservation {
    pub async fn new(spreadsheet_id: &str, sheet_name: &str, service_account_key_filepath: &str) -> Self {
        let mut res = Reservation {
            spreadsheet_id: spreadsheet_id.to_string(),
            sheet_name: sheet_name.to_string(),
            hub: None,
        };
        res.set_hub(service_account_key_filepath).await;
        res
    }

    async fn set_hub(&mut self, service_account_key_path: &str) {
        
        // Load service account key file
        let service_account_key = read_service_account_key(service_account_key_path)
            .await
            .expect("Failed to read service account key");
        
        // Create the authenticator
        let auth = ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await.expect("Failed to create authenticator");


        // Create the Sheets API client
        self.hub = Some(Sheets::new (
            hyper::Client::builder()
                .build(hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .unwrap()
                .https_or_http()
                .enable_http1()
                .build()), 
            auth.clone(),
        ));
    }

    // Update row (Guest) "Registered With Authorities" in spreadsheet 
    pub async fn update_registered_with_authorities(&self, row: &str, first_name: &str, last_name: &str) {
        let mut req = ValueRange::default();
        let range = format!("{}!M{}", self.sheet_name, row);
        req.range = Some(range.clone());
        req.values = Some(vec![vec![json!("TRUE")]]);

        let result = self.hub.clone()
            .unwrap()
            .spreadsheets()
            .values_update(req, &self.spreadsheet_id, &range)
            .value_input_option("RAW")
            .doit()
            .await;

        match result {
            Ok(response) => {
                debug!("{:?}", response.1);
                info!("Updated  {} {} on row {} col 'Registered With Authorities'", first_name, last_name, row );
            },
            Err(e) => error!("Updating registration status for guest {} {} on row {}: {}", first_name, last_name, row, e.to_string()),
        }
    }

    // Finds guests in Google Spreadsheet that have not been registered in Ubyport
    pub async fn find_unregistered_guests(
        &self,
        pending_rows: &[u32],
        last_seen_row: u32,
    ) -> UnregisteredScanResult {
        let (new_unregistered, new_last_seen_row) =
            self.discover_unregistered_rows(last_seen_row).await;

        let mut candidate_set: HashSet<u32> = pending_rows.iter().copied().collect();
        for row in &new_unregistered {
            candidate_set.insert(*row);
        }

        let mut unregistered_guest_row_nums: Vec<u32> = candidate_set.into_iter().collect();
        unregistered_guest_row_nums.sort_unstable();

        info!(
            "Candidate unregistered rows (pending={}, new={}): {:?}",
            pending_rows.len(),
            new_unregistered.len(),
            unregistered_guest_row_nums
        );

        let unregistered_guest_rows = match &self.hub {
            Some(_hub) => {
                self.get_guest_rows_response(&unregistered_guest_row_nums)
                    .await
            }
            None => panic!("No Google hub found"),
        };

        let mut unregistered_guests = Vec::new();

        for (idx, unregistered_guest) in unregistered_guest_rows.iter().enumerate() {
            let mut guest_hash: HashMap<String, String> = HashMap::new();
            let row_num = format!("{}", unregistered_guest_row_nums[idx]);

            match &unregistered_guest.values {
                Some(row) => {
                    for col in row {
                        let mut i = 1;
                        for val in col {
                            match i {
                                1  => guest_hash.insert("timestamp".to_string(), val.to_string().trim_matches('"').to_string()),
                                2  => guest_hash.insert("purpose_of_stay".to_string(), val.to_string().trim_matches('"')[0..2].to_string()),
                                3  => guest_hash.insert("check_in".to_string(), val.to_string().trim_matches('"').to_string()),
                                4  => guest_hash.insert("check_out".to_string(), val.to_string().trim_matches('"').to_string()),
                                5  => guest_hash.insert("surname".to_string(), val.to_string().trim_matches('"').to_string()),
                                6  => guest_hash.insert("first_name".to_string(), val.to_string().trim_matches('"').to_string()),
                                7  => guest_hash.insert("birth_date".to_string(), val.to_string().trim_matches('"').to_string()),
                                8  => guest_hash.insert("country_of_citizenship".to_string(), val.to_string().trim_matches('"')[0..3].to_string()),
                                9  => guest_hash.insert("travel_doc_number".to_string(), val.to_string().trim_matches('"').to_string()),
                                10 => guest_hash.insert("visa_number".to_string(), val.to_string().trim_matches('"').to_string()),
                                11 => guest_hash.insert("address_abroad".to_string(), val.to_string().trim_matches('"').to_string()),
                                12 => guest_hash.insert("full_name".to_string(), val.to_string().trim_matches('"').to_string()),
                                _ => break,
                            };
                            i += 1;
                        }

                        let guest = Guest::new(
                            &row_num,
                            cell_string(&guest_hash, "timestamp"),
                            cell_string(&guest_hash, "purpose_of_stay"),
                            cell_string(&guest_hash, "check_in"),
                            cell_string(&guest_hash, "check_out"),
                            cell_string(&guest_hash, "surname"),
                            cell_string(&guest_hash, "first_name"),
                            cell_string(&guest_hash, "birth_date"),
                            cell_string(&guest_hash, "country_of_citizenship"),
                            cell_string(&guest_hash, "travel_doc_number"),
                            cell_string(&guest_hash, "visa_number"),
                            cell_string(&guest_hash, "address_abroad"),
                            cell_string(&guest_hash, "full_name"),
                        );

                        debug!("Found unregistered guest: {}", guest);

                        unregistered_guests.push(guest.clone());

                        if !guest.data_errors.is_empty() {
                            warn!(
                                "Unregistered guest {} {} can not be registered: {}",
                                guest.first_name,
                                guest.surname,
                                guest.get_data_errors()
                            );
                        }
                    }
                }
                None => warn!("Empty guest row found"),
            }
        }

        UnregisteredScanResult {
            guests: unregistered_guests,
            last_seen_row: new_last_seen_row,
            candidate_rows: unregistered_guest_row_nums,
        }
    }

    /// Discover unregistered row numbers and the highest sheet row observed.
    ///
    /// First run (`last_seen_row == 0`): scan Column M from row 2 and use Column A for extent.
    /// Later runs: only look at rows after the watermark; new form rows are candidates,
    /// with Column M used to skip any already marked TRUE.
    async fn discover_unregistered_rows(&self, last_seen_row: u32) -> (Vec<u32>, u32) {
        let start_row = if last_seen_row < 2 {
            2
        } else {
            last_seen_row + 1
        };

        // Column A always has form timestamps; use it to advance the watermark
        // even when Column M is still empty for new responses.
        let a_sheet_range = format!("{}!A{}:A", self.sheet_name, start_row);
        let a_response = match self.sheets_values_get_with_retry(&a_sheet_range).await {
            Ok(value_range) => value_range,
            Err(e) => {
                error!(
                    "Google Sheets unavailable for spreadsheet {} sheet {} (col A): {}",
                    self.spreadsheet_id, self.sheet_name, e
                );
                return (Vec::new(), last_seen_row);
            }
        };

        let a_row_count = a_response
            .values
            .as_ref()
            .map(|rows| rows.len() as u32)
            .unwrap_or(0);
        let new_last_seen = if a_row_count == 0 {
            last_seen_row.max(start_row.saturating_sub(1))
        } else {
            start_row + a_row_count - 1
        };

        if last_seen_row < 2 {
            // Bootstrap: full Column M scan for currently unregistered guests
            let m_response = self.get_registration_column(2).await;
            let unregistered = Self::get_unregistered_guests(m_response, 2);
            let last_seen = new_last_seen.max(
                unregistered.iter().copied().max().unwrap_or(0),
            );
            info!(
                "Bootstrap scan: {} unregistered guests, last_seen_row={}",
                unregistered.len(),
                last_seen
            );
            return (unregistered, last_seen);
        }

        if a_row_count == 0 {
            info!("No new guest rows after row {}", last_seen_row);
            return (Vec::new(), new_last_seen);
        }

        // New rows are unregistered unless Column M is already TRUE
        let registered_in_new_range =
            self.registered_rows_in_range(start_row, new_last_seen).await;
        let mut unregistered = Vec::new();
        for row in start_row..=new_last_seen {
            if !registered_in_new_range.contains(&row) {
                unregistered.push(row);
            }
        }

        info!(
            "Incremental scan from row {}: {} new unregistered, last_seen_row={}",
            start_row,
            unregistered.len(),
            new_last_seen
        );

        (unregistered, new_last_seen)
    }

    async fn get_registration_column(&self, start_row: u32) -> ValueRange {
        let range = format!("!M{}:M", start_row);
        let sheet_range = format!("{}{}", self.sheet_name, range);

        match self.sheets_values_get_with_retry(&sheet_range).await {
            Ok(value_range) => value_range,
            Err(e) => {
                error!(
                    "Google Sheets unavailable for spreadsheet {} sheet {}: {}",
                    self.spreadsheet_id, self.sheet_name, e
                );
                ValueRange::default()
            }
        }
    }

    /// Rows in [start_row, end_row] whose Column M is TRUE (registered).
    async fn registered_rows_in_range(&self, start_row: u32, end_row: u32) -> HashSet<u32> {
        let mut registered = HashSet::new();
        if end_row < start_row {
            return registered;
        }

        let m_response = self.get_registration_column(start_row).await;
        if let Some(rows) = m_response.values {
            let mut row_num = start_row;
            for cell in rows {
                if row_num > end_row {
                    break;
                }
                if let Some(value) = cell.get(0) {
                    if let Some(s) = value.as_str() {
                        if s.eq_ignore_ascii_case("true") || s.to_lowercase().contains("true") {
                            registered.insert(row_num);
                        }
                    }
                }
                row_num += 1;
            }
        }
        registered
    }

    async fn sheets_values_get_with_retry(&self, sheet_range: &str) -> Result<ValueRange, String> {
        let hub = self.hub.clone().ok_or_else(|| "No Google hub found".to_string())?;
        let mut last_err = String::new();

        for attempt in 1..=SHEETS_RETRY_ATTEMPTS {
            let result = hub
                .spreadsheets()
                .values_get(&self.spreadsheet_id, sheet_range)
                .doit()
                .await;

            match result {
                Ok(response) => return Ok(response.1),
                Err(e) => {
                    last_err = e.to_string();

                    if attempt < SHEETS_RETRY_ATTEMPTS && is_retryable_sheets_error(&last_err) {
                        let delay = retry_delay(&last_err, attempt);
                        warn!(
                            "Google Sheets request failed (attempt {}/{}), retrying in {:?}: {}",
                            attempt, SHEETS_RETRY_ATTEMPTS, delay, last_err
                        );
                        sleep(delay).await;
                        continue;
                    }
                    return Err(last_err);
                }
            }
        }

        Err(last_err)
    }

    async fn sheets_values_batch_get_with_retry(
        &self,
        ranges: &[String],
    ) -> Result<Vec<ValueRange>, String> {
        if ranges.is_empty() {
            return Ok(Vec::new());
        }

        let hub = self.hub.clone().ok_or_else(|| "No Google hub found".to_string())?;
        let mut last_err = String::new();

        for attempt in 1..=SHEETS_RETRY_ATTEMPTS {
            let mut call = hub
                .spreadsheets()
                .values_batch_get(&self.spreadsheet_id);
            for range in ranges {
                call = call.add_ranges(range);
            }

            match call.doit().await {
                Ok(response) => {
                    return Ok(response.1.value_ranges.unwrap_or_default());
                }
                Err(e) => {
                    last_err = e.to_string();

                    if attempt < SHEETS_RETRY_ATTEMPTS && is_retryable_sheets_error(&last_err) {
                        let delay = retry_delay(&last_err, attempt);
                        warn!(
                            "Google Sheets batchGet failed (attempt {}/{}), retrying in {:?}: {}",
                            attempt, SHEETS_RETRY_ATTEMPTS, delay, last_err
                        );
                        sleep(delay).await;
                        continue;
                    }
                    return Err(last_err);
                }
            }
        }

        Err(last_err)
    }

    // Checks "Registered With Authorities" column input for unregistered guests
    fn get_unregistered_guests(response: ValueRange, start_row: u32) -> Vec<u32> {
        let mut guests: Vec<u32> = Vec::new();
        let mut i: u32 = start_row;
        let mut unregister_guest_count = 0;
        if let Some(row) = response.values {
            info!("Checking a total of {} guest registration cells from row {}", row.iter().count(), start_row);
            for cell in row {
                match cell.get(0) {
                    Some(value) => {
                        let v = value.as_str().unwrap().to_lowercase();
                        if v.contains("false") {
                            guests.push(i);
                            unregister_guest_count += 1;
                        }
                    }
                    None => {
                        guests.push(i);
                        unregister_guest_count += 1;
                    }
                }
                i += 1;
            }
        }

        info!("{} unregistered guests found in column M scan", unregister_guest_count);

        guests
    }

    // Gets given rows (guests) from Google Spreadsheet via batchGet
    async fn get_guest_rows_response(&self, rows: &[u32]) -> Vec<ValueRange> {
        if rows.is_empty() {
            return Vec::new();
        }

        let mut guest_rows = Vec::with_capacity(rows.len());

        for chunk in rows.chunks(BATCH_GET_CHUNK_SIZE) {
            let ranges: Vec<String> = chunk
                .iter()
                .map(|row| format!("{}!{}:{}", self.sheet_name, row, row))
                .collect();

            match self.sheets_values_batch_get_with_retry(&ranges).await {
                Ok(value_ranges) => {
                    // batchGet returns ranges in request order; pad if API omits empties
                    if value_ranges.len() == chunk.len() {
                        guest_rows.extend(value_ranges);
                    } else {
                        warn!(
                            "batchGet returned {} ranges for {} requested rows; aligning by index",
                            value_ranges.len(),
                            chunk.len()
                        );
                        for (i, _row) in chunk.iter().enumerate() {
                            if i < value_ranges.len() {
                                guest_rows.push(value_ranges[i].clone());
                            } else {
                                guest_rows.push(ValueRange::default());
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to batch-fetch guest rows {:?} from spreadsheet {}: {}",
                        chunk, self.spreadsheet_id, e
                    );
                    for _ in chunk {
                        guest_rows.push(ValueRange::default());
                    }
                }
            }
        }

        guest_rows
    }
}

fn cell_string(map: &HashMap<String, String>, key: &str) -> String {
    decode_html_entities(map.get(key).unwrap().trim())
        .into_owned()
}
