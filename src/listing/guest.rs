
use std::fmt;
use std::error::Error;
use chrono::NaiveDate;
use log::warn;

#[derive(Clone, Debug)]
pub struct Guest {
    pub row: String,
    pub timestamp: String,
    pub purpose_of_stay: String,
    pub check_in: String,
    pub check_out: String,
    pub surname: String,
    pub first_name: String,
    pub birth_date: String,
    pub country_of_citizenship: String,
    travel_doc_number: String,
    visa_number: String,
    pub address_abroad: String,
    pub full_name: String,
    pub data_errors: Vec<GuestError>
}

impl fmt::Display for Guest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Row: {}, Timestamp: {}, Purpose_of_Stay: {}, Check_In: {}, Check_Out: {}, Surname: {}, First_Name: {}, Birth_Date: {}, Country_of_Citizenship: {}, Address_Abroad: {}, Full_Name: {}",
               self.row, self.timestamp, self.purpose_of_stay, self.check_in, self.check_out, self.surname, self.first_name, self.birth_date, self.country_of_citizenship, self.address_abroad, self.full_name)
    }
}

#[derive(Clone, Debug)]
pub enum GuestError {
    InvalidInput(String),
}

impl fmt::Display for GuestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuestError::InvalidInput(field) => write!(f, "Invalid input provided for {}", field),
        }
    }
}

impl Error for GuestError {}

impl Guest {
    pub fn new(row: &String, timestamp: String, purpose_of_stay: String, check_in: String, check_out: String, surname: String, first_name: String, birth_date: String,
               country_of_citizenship: String, travel_doc_number: String, visa_number: String, address_abroad: String, full_name: String) -> Self {
        let mut guest = Guest {
            row: row.to_string(),
            timestamp,
            purpose_of_stay,
            check_in,
            check_out,
            surname,
            first_name,
            birth_date,
            country_of_citizenship,
            travel_doc_number,
            visa_number,
            address_abroad,
            full_name,
            data_errors: Vec::new(),
        };

        guest.check_input_format();

        guest
    }

    pub fn get_u_record(&self) -> String {
        format!{"U|{}|{}|{}|{}||{}|||{}|{}|{}|{}|{}||",
            self.check_in,
            self.check_out,
            self.surname,
            self.first_name,
            self.birth_date,
            self.country_of_citizenship,
            self.address_abroad,
            self.travel_doc_number,
            self.visa_number,
            self.purpose_of_stay
        }
    }
   
    fn check_input_format(&mut self) {

        if let Err(e) = self.check_format_check_in() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        }; 
        if let Err(e) = self.check_format_check_out() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_surname() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_first_name() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_dob() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_country_of_citizenship() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_address_abroad() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_travel_doc_number() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_visa_number() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
        if let Err(e) = self.check_format_purpose_of_stay() {
            warn!("Row {}, {} {}: {}", self.row, self.first_name, self.surname, e);
            self.data_errors.push(e)
        };
    }
    
    pub fn get_data_errors(&self) -> String {
        if !self.data_errors.is_empty() {
            let mut msg = String::from("Input error found in field(s): ");
            
            for(index, error) in self.data_errors.iter().enumerate() {
                match error {
                    GuestError::InvalidInput(e) => {
                        if index == self.data_errors.len() - 1 {
                            msg.push_str(&format!("{}", e));
                        } else {
                            msg.push_str(&format!("{}, ", e));
                        }
                    },
                };
            }

            return msg
        }

        String::from("No guest input errors found")
    }

    fn check_format_check_in(&self) -> Result<(), GuestError> {
        if !NaiveDate::parse_from_str(&self.check_in, "%d.%m.%Y").is_ok() {
            return Err(GuestError::InvalidInput(String::from("check in date")));
        }
        Ok(())
    }

    fn check_format_check_out(&self) -> Result<(), GuestError> {
        if !NaiveDate::parse_from_str(&self.check_out, "%d.%m.%Y").is_ok() {
            return Err(GuestError::InvalidInput(String::from("check out date")));
        }
        Ok(())
    }

    fn check_format_surname(&self) -> Result<(), GuestError> {
        match self.surname.chars().count() {
            1..=50 => Ok(()),
            _ => return Err(GuestError::InvalidInput(String::from("surname"))),
        } 
    }

    fn check_format_first_name(&self) -> Result<(), GuestError> {
        match self.first_name.chars().count() {
            0..=24 => Ok(()),
            _ => return Err(GuestError::InvalidInput(String::from("first name"))),
        } 
    }

    fn check_format_dob(&self) -> Result<(), GuestError> {
        if !NaiveDate::parse_from_str(&self.birth_date, "%d.%m.%Y").is_ok() {
            return Err(GuestError::InvalidInput(String::from("date of birth")));
        }
        Ok(())
    }

    fn check_format_country_of_citizenship(&self) -> Result<(), GuestError> {
        match self.country_of_citizenship.chars().count() {
            3 => Ok(()),
            _ => return Err(GuestError::InvalidInput(String::from("country of citizenship"))),
        } 
    }

    fn check_format_address_abroad(&self) -> Result<(), GuestError> {
        match self.address_abroad.chars().count() {
            0..=255 => Ok(()),
            _ => return Err(GuestError::InvalidInput(String::from("address abroad"))),
        } 
    }

    fn check_format_travel_doc_number(&self) -> Result<(), GuestError> {
        match self.travel_doc_number.chars().count() {
            6..=30 => Ok(()),
            _ => return Err(GuestError::InvalidInput(String::from("travel doc number"))),
        } 
    }

    fn check_format_visa_number(&self) -> Result<(), GuestError> {
        match self.visa_number.chars().count() {
            0..=15 => Ok(()),
            _ => return Err(GuestError::InvalidInput(String::from("visa number"))),
        } 
    }

    fn check_format_purpose_of_stay(&self) -> Result<(), GuestError> {
        match self.purpose_of_stay.chars().count() {
            2 => Ok(()),
            _ => return Err(GuestError::InvalidInput(String::from("purpose of stay"))),
        } 
    }
}
