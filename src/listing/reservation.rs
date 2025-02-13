use std::collections::HashMap;
use google_sheets4::oauth2::{read_service_account_key, ServiceAccountAuthenticator};
use google_sheets4::{hyper::client::HttpConnector, hyper_rustls::HttpsConnector};
use google_sheets4::{Sheets, hyper, hyper_rustls};
use google_sheets4::api::ValueRange;
use serde_json::json;
use log::{debug, info, warn, error};
use crate::listing::guest::Guest;


#[derive(Clone)]
pub struct Reservation {
    spreadsheet_id: String,
    sheet_name: String,
    hub: Option<Sheets<HttpsConnector<HttpConnector>>>,
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
    pub async fn find_unregistered_guests(&self) -> Vec<Guest> {
        
        // Find row that contain unregistered guests in guest response form
        let unregistered_guest_row_nums = self.unregistered_guests().await;
        
        // Get guest row(s) from spreadsheet
        let unregistered_guest_rows = match &self.hub {
            Some(_hub) => {
                self.clone().get_guest_rows_response(unregistered_guest_row_nums.clone()).await
            },
            None => panic!("No Google hub found"),
        };

        // Convert to Guest object instances
        let mut unregistered_guests = Vec::new();
        let mut unregistered_guest_count = 0; 
        
        for unregistered_guest in &unregistered_guest_rows {
            let mut guest_hash: HashMap<String, String> = HashMap::new(); 
            let row_num = format!("{}", unregistered_guest_row_nums[unregistered_guest_count]);

            // Load row into hash
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

                        // Load hash into Guest Object
                        let guest = Guest::new(
                            &row_num,
                            guest_hash.get("timestamp").unwrap().to_string(),
                            guest_hash.get("purpose_of_stay").unwrap().to_string(),
                            guest_hash.get("check_in").unwrap().to_string(),
                            guest_hash.get("check_out").unwrap().to_string(),
                            guest_hash.get("surname").unwrap().to_string(),
                            guest_hash.get("first_name").unwrap().to_string(),
                            guest_hash.get("birth_date").unwrap().to_string(),
                            guest_hash.get("country_of_citizenship").unwrap().to_string(),
                            guest_hash.get("travel_doc_number").unwrap().to_string(),
                            guest_hash.get("visa_number").unwrap().to_string(),
                            guest_hash.get("address_abroad").unwrap().to_string(),
                            guest_hash.get("full_name").unwrap().to_string()
                        );

                        debug!("Found unregistered guest: {}", guest);

                        unregistered_guests.push(guest.clone());

                        // Check input data format
                        if !guest.data_errors.is_empty() {
                            warn!("Unregistered guest {} {} can not be registered: {}", 
                                guest.first_name,
                                guest.surname,
                                guest.get_data_errors())
                            ;
                        }                        
                    }
                },
                None => warn!("Empty guest row found"),
            }

            unregistered_guest_count += 1; 
        }
        // Add Guest object to list
        unregistered_guests
    }

    // Find Google Spreadsheet rows with unregistered guests
    async fn unregistered_guests(&self) -> Vec<u32> {
        let form_responses = self.clone().get_unregistered_responses().await;
        Self::get_unregistered_guests(form_responses)
    }

    // Gets Google Spreadsheet column containing is_registered bool
    async fn get_unregistered_responses(&self) -> ValueRange{
        let range = "!M2:M";
        let sheet_range = format!("{}{}", self.sheet_name, range);
        let result = self.hub.clone()
            .unwrap()
            .spreadsheets()
            .values_get(&self.spreadsheet_id, &sheet_range)
            .doit()
            .await
            .unwrap();

        result.1
    }

    // Checks "Registered With Authorities" column input for unregistered guests
    fn get_unregistered_guests(response: ValueRange) -> Vec<u32> {
        let mut guests: Vec<u32> = Vec::new();
        let mut i: u32 = 2;
        let mut unregister_guest_count = 0;
        if let Some(row) = response.values {
            info!("Checking a total of {} guests", row.iter().count());
            for cell in row {
                //let v = cell.get(0)//.unwrap().as_str().unwrap().to_lowercase();
                
                match cell.get(0) {
                    Some(value) => {
                        let v = value.as_str().unwrap().to_lowercase();
                        if v.contains("false") {
                            guests.push(i);
                            unregister_guest_count += 1;
                        }
                    },
                    None => {
                        guests.push(i);
                        unregister_guest_count += 1;
                    },
                }
                i += 1;
            }
        }

        info!("{} unregistered guests found", unregister_guest_count);

        guests
    }

    // Gets given rows (guests) from Google Spreadsheet
    async fn get_guest_rows_response(self, rows: Vec<u32>) -> Vec<ValueRange>{
        let mut guest_rows = Vec::new();
        
        for row in rows {
            let range = format!("!{}:{}", row, row);
            let sheet_range = format!("{}{}", self.sheet_name, range);
            let result = self.hub.clone()
                .unwrap()
                .spreadsheets()
                .values_get(&self.spreadsheet_id, &sheet_range)
                .doit()
                .await
                .unwrap();
            // Log response
            debug!("log guest_rows_response: {:?}", result.0);
            guest_rows.push(result.1)
        }
        guest_rows
    }
}