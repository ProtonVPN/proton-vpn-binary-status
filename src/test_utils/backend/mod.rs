// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

//! This module provides test utilities for interacting with the https backend
//! api.

use anyhow::Result;

use crate::{Country, Load, Location, Logical};

// CA#667
pub const USER_IP_ADDRESS: &str = "149.102.228.0";
pub const USER_LATITUDE: f32 = 34.0544;
pub const USER_LONGITUDE: f32 = -118.244;
pub const USER_COUNTRY: &[u8; 2] = b"US";

const NETZONE_HEADER: &str = "X-PM-netzone";

pub mod compute_variance;
mod endpoints;
pub mod v1;
pub mod v2;

pub use compute_variance::compute_variance;
pub use endpoints::Endpoints;
