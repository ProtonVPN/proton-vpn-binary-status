// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------
use crate::country_code::CountryCode;
use crate::location::Location;

/// Contains information necessary for calculating the
/// server status
///
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "cffi", repr(C))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct StatusReference {
    // The position of the server in the binary status file
    #[cfg_attr(feature = "serde", serde(rename = "Index"))]
    pub index: u32,
    // The penalty computed by the back end
    #[cfg_attr(feature = "serde", serde(rename = "Penalty"))]
    pub penalty: f64,
    // Is this server expensive ? 1 for yes, 0 for no.
    #[cfg_attr(feature = "serde", serde(rename = "Cost"))]
    pub cost: u8,
}

/// Contains server specific information obtained from /logicals
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "cffi", repr(C))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Logical {
    // The information necesary for calculating the status of the server
    #[cfg_attr(feature = "serde", serde(rename = "StatusReference"))]
    pub status_reference: StatusReference,
    // The entry location of the server, this is used for secure core servers
    // to determine the entry point of the secure core tunnel.
    #[cfg_attr(feature = "serde", serde(rename = "EntryLocation"))]
    pub entry_location: Location,
    // The longitude and latitude of the server
    #[cfg_attr(feature = "serde", serde(rename = "ExitLocation"))]
    pub exit_location: Location,
    // A 2 character byte array representing the country this server is in.
    #[cfg_attr(feature = "serde", serde(rename = "ExitCountry"))]
    pub exit_country: CountryCode,
}
