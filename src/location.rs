// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

/// Contains latitude and longitude information for server the client
/// locations.
///
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "cffi", repr(C))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Location {
    #[cfg_attr(feature = "serde", serde(rename = "Latitude"))]
    pub latitude: f32,
    #[cfg_attr(feature = "serde", serde(rename = "Longitude"))]
    pub longitude: f32,
}
