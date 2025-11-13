const RADIUS_OF_THE_EARTH: f64 = 6_371.0; // km

/// Represents a geographical coordinate in radians.
// It is used to compute the arc distance between two coordinates.
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

impl Coord {
    /// Creates a new coordinate from latitude and longitude in radians.
    pub fn from_degrees(lat: f64, lon: f64) -> Self {
        let deg2rad = std::f64::consts::PI / 180.0;
        Self {
            lat: lat * deg2rad,
            lon: lon * deg2rad,
        }
    }

    /// Computes the arc distance in kilometers from this coordinate to
    /// the given coordinate 'b'.
    pub fn distance_from(&self, b: &Coord) -> f64 {
        let lat_from = self.lat;
        let lon_from = self.lon;
        let lat_to = b.lat;
        let lon_to = b.lon;

        let lat_delta = lat_to - lat_from;
        let lon_delta = lon_to - lon_from;

        let sq = |a| a * a;

        let angle = 2.0
            * f64::asin(f64::sqrt(
                sq(f64::sin(lat_delta / 2.0))
                    + (f64::cos(lat_from)
                        * f64::cos(lat_to)
                        * sq(f64::sin(lon_delta / 2.0))),
            ));

        RADIUS_OF_THE_EARTH * angle
    }
}

#[cfg(test)]
mod tests {
    use super::Coord;
    use core::f64;

    #[test_log::test]
    fn test_coord_from_degrees() {
        let coord = Coord::from_degrees(180.0, 90.0);
        assert_eq!(coord.lat, std::f64::consts::PI);
        assert_eq!(coord.lon, std::f64::consts::PI / 2.0);
    }

    #[test_log::test]
    // Coord::distance_from is based on the Haversine formula below.
    //
    // If you run the following PHP code, you will get results which have less
    // than one micrometer difference from the Rust result.
    //
    // <?php
    // haversineGreatCircleDistance
    // function distance(
    //     float|int $latitudeFrom,
    //     float|int $longitudeFrom,
    //     float|int $latitudeTo,
    //     float|int $longitudeTo,
    //     float|int $earthRadius = 6371.000,
    // ): float {
    //     $latFrom = deg2rad($latitudeFrom);
    //     $lonFrom = deg2rad($longitudeFrom);
    //     $latTo = deg2rad($latitudeTo);
    //     $lonTo = deg2rad($longitudeTo);

    //     $latDelta = $latTo - $latFrom;
    //     $lonDelta = $lonTo - $lonFrom;

    //     $angle = 2 * asin(sqrt(sin($latDelta / 2) ** 2 +
    //                             cos($latFrom) * cos($latTo) * sin($lonDelta / 2) ** 2));

    //     return $angle * $earthRadius;
    // }

    // $countries = [
    //     [46.204391, 6.143158],   // Geneva
    //     [48.864716, 2.349014],   // Paris
    //     [-41.28664, 174.77557],  // Wellington
    //     [47.36667, 8.55],        // Zurich
    //     [40.730610, -73.935242], // New york
    // ];

    // foreach ($countries as $from) {
    //     foreach ($countries as $to) {
    //         $distance = distance( $from[0], $from[1], $to[0], $to[1] );
    //         if ($distance != 0.0) {
    //             echo number_format($distance, 15, ".", ""); echo "\n";
    //         }
    //     }
    //     echo "\n";
    // }
    // ?>
    fn test_coord_distance_between_cities() {
        const COORDS: [[f64; 2]; 5] = [
            [46.204391, 6.143158],   // Geneva
            [48.864716, 2.349014],   // Paris
            [-41.28664, 174.77557],  // Wellington
            [47.36667, 8.55],        // Zurich
            [40.730610, -73.935242], // New york
        ];

        const CITIES: [&str; 5] =
            ["Geneva", "Paris", "Wellington", "Zurich", "New York"];

        const DISTANCES: [f64; 20] = [
            // Geneva
            410.554_985_724_153_54, // Paris
            18_952.243_867_432_37,  // Wellington
            224.225_491_683_033_65, // Zurich
            6_210.329_769_582_576,  // New York
            // Paris
            410.554_985_724_153_54, // Geneva
            18_984.858_178_220_91,  // Wellington
            489.377_499_064_556_56, // Zurich
            5_830.710_905_402_295,  // New York
            // Wellington
            18_952.243_867_432_37,  // Geneva
            18_984.858_178_220_91,  // Paris
            18_730.398_033_198_726, // Zurich
            14_409.665_181_275_78,  // New York
            // Zurich
            224.225_491_683_033_65, // Geneva
            489.377_499_064_556_56, // Paris
            18_730.398_033_198_726, // Wellington
            6_318.845_334_665_826,  // New York
            // New York
            6_210.329_769_582_576, // Geneva
            5_830.710_905_402_295, // Paris
            14_409.665_181_275_78, // Wellington
            6_318.845_334_665_826, // Zurich
        ];

        const MICROMETER: f64 = 1.0 / 1_000_000_000.0; // 1000,000,000

        let mut k = 0;
        for j in 0..CITIES.len() {
            for i in 0..CITIES.len() {
                let a = Coord::from_degrees(COORDS[j][0], COORDS[j][1]);
                let b = Coord::from_degrees(COORDS[i][0], COORDS[i][1]);
                let distance = a.distance_from(&b);
                if distance != 0.0 {
                    log::info!(
                        "{} -> {} = {} == {}",
                        CITIES[j],
                        CITIES[i],
                        DISTANCES[k],
                        distance
                    );
                    assert!((distance - DISTANCES[k]).abs() < MICROMETER);
                    k += 1;
                }
            }
        }
    }
}
