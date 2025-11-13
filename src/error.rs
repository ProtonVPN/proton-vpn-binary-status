// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum Error {
    #[error("failed to parse status file with error {0}")]
    ParserError(String),
    #[error(
        "Length of Logicals ({servers}) and Loads ({loads}) are not the same."
    )]
    LengthsNotConsistent { servers: u64, loads: u64 },
}
pub type Result<T> = std::result::Result<T, Error>;
