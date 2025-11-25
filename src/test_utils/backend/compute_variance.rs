use super::*;
use crate::compute_score::SCORE_NORMALIZATION_FACTOR;

pub fn match_jitter(a: &v1::Server, b: &v1::Server) -> (f64, f64) {
    let score_jitter_mbps = a.score_jitter_bps / 1_000_000.0;
    let a_score = a.score;
    let b_score = (b.score.fract()
        - (score_jitter_mbps / SCORE_NORMALIZATION_FACTOR))
        .clamp(0.0, 1.0)
        + b.score.trunc();

    (a_score, b_score)
}

pub fn compute_variance(a: &v1::Server, b: &v1::Server) -> f64 {
    let (a_score, b_score) = match_jitter(a, b);
    (a_score - b_score).abs()
}
