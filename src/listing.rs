mod guest;
mod reservation;

use crate::listing::guest::Guest;
use crate::listing::reservation::Reservation;
use chrono::{prelude::*, FixedOffset};


#[derive(Clone)]
pub struct Listing {
    id: String,
    name: String,
    address: String,
    client_id: String,
    client_secret: String,
    spreadsheet_id: String,
    sheet_name: String,
    reservation: Reservation,
    a_record: String,
}

impl Listing {
    pub async fn new(id: &str, name: &str, address: &str, client_id: &str, client_secret: &str, 
                     spreadsheet_id: &str, sheet_name: &str, a_record: &str) -> Self {
        Listing {
            id: id.to_string(),
            name: name.to_string(),
            address: address.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            spreadsheet_id: spreadsheet_id.to_string(),
            sheet_name: sheet_name.to_string(),
            a_record: Self::add_datetime(a_record),
            reservation: Reservation::new(client_id, client_secret, spreadsheet_id, sheet_name).await,
        }
    }

    pub fn get_name(&self) -> &str {&self.name}
    pub fn get_a_record(&self) -> &str {&self.a_record}

    pub async fn find_unregistered_guests(&self) -> Vec<Guest> {
        let unregistered_guests =  self.reservation.find_unregistered_guests().await;
        unregistered_guests
    }
    
    pub async fn update_guest_as_registered(&self, row: &str, first_name: &str, last_name: &str) {
        self.reservation.update_registered_with_authorities(row, first_name, last_name).await;
    }

    fn add_datetime(a_record: &str) -> String {
        let utc_now: DateTime<Utc> = Utc::now();
        let prague_offset = FixedOffset::east_opt(1 * 3600).expect("Invalid offset");
        let prague_now: DateTime<FixedOffset> = utc_now.with_timezone(&prague_offset);
        let formatted_datetime = prague_now.format("%Y.%m.%d %H:%M:%S").to_string();
        a_record.replace("AddDate", &formatted_datetime)
    }
}