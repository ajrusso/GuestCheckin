use config::{Config, ConfigError, File, FileFormat};
use serde_derive::Deserialize;
use std::path::{Path, PathBuf};

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

/// Resolve config file path.
///
/// Priority (documented in `docs/configuration.md`):
/// 1. `--config <path>` / `--config=<path>`
/// 2. `GUESTCHECKIN_CONFIG` env
/// 3. `./config.toml` (portable/prod)
/// 4. `src/config/config.toml` (repo-local dev)
pub fn resolve_config_path() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--config" {
            if let Some(path) = args.next() {
                return PathBuf::from(path);
            }
        } else if let Some(path) = arg.strip_prefix("--config=") {
            return PathBuf::from(path);
        }
    }

    if let Ok(path) = std::env::var("GUESTCHECKIN_CONFIG") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    if Path::new("config.toml").exists() {
        PathBuf::from("config.toml")
    } else {
        PathBuf::from("src/config/config.toml")
    }
}

fn file_source(path: &Path) -> File<config::FileSourceFile, FileFormat> {
    let path_str = path.to_string_lossy();
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
    {
        File::new(path_str.as_ref(), FileFormat::Toml)
    } else {
        File::with_name(path_str.as_ref())
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = resolve_config_path();
        log::info!("Loading config from {}", config_path.display());

        let s = Config::builder()
            .add_source(file_source(&config_path))
            .build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_falls_back_to_default_names() {
        // Without env/args overrides in unit test process, expect one of the defaults.
        let path = resolve_config_path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        assert!(
            name == "config.toml" || path.ends_with("src/config/config.toml"),
            "unexpected default config path: {}",
            path.display()
        );
    }
}
