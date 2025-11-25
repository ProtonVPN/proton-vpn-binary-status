#[cfg(feature = "test_utils_backend")]
use anyhow::Result;

#[cfg(feature = "test_utils_backend")]
use proton_vpn_binary_status::test_utils::backend;

#[cfg(feature = "test_utils_backend")]
#[test_log::test(tokio::test)]
async fn test_ranking_order() -> Result<()> {
    let mut v1_endpoint =
        backend::Endpoints::new("./tests/resources/endpoints_exact").await?;
    let mut v2_endpoint = v1_endpoint.clone();

    let (v1, v2) = futures::join!(
        backend::v1::get_logicals(&mut v1_endpoint, |_| true),
        backend::v2::get_logicals(&mut v2_endpoint, |_| true),
    );

    let lookup: std::collections::HashMap<String, backend::v1::Server> = v2?
        .logical_servers
        .iter()
        .map(|s| (s.name.clone(), s.clone()))
        .collect();

    let mut v1_s = Vec::new();
    let mut v2_s = Vec::new();

    for i in v1?.logical_servers.iter() {
        if let Some(v2_server) = lookup.get(&i.name) {
            if i.load == v2_server.load {
                let (v1_score, v2_score) =
                    backend::compute_variance::match_jitter(i, v2_server);
                let mut v1_logical = i.clone();
                let mut v2_logical = v2_server.clone();
                v1_logical.score = v1_score;
                v2_logical.score = v2_score;
                v1_s.push(v1_logical);
                v2_s.push(v2_logical);
            }
        }
    }

    v1_s.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .expect("Failed to compare scores")
    });
    v2_s.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .expect("Failed to compare scores")
    });

    for (i, j) in v1_s.iter().zip(v2_s.iter()) {
        assert_eq!(i.name, j.name, "Ranking order mismatch");
        log::info!(
            "{:13} {:13} with   {:.8} {:.8}",
            i.name,
            j.name,
            i.score,
            j.score
        );
    }

    Ok(())
}
