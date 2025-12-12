// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

use super::{compute_loads, CountryCode, Load, Location, Logical, Result};

#[uniffi::export]
pub fn compute_loads_uniffi(
    logicals: &[Logical],
    status_file: &[u8],
    user_location: &Option<Location>,
    user_country: &Option<CountryCode>,
) -> Result<Vec<Load>> {
    let mut result_loads = vec![Load::default(); logicals.len()];
    compute_loads(
        &mut result_loads,
        logicals,
        status_file,
        user_location,
        user_country,
    )?;
    Ok(result_loads)
}
