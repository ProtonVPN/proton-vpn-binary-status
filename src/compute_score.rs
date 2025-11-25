// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------
use super::coord::Coord;
use super::country::Country;
#[cfg(feature = "debug")]
use super::load::LoadDebugFields;
use super::location::Location;
// -----------------------------------------------------------------------------
const PARTIAL_SCORE_CEILING: f64 = 0.99; // = normalize(100.0) = (10000 - 100) / 10000
const BANDWITH_DISTANCE_FACTOR: f64 = 738_000.0; // Mbps/km

pub(crate) const SCORE_NORMALIZATION_FACTOR: f64 = 10_000.0; // Mbps (10 Gbps)

// The server load jitter range is 100 Mbps. It oscillates between -50 and +50 Mbps.
// The jitter is applied after the load is normalized and so we must also
// normalize it. As the jitter is an offset, and not an absolute value,
// normalization is done by dividing the jitter by the SCORE_NORMALIZATION_FACTOR
// and negating it.
#[cfg(feature = "jitter")]
pub(crate) const NORMALIZED_JITTER_RANGE: f64 =
    -(100.0 / SCORE_NORMALIZATION_FACTOR); // (100 Mbps)  normalized to 0.01

pub const STATUS_ENABLED: u8 = 1 << 0;
pub const STATUS_VISIBLE: u8 = 1 << 1;
pub const STATUS_AUTOCONNECTABLE: u8 = 1 << 2;

pub struct ComputeScoreServerParams<'a> {
    pub status_penalty: f64,
    pub status_cost: u8,
    pub country: Country,
    pub partial_score: f64,
    pub status: u8,
    pub exit_location: &'a Location, // 0 = Lat, 1 = Long
    pub entry_location: &'a Location,
    pub normalized_jitter: f64,
    #[cfg(feature = "debug")]
    pub debug: &'a mut LoadDebugFields,
}

pub(crate) fn normalize(score: f64) -> f64 {
    (SCORE_NORMALIZATION_FACTOR - score) / SCORE_NORMALIZATION_FACTOR
}

fn compute_distance_between(a: &Location, b: &Location) -> f64 {
    Coord::from_degrees(a.latitude as f64, a.longitude as f64).distance_from(
        &Coord::from_degrees(b.latitude as f64, b.longitude as f64),
    )
}

// Depending on whether we are using legacy mode or not,
// the travel distance is computed differently.
//
// In legacy mode, we calculate the distance from the client to the server as:
//   distance(client -> server_exit)
// In non-legacy mode, we calculate the distance from the client to the server as:
//   distance(client -> server_entry)

#[cfg(feature = "legacy")]
pub fn compute_travel_distance(
    server_exit_location: &Location,
    server_entry_location: &Location,
    client_position: &Location,
) -> f64 {
    compute_distance_between(client_position, server_exit_location)
        + compute_distance_between(server_entry_location, server_exit_location)
}

#[cfg(not(feature = "legacy"))]
pub fn compute_travel_distance(
    server_exit_location: &Location,
    server_entry_location: &Location,
    client_position: &Location,
) -> f64 {
    compute_distance_between(client_position, server_entry_location)
        + compute_distance_between(server_entry_location, server_exit_location)
}

pub(crate) fn compute_distance_score(
    server_exit_location: &Location,
    server_entry_location: &Location,
    client_position: &Option<Location>,
) -> f64 {
    let distance_in_km = if let Some(client_position) = client_position {
        compute_travel_distance(
            server_exit_location,
            server_entry_location,
            client_position,
        )
    } else {
        0.0
    };

    let proximity_based_bandwidth_estimate =
        BANDWITH_DISTANCE_FACTOR / f64::max(1.0, distance_in_km);

    normalize(proximity_based_bandwidth_estimate)
}

pub(crate) fn compute_penalty(
    status_penalty: f64,
    status_cost: u8,
    norm_server_available_bandwidth_for_session: f64,
    client_country: &Option<Country>,
    server_country: Country,
    server_status: u8,
) -> f64 {
    let is_in_same_country = if let Some(country) = client_country {
        (*country) == server_country
    } else {
        true
    };

    //--------------------------------------------------------------------------
    // The server side penalties
    //--------------------------------------------------------------------------
    let mut penalty = status_penalty;

    //--------------------------------------------------------------------------
    // The client side penalties
    //--------------------------------------------------------------------------
    let server_disabled = (server_status & STATUS_ENABLED) == 0;
    let server_hidden = (server_status & STATUS_VISIBLE) == 0;
    if server_disabled || server_hidden {
        penalty += 1000.0;
    }

    if (!is_in_same_country)
        || norm_server_available_bandwidth_for_session >= PARTIAL_SCORE_CEILING
    {
        penalty += 1.0;
    }

    if (!is_in_same_country) && status_cost == 1_u8 {
        penalty += 3.0;
    }

    penalty
}

/// Computes the loads from an array of servers and a status file.
pub fn compute_score(
    server: ComputeScoreServerParams,
    user_location: &Option<Location>,
    user_country: &Option<Country>,
) -> f64 {
    let distance_score = compute_distance_score(
        server.exit_location,
        server.entry_location,
        user_location,
    );

    let capped_score = f64::max(distance_score, server.partial_score);

    let base_score = (capped_score + server.normalized_jitter).clamp(0.0, 1.0);

    // Additional debug information
    #[cfg(feature = "debug")]
    {
        server.debug.partial_score = server.partial_score;
    }

    let penalty = compute_penalty(
        server.status_penalty,
        server.status_cost,
        server.partial_score,
        user_country,
        server.country,
        server.status,
    );

    base_score + penalty
}

#[cfg(test)]
mod tests {
    use super::*;

    type AnyResult = Result<(), Box<dyn std::error::Error>>;

    #[test_log::test]
    fn test_normalize() -> AnyResult {
        assert_eq!(normalize(SCORE_NORMALIZATION_FACTOR), 0.0);
        assert_eq!(normalize(0.0), 1.0);
        assert_eq!(normalize(5000.0), 0.5);
        assert_eq!(normalize(2500.0), 0.75);
        assert_eq!(normalize(7500.0), 0.25);

        Ok(())
    }

    #[test_log::test]
    fn test_compute_distance_score() -> AnyResult {
        let server_exit = Location {
            latitude: 48.8566, // Paris
            longitude: 2.3522,
        };
        let client_location = Location {
            latitude: 51.5074, // London
            longitude: -0.1278,
        };

        let distance_to_score = |distance_in_km: f64| {
            normalize(BANDWITH_DISTANCE_FACTOR / f64::max(1.0, distance_in_km))
        };

        // Test with no client location, or server entry location
        // (i.e. server exit location only)
        let score = compute_distance_score(&server_exit, &server_exit, &None);
        assert_eq!(
            score,
            1.0 - (BANDWITH_DISTANCE_FACTOR / SCORE_NORMALIZATION_FACTOR)
        );

        // Test with a known client location and no server entry location.
        let score = compute_distance_score(
            &server_exit,
            &server_exit,
            &Some(client_location.clone()),
        );
        assert!((0.0..=1.0).contains(&score));

        // Test with a known client location, a known server entry location
        // and a known server exit location.
        let server_entry = Location {
            latitude: 40.4168, // Madrid
            longitude: -3.7038,
        };
        let score = compute_distance_score(
            &server_exit,
            &server_entry.clone(),
            &Some(client_location.clone()),
        );

        let distance_in_km =
            compute_distance_between(&server_exit, &server_entry)
                + compute_distance_between(&server_entry, &client_location);
        assert_eq!(score, distance_to_score(distance_in_km));

        Ok(())
    }

    #[test_log::test]
    fn test_compute_penalty() -> AnyResult {
        assert_eq!(
            0.0, // Everything is optimal
            compute_penalty(
                0.0,
                0_u8,
                0.5,
                &Some(Country::new(b"FR")?),
                Country::new(b"FR")?,
                STATUS_ENABLED | STATUS_VISIBLE
            )
        );
        assert_eq!(
            123.0, // Because the status penalty is high
            compute_penalty(
                123.0,
                0_u8,
                0.5,
                &Some(Country::new(b"FR")?),
                Country::new(b"FR")?,
                STATUS_ENABLED | STATUS_VISIBLE
            )
        );
        assert_eq!(
            1.0, // Because load score is high
            compute_penalty(
                0.0,
                0_u8,
                0.99,
                &Some(Country::new(b"FR")?),
                Country::new(b"FR")?,
                STATUS_ENABLED | STATUS_VISIBLE
            )
        );
        assert_eq!(
            1.0, // Because countries are different
            compute_penalty(
                0.0,
                0_u8,
                0.5,
                &Some(Country::new(b"FR")?),
                Country::new(b"GB")?,
                STATUS_ENABLED | STATUS_VISIBLE
            )
        );
        assert_eq!(
            4.0, // Because countries are different and cost is 1
            compute_penalty(
                0.0,
                1_u8,
                0.5,
                &Some(Country::new(b"FR")?),
                Country::new(b"GB")?,
                STATUS_ENABLED | STATUS_VISIBLE
            )
        );
        assert_eq!(
            1000.0, // Because server active but not visible
            compute_penalty(
                0.0,
                0_u8,
                0.5,
                &Some(Country::new(b"FR")?),
                Country::new(b"FR")?,
                STATUS_ENABLED
            )
        );
        assert_eq!(
            1000.0, // Because server is visible but not active
            compute_penalty(
                0.0,
                0_u8,
                0.5,
                &Some(Country::new(b"FR")?),
                Country::new(b"FR")?,
                STATUS_VISIBLE
            )
        );
        assert_eq!(
            1000.0, // Because server is disabled and hidden
            compute_penalty(
                0.0,
                0_u8,
                0.5,
                &Some(Country::new(b"FR")?),
                Country::new(b"FR")?,
                0_u8
            )
        );

        assert_eq!(
            0.0, // We do not know if countries are different, so we assume
            // they are the same and therefore everything is optimal.
            compute_penalty(
                0.0,
                0_u8,
                0.5,
                &None,
                Country::new(b"FR")?,
                STATUS_ENABLED | STATUS_VISIBLE
            )
        );

        Ok(())
    }

    #[test_log::test]
    fn test_compute_score() -> AnyResult {
        let paris = Location {
            latitude: 48.8566, // Paris
            longitude: 2.3522,
        };
        let toulouse = Location {
            latitude: 43.6047, // Toulouse
            longitude: 1.4442,
        };

        #[cfg(feature = "debug")]
        let mut debug_fields = LoadDebugFields::default();

        // In same country, far enough to apply distance cap.
        let score = compute_score(
            ComputeScoreServerParams {
                status_penalty: 0.0,
                status_cost: 0_u8,
                country: Country::new(b"FR")?,
                partial_score: 0.5,
                status: STATUS_ENABLED | STATUS_VISIBLE,
                exit_location: &paris,
                entry_location: &paris,
                normalized_jitter: 0_f64,
                #[cfg(feature = "debug")]
                debug: &mut debug_fields,
            },
            &Some(toulouse.clone()),
            &Some(Country::new(b"FR")?),
        );

        assert_eq!(
            compute_distance_score(&paris, &paris, &Some(toulouse)),
            score
        );

        #[cfg(feature = "debug")]
        assert_eq!(debug_fields.partial_score, 0.5);

        // In different countries, close enough to avoid distance cap,
        // with penalty of 1.
        let score = compute_score(
            ComputeScoreServerParams {
                status_penalty: 0.0,
                status_cost: 0_u8,
                country: Country::new(b"FR")?,
                partial_score: 0.5,
                status: STATUS_ENABLED | STATUS_VISIBLE,
                exit_location: &Location {
                    latitude: 45.8992, // Annecy
                    longitude: 6.1294,
                },
                entry_location: &Location {
                    latitude: 45.8992, // Annecy
                    longitude: 6.1294,
                },
                normalized_jitter: 0_f64,
                #[cfg(feature = "debug")]
                debug: &mut debug_fields,
            },
            &Some(Location {
                latitude: 46.2044, // Geneva
                longitude: 6.1432,
            }),
            &Some(Country::new(b"CH")?),
        );

        assert_eq!(1.5, score);
        #[cfg(feature = "debug")]
        assert_eq!(debug_fields.partial_score, 0.5);

        Ok(())
    }
}
