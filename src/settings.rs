use config::{Config, File, ConfigError};
use serde_derive::Deserialize;


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Listing {
    pub id: String,
    pub name: String,
    pub address: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_spreadsheet_id: String,
    pub google_sheet_name: String,
    pub a_record: String
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct AWS {
    pub region: String,
    pub stage: String,
    pub api_id: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct SES {
    pub from: String,
    pub to: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub aws: AWS,
    pub ses: SES,
    pub listing: Vec<Listing>,
    pub log_filepath: String,
    pub unl_file_directory: String,
    pub service_account_key_filepath: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {        
        let s = Config::builder()
            .add_source(File::with_name("src/config/config.toml"))
            .build()?;

        s.try_deserialize()
    }
}
