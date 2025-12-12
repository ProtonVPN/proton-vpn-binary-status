// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

//! This library provides functionality to compute logical scores and loads
//! based on the binary status file provided by the backend.
//!
//! It includes:
//! - A `compute_loads` function that computes the load for each server based on
//!   the user location and status file.
//! - A parser for the binary status file.

#[cfg(feature = "uniffi")]
mod bindings_uniffi;
mod compute_loads;
mod compute_score;
mod coord;
mod country_code;
mod error;
mod jitter;
mod load;
mod location;
mod logical;
mod status;

pub use compute_loads::compute_loads;
pub use country_code::{CountryCode, CountryCodeConversionError};
pub use error::{Error, Result};
pub use load::Load;
pub use location::Location;
pub use logical::*;
pub use status::Parser;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

#[cfg(feature = "cffi")]
pub mod bindings_cffi;

#[cfg(any(feature = "test_utils_backend", feature = "test_utils_webview"))]
pub mod test_utils;
