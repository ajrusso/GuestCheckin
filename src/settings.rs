use config::{Config, ConfigError, File};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubmitMode {
    Unl,
    Soap,
}

impl Default for SubmitMode {
    fn default() -> Self {
        SubmitMode::Unl
    }
}

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
    pub a_record: String,
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

#[derive(Debug, Deserialize, Clone)]
pub struct CheckIn {
    pub base_url: String,
    pub pilot_token: String,
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
    #[serde(default = "default_scan_state_filepath")]
    pub scan_state_filepath: String,
    /// `unl` (default) or `soap` (CheckIn submit-guests POC bridge).
    #[serde(default)]
    pub submit_mode: SubmitMode,
    /// Required when `submit_mode = soap`.
    pub checkin: Option<CheckIn>,
}

fn default_scan_state_filepath() -> String {
    "./scan_state.json".to_string()
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Portable/prod layout first, then repo-local layout for development.
        let config_path = if std::path::Path::new("config.toml").exists() {
            "config.toml"
        } else {
            "src/config/config.toml"
        };

        let s = Config::builder()
            .add_source(File::with_name(config_path))
            .build()?;

        s.try_deserialize()
    }
}
