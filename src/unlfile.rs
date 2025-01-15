use encoding_rs::WINDOWS_1250;
use std::fs::File;
use std::io::{Write};
use std::error::Error;
use std::fmt;


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

pub struct UnlFile {
    file_name: String,
    unl_file: File,
}

impl UnlFile {
    pub fn new(a_record: &str, u_records: Vec<String>, file_name: &str) -> Result<Self, UnlFileError> {
        // Create File
        let mut file: File = File::create(file_name)?;

        // Add A-Record
        Self::add_a_record(&mut file, a_record)?;
        
        // Add U-Record
        Self::add_u_records(&mut file, u_records)?;

        // Ensure file is flushed
        file.flush()?;

        Ok(
            Self {
            unl_file: file,
            file_name: file_name.to_string(),
            }
        )
    }

    pub fn get_filename(&self) -> &str { &self.file_name }

    fn add_a_record(file: &mut File, a_record: &str) -> Result<(), UnlFileError> {
        let record = format!("{}\r\n", a_record);
        let (encoded, _, _) = WINDOWS_1250.encode(&record);
        file.write_all(&encoded)?;

        Ok(())
    }

    fn add_u_records(file: &mut File, u_records: Vec<String>) -> Result<(), UnlFileError> {
        for u_record in u_records {
            let record = format!("{}\r\n", u_record);
            let (encoded, _, _) = WINDOWS_1250.encode(&record);
            file.write_all(&encoded)?;
        }

        Ok(())
    }
}