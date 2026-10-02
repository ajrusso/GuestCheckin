use std::collections::HashMap;
use std::fs;
use std::path::Path;

use log::{info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListingScanState {
    #[serde(default)]
    pub last_seen_row: u32,
    #[serde(default)]
    pub pending_rows: Vec<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanState {
    #[serde(default)]
    pub listings: HashMap<String, ListingScanState>,
}

impl ScanState {
    pub fn listing_key(spreadsheet_id: &str, sheet_name: &str) -> String {
        format!("{}::{}", spreadsheet_id, sheet_name)
    }

    pub fn load(path: &str) -> Self {
        let path = Path::new(path);
        if !path.exists() {
            info!("No scan state file at {}; starting fresh", path.display());
            return ScanState::default();
        }

        match fs::read_to_string(path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(state) => state,
                Err(e) => {
                    warn!(
                        "Failed to parse scan state {}: {}; starting fresh",
                        path.display(),
                        e
                    );
                    ScanState::default()
                }
            },
            Err(e) => {
                warn!(
                    "Failed to read scan state {}: {}; starting fresh",
                    path.display(),
                    e
                );
                ScanState::default()
            }
        }
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create scan state directory: {}", e))?;
            }
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize scan state: {}", e))?;
        fs::write(path, json).map_err(|e| format!("Failed to write scan state {}: {}", path.display(), e))
    }

    pub fn get_listing(&self, spreadsheet_id: &str, sheet_name: &str) -> ListingScanState {
        self.listings
            .get(&Self::listing_key(spreadsheet_id, sheet_name))
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_listing(
        &mut self,
        spreadsheet_id: &str,
        sheet_name: &str,
        state: ListingScanState,
    ) {
        self.listings
            .insert(Self::listing_key(spreadsheet_id, sheet_name), state);
    }
}
