mod listing;
mod settings;
mod logger;
mod unlfile;
mod email;

use listing::Listing;
use unlfile::UnlFile;
use email::Email;
use logger::Logger;
use log::{info, error};
use settings::Settings;
use tokio;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
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

    /////////////////////////////////////////////////////////////////
    // Create UNL file for each listing which contains:            //
    // a_record - top row, contains listing metadata               //
    // u_record - preceeding rows, one for each unregistered guest //
    /////////////////////////////////////////////////////////////////
    
    let mut unl_files: Vec<UnlFile> = Vec::new();
    let mut all_unreg_guests: Vec<Vec<String>> = Vec::new();
    let mut all_checkin_issues: Vec<Vec<String>> = Vec::new();
    
    for listing in settings.listing {
        let listing: Listing = Listing::new(
            &listing.id,
            &listing.name,
            &listing.address,
            &listing.google_client_id,
            &listing.google_client_secret,
            &listing.google_spreadsheet_id,
            &listing.google_sheet_name,
            &listing.a_record,
        ).await;

        info!("Listing: {}", listing.get_name());
        
        // Find Unregistered Guests
        let mut unreg_guests = listing.find_unregistered_guests().await;

        // Remove unregistered guests with checkin issues
        for (_index, guest) in unreg_guests.iter().enumerate() {
            if !guest.data_errors.is_empty() {
                all_checkin_issues.push(
                    vec![listing.get_name().to_string(),
                    guest.row.clone(),
                    format!("{} {}", guest.first_name, guest.surname ),
                    guest.get_data_errors()]
                );
            }
        }

        unreg_guests.retain(|guest| guest.data_errors.is_empty());

        if !unreg_guests.is_empty() {

            // Get filepath
            let file_name = format!("{}{}{}", settings.unl_file_directory, listing.get_name(), ".unl");

            // Find u_records
            let mut u_records: Vec<String> = Vec::new();
            for guest in &unreg_guests {
                let u_record = guest.get_u_record();
                u_records.push(u_record);
            }

            // Create UNL file
            let result= UnlFile::new(&listing.get_a_record(), u_records, &file_name);
            match result {
                Ok(unl_file) => {
                    info!("UNLFile created successfully");
                    unl_files.push(unl_file);
                    
                    
                    // Prepare unregistered guests for email
                    for guest in &unreg_guests {
                        info!("{}", guest);
                        
                        all_unreg_guests.push(
                            vec![listing.get_name().to_string(),
                            guest.row.clone(),
                            format!("{} {}", guest.first_name, guest.surname ),
                            guest.check_in.clone(),
                            guest.check_out.clone()]
                        );

                        // Update guest as registered
                        listing.update_guest_as_registered(
                            &guest.row,
                            &guest.first_name,
                            &guest.surname
                        ).await;
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                    // Send admin email error
                    // #### Add Here ####
                    continue;
                },
            }
        } else {
            info!("No unregistered guests found for {}", listing.get_name());

        }
    }

    ///////////
    // Email //
    ///////////

    // Create Email
    info!("Prepare Email For Sending");
    let mut attachments: Vec<String> = Vec::new();
    for file in unl_files {
        attachments.push(file.get_filename().to_string());
    }

    // Add Errors
    // todo

    let mail = Email::new(
        attachments, 
        settings.ses.from,
        settings.ses.to,
        "Guest Checkin - Unregistered Guests Available",
        &settings.aws.access_key,
        &settings.aws.secret_key,
        &settings.aws.region,
    );

    // Send Mail
        mail.send(all_unreg_guests, all_checkin_issues).await;

    // Upon unsuccessful email delivery, mark all guests as unregistered
    // for listing in settings.listing {
    //    
    // }


    Ok(())
}