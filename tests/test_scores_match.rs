#[cfg(feature = "test_utils_backend")]
use anyhow::Result;

#[cfg(feature = "test_utils_backend")]
use proton_vpn_binary_status::test_utils::backend;

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

    let mut v1_endpoint = backend::Endpoints::new().await?;
    let mut v2_endpoint = v1_endpoint.clone();

    let (v1, v2) = futures::join!(
        backend::v1::get_logicals(&mut v1_endpoint),
        backend::v2::get_logicals(&mut v2_endpoint),
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
    let mut mismatched = 0;
    for a in v1.logical_servers {
        //assert_eq!(a.name, b.name, "Server names do not match");

        if let Some(b) = v2_lookup.get(a.name.as_str()) {
            let variance = backend::compute_variance_mbps(&a, b);
            if a.load == b.load && variance > ACCEPTABLE_VARIANCE {
                log::info!("v1 name={} load={} {:.5}", a.name, a.load, a.score);
                log::info!("v2 name={} load={} {:.5}", b.name, b.load, b.score);
                log::info!("Variance: {:.4} %", variance * 100.0);
                result = false;
                mismatched += 1;
            }
        } else {
            log::info!("Server {} not found in v2 logicals", a.name);
            mismatched += 1;
        }
        total += 1;
    }

    log::info!(
        "Total servers checked: {}, mismatched:{:.3} %\n",
        total,
        (mismatched as f64 / total as f64) * 100.0
    );

    assert!(result, "Logicals comparison failed");

    Ok(())
}
