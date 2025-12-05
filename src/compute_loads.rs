// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------
use super::compute_score::{
    compute_score, ComputeScoreServerParams, STATUS_AUTOCONNECTABLE,
    STATUS_ENABLED, STATUS_VISIBLE,
};
use super::jitter;
use super::status::Parser;
use super::{Country, Error, Load, Location, Logical, Result};

/// Computes the load for each server based on the user location and status file.
/// The function updates the `loads` slice with the computed values.
/// # Arguments
/// * `loads` - A mutable slice of `Load` where the computed load and score will be stored.
/// * `logicals` - A slice of `Logical` servers that contains the server information.
/// * `status_file` - A byte slice representing the binary status file.
/// * `user_location` - An optional reference to the user's latitude and longitude.
/// * `user_country` - An optional reference to the user's country.
pub fn compute_loads(
    loads: &mut [Load],
    logicals: &[Logical],
    status_file: &[u8],
    user_location: &Option<Location>,
    user_country: &Option<Country>,
) -> Result<()> {
    let statuses = Parser::try_from(status_file)?;

    if loads.len() != logicals.len() {
        return Err(Error::LengthsNotConsistent {
            // Errors are used in bindings to other languages so they can't use usize.
            // The unwraps are ugly, but OTOH we'll never get file with length > u64 and indices are currently 32 bit long.
            servers: u64::try_from(logicals.len())  // nosemgrep: panic-in-function-returning-result
                .expect("Unable to convert from usize to u64"),
            loads: u64::try_from(loads.len())  // nosemgrep: panic-in-function-returning-result
                .expect("Unable to convert from usize to u64"),
        });
    }

    let mut normalized_jitter = jitter::generator();

    let mut error_reported = false;
    let mut report_parsing_error =
        move |index: usize, byte_offset: usize, error_msg: &str| {
            if !error_reported {
                error_reported = true;
                log::warn!(
                    "Failed to parse server status at index {index} with bytes offset {byte_offset}: {error_msg}. \
                    Using default status for this server. \
                    Further server status parsing errors will be ignored.",
                );
            }
        };

    let status_is_unknown = super::status::ServerStatus::default();
    for (load, logical) in std::iter::zip(loads, logicals) {
        // Obtain the status from the binary status file
        let status = statuses.get(
            logical.status_reference.index as usize,
            &status_is_unknown,
            &mut report_parsing_error,
        );

        // Compute the score
        let score = compute_score(
            ComputeScoreServerParams {
                status_penalty: logical.status_reference.penalty,
                status_cost: logical.status_reference.cost,
                country: logical.exit_country,
                partial_score: status.partial_score as f64,
                status: status.status,
                exit_location: &logical.exit_location,
                entry_location: &logical.entry_location,
                normalized_jitter: normalized_jitter(),
                #[cfg(feature = "debug")]
                debug: &mut load.debug,
            },
            user_location,
            user_country,
        );

        load.is_enabled = status.status & STATUS_ENABLED != 0;
        load.is_visible = status.status & STATUS_VISIBLE != 0;
        load.is_autoconnectable = status.status & STATUS_AUTOCONNECTABLE != 0;
        load.load = status.load;
        load.score = score;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::country::Country;
    use crate::location::Location;
    use crate::logical::StatusReference;

    fn create_dummy_location() -> Location {
        Location {
            latitude: 0.0,
            longitude: 0.0,
        }
    }

    fn create_dummy_country() -> Country {
        Country::try_from(b"CH").expect("Invalid country code")
    }

    #[test]
    fn test_compute_loads_error_lengths_not_consistent() {
        let lengths = [(1, 2), (1, 2)];

        for (loads_len, servers_len) in lengths {
            let mut loads = Vec::new();
            loads.resize(loads_len, Load::default());

            let mut servers = Vec::new();
            servers.resize(servers_len, Logical::default());

            let error = compute_loads(
                &mut loads,
                &servers,
                &[1_u8, 0_u8, 0_u8, 0_u8],
                &Some(create_dummy_location()),
                &Some(create_dummy_country()),
            )
            .unwrap_err();

            match error {
                Error::LengthsNotConsistent { servers, loads } => {
                    assert_eq!(servers, 2);
                    assert_eq!(loads, 1);
                }
                _ => panic!("Expected LengthsNotConsistent error"),
            }
        }
    }

    #[test]
    fn test_compute_loads_status_flags() {
        let servers = vec![
            Logical {
                status_reference: StatusReference {
                    index: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
            Logical {
                status_reference: StatusReference {
                    index: 1,
                    ..Default::default()
                },
                ..Default::default()
            },
            Logical {
                status_reference: StatusReference {
                    index: 2,
                    ..Default::default()
                },
                ..Default::default()
            },
            Logical {
                status_reference: StatusReference {
                    index: 3,
                    ..Default::default()
                },
                ..Default::default()
            },
        ];

        let mut loads = Vec::new();
        loads.resize(servers.len(), Load::default());

        let status_file = [
            1_u8, 0_u8, 0_u8, 0_u8, // Version
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, // Disabled, not visible
            1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, // Enabled, not visible
            2_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, // Disabled, visible
            3_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, // Enabled, visible
        ];

        compute_loads(
            &mut loads,
            &servers,
            &status_file,
            &Some(create_dummy_location()),
            &Some(create_dummy_country()),
        )
        .expect("Failed to compute loads");

        assert!(!loads[0].is_enabled);
        assert!(!loads[0].is_visible);

        assert!(loads[1].is_enabled);
        assert!(!loads[1].is_visible);

        assert!(!loads[2].is_enabled);
        assert!(loads[2].is_visible);

        assert!(loads[3].is_enabled);
        assert!(loads[3].is_visible);
    }
}
