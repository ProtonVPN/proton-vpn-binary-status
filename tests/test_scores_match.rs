#[cfg(feature = "test_utils_backend")]
use anyhow::Result;

#[cfg(feature = "test_utils_backend")]
use proton_vpn_binary_status::test_utils::backend;

#[cfg(feature = "test_utils_backend")]
async fn compare_scores(
    acceptable_variance: f64,
    default_cache_dir: &str,
    filter_query: impl Fn(&str) -> bool,
    filter_comparison: impl Fn(&backend::v1::Server, &backend::v1::Server) -> bool,
) -> Result<()> {
    let mut v1_endpoint = backend::Endpoints::new(default_cache_dir).await?;
    let mut v2_endpoint = v1_endpoint.clone();

    let (v1, v2) = futures::join!(
        backend::v1::get_logicals(&mut v1_endpoint, |s| filter_query(&s.name)),
        backend::v2::get_logicals(&mut v2_endpoint, |s| filter_query(&s.name)),
    );

    let v1 = v1?;
    let v2 = v2?;

    let v2_lookup: std::collections::HashMap<&str, backend::v1::Server> = v2
        .logical_servers
        .iter()
        .map(|s| (s.name.as_str(), s.clone()))
        .collect();

    let mut result = true;

    let mut total = 0;
    let mut total_matching_filter = 0;
    let mut mismatched = 0;
    for a in v1.logical_servers {
        if let Some(b) = v2_lookup.get(a.name.as_str()) {
            let variance = backend::compute_variance(&a, b);
            if filter_comparison(&a, b) {
                if variance > acceptable_variance {
                    log::info!(
                        "v1 name={:10} status={} load={:3} score={:.6}",
                        a.name,
                        a.status,
                        a.load,
                        a.score
                    );
                    log::info!("\x1b[90mv2 name={:10} status={} load={:3} score={:.6}\x1b[0m", b.name, b.status, b.load, b.score);
                    log::info!("Variance: {:.4} % ({:.6} Mbps for {:.4} Mbps tolerance)", variance * 100.0, variance * 10_000.0, acceptable_variance * 10_000.0);
                    #[cfg(feature = "debug")]
                    {
                        log::info!("    v1 partial_score=...");
                        log::info!(
                            "    v2 partial_score={:.5}",
                            b.debug.partial_score
                        );
                    }
                    result = false;
                    mismatched += 1;
                }
                total_matching_filter += 1;
            }
        } else {
            log::info!("Server {} not found in v2 logicals", a.name);
            mismatched += 1;
        }
        total += 1;
    }

    log::info!(
        "Total servers checked: {}, with matching filter: {} mismatched:{} ({:.3}%)\n",
        total,
        total_matching_filter,
        mismatched,
        (mismatched as f64 / total_matching_filter as f64) * 100.0
    );

    assert!(result, "Logicals comparison failed");

    Ok(())
}

#[cfg(feature = "test_utils_backend")]
#[test_log::test(tokio::test)]
async fn test_scores_match() -> Result<()> {
    // Assuming there is no jitter between the v1 and v2 calculations,
    // and assuming that both logicals have the same load values,
    // an acceptable variance between the two scores is the margin of
    // 1%.
    //
    // This is because the load is given as a whole number percentage,
    // and so logicals with the same load can still have a score variance
    // of up to just under 1% of the maximum bandwidth (10 Gbps)
    //
    // Therefore the acceptable variance (1%) as a fraction of 1 is 0.01.
    // Which is 0.01 * 10 Gbps = 100 Mbps.
    const ACCEPTABLE_VARIANCE: f64 = 0.01;
    compare_scores(
        ACCEPTABLE_VARIANCE,
        "./tests/resources/endpoints_variable",
        |_| true,
        |_, _| true,
    )
    .await
}

#[cfg(feature = "test_utils_backend")]
#[test_log::test(tokio::test)]
async fn test_scores_match_exactly() -> Result<()> {
    // Here we are using a dataset where the load values are exactly the same
    // between v1 and v2 logicals, so we can expect the scores to match
    // very closely.
    // Therefore we set an acceptable variance of 0.0001 (0.01%),
    // which is 0.0001 * 10 Gbps = 1 Mbps.
    const ACCEPTABLE_VARIANCE: f64 = 0.0001;
    compare_scores(ACCEPTABLE_VARIANCE, "./tests/resources/endpoints_exact",
                  |_| true,
                |v1_logical, v2_logical| {
                    let result = v1_logical.status != 0 && v2_logical.status != 0;
                    if !result {
                        log::info!("\x1b[90mSkipping comparison for logical {} status is v1={} v2={}\x1b[0m",
                                   v1_logical.name, v1_logical.status, v2_logical.status);
                    }
                    result
                }
    ).await
}
