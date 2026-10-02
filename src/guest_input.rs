//! Map Sheet guests to CheckIn `GuestInput` JSON (POC soap mode / #68).

use serde::Serialize;

/// Payload matching CheckIn `POST /api/ubyport/submit-guests` guest schema.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GuestInput {
    pub surname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    pub birth_date: String,
    pub check_in: String,
    pub check_out: String,
    pub nationality: String,
    pub travel_document_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visa_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_abroad: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose_of_stay: Option<String>,
}

/// Redact travel document for email/logs (keep last 4).
pub fn redact_travel_document(value: &str) -> String {
    let trimmed: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    if trimmed.chars().count() <= 4 {
        return "****".to_string();
    }
    let last4: String = trimmed.chars().rev().take(4).collect::<String>().chars().rev().collect();
    format!("***{last4}")
}

pub fn map_sheet_guest(
    surname: &str,
    first_name: &str,
    birth_date: &str,
    check_in: &str,
    check_out: &str,
    nationality: &str,
    travel_document_number: &str,
    visa_number: &str,
    address_abroad: &str,
    purpose_of_stay: &str,
) -> GuestInput {
    let first = first_name.trim();
    let visa = visa_number.trim();
    let addr = address_abroad.trim();
    let purpose = purpose_of_stay.trim();

    GuestInput {
        surname: surname.trim().to_string(),
        first_name: if first.is_empty() {
            None
        } else {
            Some(first.to_string())
        },
        birth_date: birth_date.trim().to_string(),
        check_in: check_in.trim().to_string(),
        check_out: check_out.trim().to_string(),
        nationality: nationality.trim().to_uppercase(),
        travel_document_number: travel_document_number.trim().to_string(),
        visa_number: if visa.is_empty() {
            None
        } else {
            Some(visa.to_string())
        },
        address_abroad: if addr.is_empty() {
            None
        } else {
            Some(addr.to_string())
        },
        purpose_of_stay: if purpose.is_empty() {
            None
        } else {
            Some(purpose.to_string())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_guest_to_checkin_shape() {
        let g = map_sheet_guest(
            "Novák",
            "Jan",
            "15.03.1990",
            "01.10.2026",
            "05.10.2026",
            "svk",
            "AB1234567",
            "",
            "Bratislava",
            "01",
        );
        let json = serde_json::to_value(&g).unwrap();
        assert_eq!(json["surname"], "Novák");
        assert_eq!(json["firstName"], "Jan");
        assert_eq!(json["birthDate"], "15.03.1990");
        assert_eq!(json["nationality"], "SVK");
        assert_eq!(json["travelDocumentNumber"], "AB1234567");
        assert!(json.get("visaNumber").is_none());
    }

    #[test]
    fn redacts_travel_docs() {
        assert_eq!(redact_travel_document("AB1234567"), "***4567");
        assert_eq!(redact_travel_document("AB12"), "****");
        assert!(!redact_travel_document("AB1234567").contains("AB1234567"));
    }
}
