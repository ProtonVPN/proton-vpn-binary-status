// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------
#[cfg(feature = "jitter")]
pub fn generator() -> impl FnMut() -> f64 {
    use crate::compute_score::NORMALIZED_JITTER_RANGE;
    use rand::Rng as _;

    let mut rng = rand::rng();
    move || rng.random_range(-0.5..0.5) * NORMALIZED_JITTER_RANGE
}

#[cfg(not(feature = "jitter"))]
pub fn generator() -> impl FnMut() -> f64 {
    move || 0_f64
}
