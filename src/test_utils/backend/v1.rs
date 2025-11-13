use super::{Endpoints, Result, NETZONE_HEADER, USER_IP_ADDRESS};
#[derive(
    serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq,
)]
pub struct Location {
    #[serde(rename = "Lat")]
    pub latitude: f32,
    #[serde(rename = "Long")]
    pub longitude: f32,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq,
)]
pub struct Server {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Status")]
    pub status: u8,
    #[serde(rename = "Load")]
    pub load: u8,
    #[serde(rename = "Score")]
    pub score: f64,

    // The below field is for debugging only, speak to backend team if you need
    // it.
    #[serde(rename = "ScoreJitter")]
    pub score_jitter_bps: f64,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Default, PartialEq,
)]
pub struct Logicals {
    #[serde(rename = "LogicalServers")]
    pub logical_servers: Vec<Server>,
}

pub async fn get_logicals(endpoints: &mut Endpoints) -> Result<Logicals> {
    let mut logicals: Logicals = endpoints
        .get_deserialized(
            "vpn/v1/logicals",
            Some((NETZONE_HEADER, USER_IP_ADDRESS)),
        )
        .await?;

    logicals.logical_servers.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(logicals)
}
