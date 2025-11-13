// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

/// Contains additional debug fields when this lib is built with the "debug"
/// feature enabled.
///
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "cffi", repr(C))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg(feature = "debug")]
pub struct LoadDebugFields {
    #[cfg_attr(feature = "serde", serde(rename = "PartialScore"))]
    pub partial_score: f64,
}

/// Contains an up to date status, load and score for a server.
///
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "cffi", repr(C))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Load {
    #[cfg_attr(feature = "serde", serde(rename = "IsEnabled"))]
    pub is_enabled: bool,
    #[cfg_attr(feature = "serde", serde(rename = "IsVisible"))]
    pub is_visible: bool,
    #[cfg_attr(feature = "serde", serde(rename = "IsAutoconnectable"))]
    pub is_autoconnectable: bool,
    #[cfg_attr(feature = "serde", serde(rename = "Load"))]
    pub load: u8,
    #[cfg_attr(feature = "serde", serde(rename = "Score"))]
    pub score: f64,
    #[cfg(feature = "debug")]
    #[cfg_attr(feature = "serde", serde(rename = "Debug"))]
    pub debug: LoadDebugFields,
}
