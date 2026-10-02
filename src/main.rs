mod listing;
mod settings;
mod logger;
mod unlfile;
mod email;
mod scan_state;

use listing::Listing;
use unlfile::UnlFile;
use email::Email;
use logger::Logger;
use log::{error, info, warn};
use settings::{Settings, SubmitMode};
use scan_state::{ListingScanState, ScanState};
use guest_checkin::checkin_client::{should_mark_column_m, CheckInClient};
use guest_checkin::guest_input::GuestInput;
use tokio;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn pause_before_exit() {
    println!();
    print!("Press Enter to exit...");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
}

/// Ensures the console pause runs even if the app panics (unwinding).
struct PauseOnDrop;

impl Drop for PauseOnDrop {
    fn drop(&mut self) {
        pause_before_exit();
    }
}

#[tokio::main]
async fn main() {
    let _pause = PauseOnDrop;
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::new()?;
    Logger::new(log::LevelFilter::Info, "./output.log")?;

    info!(r"                       _          _               _    _       ");
    info!(r"                      | |        | |             | |  (_)      ");
    info!(r"  __ _ _   _  ___  ___| |_    ___| |__   ___  ___| | ___ _ __  ");
    info!(r" / _` | | | |/ _ \/ __| __|  / __| '_ \ / _ \/ __| |/ / | '_ \ ");
    info!(r"| (_| | |_| |  __/\__ \ |_  | (__| | | |  __/ (__|   <| | | | |");
    info!(r" \__, |\__,_|\___||___/\__|  \___|_| |_|\___|\___|_|\_\_|_| |_|");
    info!(r"  __/ |                                                        ");
    info!(r" |___/                                                         ");

    info!("Starting Guest Checkin...");
    info!("submit_mode = {:?}", settings.submit_mode);

    match settings.submit_mode {
        SubmitMode::Unl => run_unl_mode(&settings).await?,
        SubmitMode::Soap => run_soap_mode(&settings).await?,
    }

    Ok(())
}

async fn run_unl_mode(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let mut unl_files: Vec<UnlFile> = Vec::new();
    let mut all_unreg_guests: Vec<Vec<String>> = Vec::new();
    let mut all_checkin_issues: Vec<Vec<String>> = Vec::new();
    let mut scan_state = ScanState::load(&settings.scan_state_filepath);

    let path = Path::new(&settings.unl_file_directory);
    if !path.exists() {
        match fs::create_dir(path) {
            Ok(_) => info!("Directory {} created successfully!", path.display()),
            Err(e) => info!("Failed to create directory {}: {}", path.display(), e),
        }
    } else {
        info!("Directory {} already exists.", path.display());
    }

    for listing_cfg in &settings.listing {
        let listing = Listing::new(
            &listing_cfg.id,
            &listing_cfg.name,
            &listing_cfg.address,
            &listing_cfg.google_spreadsheet_id,
            &listing_cfg.google_sheet_name,
            &listing_cfg.a_record,
            &settings.service_account_key_filepath,
        )
        .await;

        info!("Listing: {}", listing.get_name());

        let listing_state = scan_state.get_listing(
            listing.get_spreadsheet_id(),
            listing.get_sheet_name(),
        );

        let scan_result = listing
            .find_unregistered_guests(&listing_state.pending_rows, listing_state.last_seen_row)
            .await;
        let mut unreg_guests = scan_result.guests;
        let candidate_rows = scan_result.candidate_rows;

        for guest in unreg_guests.iter() {
            if !guest.data_errors.is_empty() {
                all_checkin_issues.push(vec![
                    listing.get_name().to_string(),
                    guest.row.clone(),
                    format!("{} {}", guest.first_name, guest.surname),
                    guest.get_data_errors(),
                ]);
            }
        }

        unreg_guests.retain(|guest| guest.data_errors.is_empty());

        let mut registered_rows: HashSet<u32> = HashSet::new();

        if !unreg_guests.is_empty() {
            let file_name = format!("{}{}{}", settings.unl_file_directory, listing.get_name(), ".unl");
            let mut u_records: Vec<String> = Vec::new();
            for guest in &unreg_guests {
                u_records.push(guest.get_u_record());
            }

            match UnlFile::new(&listing.get_a_record(), u_records, &file_name) {
                Ok(unl_file) => {
                    info!("UNLFile created successfully");
                    unl_files.push(unl_file);

                    for guest in &unreg_guests {
                        info!("{}", guest);

                        all_unreg_guests.push(vec![
                            listing.get_name().to_string(),
                            guest.row.clone(),
                            format!("{} {}", guest.first_name, guest.surname),
                            guest.check_in.clone(),
                            guest.check_out.clone(),
                        ]);

                        listing
                            .update_guest_as_registered(
                                &guest.row,
                                &guest.first_name,
                                &guest.surname,
                            )
                            .await;

                        if let Ok(row_num) = guest.row.parse::<u32>() {
                            registered_rows.insert(row_num);
                        }
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        } else {
            info!("No unregistered guests found for {}", listing.get_name());
        }

        let mut pending_rows: Vec<u32> = candidate_rows
            .into_iter()
            .filter(|row| !registered_rows.contains(row))
            .collect();
        pending_rows.sort_unstable();

        scan_state.set_listing(
            listing.get_spreadsheet_id(),
            listing.get_sheet_name(),
            ListingScanState {
                last_seen_row: scan_result.last_seen_row,
                pending_rows,
            },
        );
    }

    if let Err(e) = scan_state.save(&settings.scan_state_filepath) {
        warn!("Failed to persist scan state: {}", e);
    } else {
        info!("Saved scan state to {}", settings.scan_state_filepath);
    }

    info!("Prepare Email For Sending");
    let mut attachments: Vec<String> = Vec::new();
    for file in unl_files {
        attachments.push(file.get_filename().to_string());
    }

    let mail = Email::new(
        attachments,
        settings.ses.from.clone(),
        settings.ses.to.clone(),
        "Guest Checkin - Unregistered Guests Available",
        &settings.aws.access_key,
        &settings.aws.secret_key,
        &settings.aws.region,
    );

    mail.send(all_unreg_guests, all_checkin_issues).await;
    Ok(())
}

async fn run_soap_mode(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let checkin_cfg = settings.checkin.as_ref().ok_or(
        "submit_mode=soap requires [checkin] base_url and pilot_token in config.toml",
    )?;
    let client = CheckInClient::new(&checkin_cfg.base_url, &checkin_cfg.pilot_token)?;

    let mut all_accepted: Vec<Vec<String>> = Vec::new();
    let mut all_rejected: Vec<Vec<String>> = Vec::new();
    let mut all_checkin_issues: Vec<Vec<String>> = Vec::new();
    let mut scan_state = ScanState::load(&settings.scan_state_filepath);

    for listing_cfg in &settings.listing {
        let listing = Listing::new(
            &listing_cfg.id,
            &listing_cfg.name,
            &listing_cfg.address,
            &listing_cfg.google_spreadsheet_id,
            &listing_cfg.google_sheet_name,
            &listing_cfg.a_record,
            &settings.service_account_key_filepath,
        )
        .await;

        info!("Listing: {}", listing.get_name());

        let listing_state = scan_state.get_listing(
            listing.get_spreadsheet_id(),
            listing.get_sheet_name(),
        );

        let scan_result = listing
            .find_unregistered_guests(&listing_state.pending_rows, listing_state.last_seen_row)
            .await;
        let mut unreg_guests = scan_result.guests;
        let candidate_rows = scan_result.candidate_rows;

        for guest in unreg_guests.iter() {
            if !guest.data_errors.is_empty() {
                all_checkin_issues.push(vec![
                    listing.get_name().to_string(),
                    guest.row.clone(),
                    format!("{} {}", guest.first_name, guest.surname),
                    guest.get_data_errors(),
                ]);
            }
        }

        unreg_guests.retain(|guest| guest.data_errors.is_empty());

        let mut registered_rows: HashSet<u32> = HashSet::new();

        if !unreg_guests.is_empty() {
            let guests: Vec<GuestInput> = unreg_guests.iter().map(|g| g.to_guest_input()).collect();
            let batch_id = format!(
                "{}-{}",
                listing.get_name().replace(' ', "_"),
                chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
            );

            match client.submit_guests(&batch_id, &guests).await {
                Ok(outcome) => {
                    let results = outcome
                        .data
                        .as_ref()
                        .map(|d| d.results.as_slice())
                        .unwrap_or(&[]);

                    for (idx, guest) in unreg_guests.iter().enumerate() {
                        let result = results.iter().find(|r| r.index == idx);
                        let status = result
                            .map(|r| r.status.as_str())
                            .unwrap_or_else(|| if outcome.ok { "accepted" } else { "rejected" });
                        let codes = result
                            .map(|r| {
                                r.chyby_codes
                                    .iter()
                                    .map(|c| c.to_string())
                                    .collect::<Vec<_>>()
                                    .join(";")
                            })
                            .unwrap_or_default();

                        let fullname = format!("{} {}", guest.first_name, guest.surname);

                        if should_mark_column_m(status) {
                            listing
                                .update_guest_as_registered(
                                    &guest.row,
                                    &guest.first_name,
                                    &guest.surname,
                                )
                                .await;
                            if let Ok(row_num) = guest.row.parse::<u32>() {
                                registered_rows.insert(row_num);
                            }
                            all_accepted.push(vec![
                                listing.get_name().to_string(),
                                guest.row.clone(),
                                fullname,
                                guest.check_in.clone(),
                                guest.check_out.clone(),
                                status.to_string(),
                            ]);
                        } else {
                            all_rejected.push(vec![
                                listing.get_name().to_string(),
                                guest.row.clone(),
                                fullname,
                                status.to_string(),
                                codes,
                            ]);
                        }
                    }

                    if results.is_empty() && !outcome.ok {
                        warn!(
                            "No per-guest results from CheckIn (HTTP {} code {:?})",
                            outcome.http_status, outcome.error_code
                        );
                    }
                }
                Err(e) => {
                    error!("CheckIn submit failed for {}: {}", listing.get_name(), e);
                    for guest in &unreg_guests {
                        all_rejected.push(vec![
                            listing.get_name().to_string(),
                            guest.row.clone(),
                            format!("{} {}", guest.first_name, guest.surname),
                            "error".to_string(),
                            e.clone(),
                        ]);
                    }
                }
            }
        } else {
            info!("No unregistered guests found for {}", listing.get_name());
        }

        let mut pending_rows: Vec<u32> = candidate_rows
            .into_iter()
            .filter(|row| !registered_rows.contains(row))
            .collect();
        pending_rows.sort_unstable();

        scan_state.set_listing(
            listing.get_spreadsheet_id(),
            listing.get_sheet_name(),
            ListingScanState {
                last_seen_row: scan_result.last_seen_row,
                pending_rows,
            },
        );
    }

    if let Err(e) = scan_state.save(&settings.scan_state_filepath) {
        warn!("Failed to persist scan state: {}", e);
    } else {
        info!("Saved scan state to {}", settings.scan_state_filepath);
    }

    info!("Prepare soap-mode distro email (no UNL/PDF attachments)");
    let mail = Email::new(
        Vec::new(),
        settings.ses.from.clone(),
        settings.ses.to.clone(),
        "Guest Checkin - UbyPort submit summary (POC)",
        &settings.aws.access_key,
        &settings.aws.secret_key,
        &settings.aws.region,
    );
    mail.send_soap_summary(all_accepted, all_rejected, all_checkin_issues)
        .await;

    Ok(())
}
