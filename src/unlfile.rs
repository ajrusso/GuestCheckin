//! UNL file generation for UbyPort.
//!
//! UNL (guest list) files use one A-record (header) and multiple U-records (one per guest).
//! All content is transliterated per UbyPort rules, then encoded as Windows-1250 (CP1250).

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Write;

use encoding_rs::WINDOWS_1250;

use crate::transliteration::transliterate;

/// Errors that can occur when creating or writing an UNL file.
#[derive(Debug)]
pub enum UnlFileError {
    IOError(std::io::Error),
}

impl From<std::io::Error> for UnlFileError {
    fn from(error: std::io::Error) -> Self {
        UnlFileError::IOError(error)
    }
}

impl fmt::Display for UnlFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnlFileError::IOError(field) => write!(f, "Standard IO Error for {}", field),
        }
    }
}

impl Error for UnlFileError {}

/// Represents a UNL file on disk and its filename.
/// The constructor writes all records immediately, then keeps the file handle.
pub struct UnlFile {
    file_name: String,
    unl_file: File,
}

impl UnlFile {
    /// Create a new UNL file and write:
    /// - A-record (header)
    /// - U-records (guest lines)
    ///
    /// `a_record` and `u_records` are assumed to be raw UTF-8 strings
    /// from your database/business logic.
    ///
    /// Inside this function we:
    /// 1. Transliterate each record according to UbyPort rules
    /// 2. Encode each line as Windows-1250
    pub fn new(
        a_record: &str,
        u_records: Vec<String>,
        file_name: &str,
    ) -> Result<Self, UnlFileError> {
        // Create the target file (overwrites if it already exists).
        let mut file: File = File::create(file_name)?;

        // Add A-record (header line).
        Self::add_a_record(&mut file, a_record)?;

        // Add all U-records (guest lines).
        Self::add_u_records(&mut file, u_records)?;

        // Ensure all data is flushed to disk before returning.
        file.flush()?;

        Ok(Self {
            unl_file: file,
            file_name: file_name.to_string(),
        })
    }

    /// Get the name of the UNL file on disk.
    pub fn get_filename(&self) -> &str {
        &self.file_name
    }

    /// Write the A-record to the file.
    ///
    /// Steps:
    /// - Transliterate the record (UbyPort spec)
    /// - Append CRLF (`\r\n`) as required by UNL format
    /// - Encode using Windows-1250
    fn add_a_record(file: &mut File, a_record: &str) -> Result<(), UnlFileError> {
        // Apply transliteration rules first (e.g., Ñ → N, ß → SS).
        let sanitized = transliterate(a_record);

        // Add CRLF terminator for UNL line format.
        let record = format!("{}\r\n", sanitized);

        // Encode as Windows-1250.
        let (encoded, _, _) = WINDOWS_1250.encode(&record);

        file.write_all(&encoded)?;

        Ok(())
    }

    /// Write all U-records (guest lines) to the file.
    ///
    /// Each `u_record` is:
    /// - Transliteration-filtered (only allowed characters remain)
    /// - Terminated with CRLF (`\r\n`)
    /// - Encoded as Windows-1250
    fn add_u_records(file: &mut File, u_records: Vec<String>) -> Result<(), UnlFileError> {
        for u_record in u_records {
            // Apply transliteration so the line only contains allowed characters.
            let sanitized = transliterate(&u_record);

            // Add CRLF line ending.
            let record = format!("{}\r\n", sanitized);

            // Encode as Windows-1250.
            let (encoded, _, _) = WINDOWS_1250.encode(&record);

            file.write_all(&encoded)?;
        }

        Ok(())
    }
}