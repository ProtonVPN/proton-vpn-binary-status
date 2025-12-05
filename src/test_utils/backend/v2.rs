use super::*;
use crate::compute_loads;

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Default, Clone, PartialEq,
)]
pub struct Server {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(flatten)]
    pub logical: Logical,
}

#[derive(
    serde::Serialize, serde::Deserialize, Debug, Clone, Default, PartialEq,
)]
pub struct Logicals {
    #[serde(rename = "StatusID")]
    pub status_id: String,

    #[serde(rename = "LogicalServers")]
    pub logical_servers: Vec<Server>,
}

pub fn patch_secure_core(logicals: &mut super::v2::Logicals) {
    let secure_core_prefixes = [
        ("CH-", (46.818188, 8.227512)),
        ("IS-", (64.963051, -19.020835)),
        ("SE-", (60.128161, 18.643501)),
    ];
    for server in logicals.logical_servers.iter_mut() {
        for (prefix, location) in secure_core_prefixes.iter() {
            if server.name.starts_with(prefix) {
                log::info!(
                    "Patching secure core server {} location to {:?}",
                    server.name,
                    location
                );
                server.logical.entry_location = crate::Location {
                    latitude: location.0 as f32,
                    longitude: location.1 as f32,
                };
                break;
            }
        }
    }
}

pub async fn get_logicals(
    endpoints: &mut Endpoints,
    filter: impl Fn(&super::v2::Server) -> bool,
) -> Result<super::v1::Logicals> {
    let mut logicals: Logicals =
        endpoints.get_deserialized("vpn/v2/logicals", None).await?;

    logicals.logical_servers.retain(filter);

    #[cfg(feature = "legacy")]
    {
        patch_secure_core(&mut logicals);
    }

    let status_endpoints =
        format!("vpn/v2/status/{}/binary", logicals.status_id);
    let status = endpoints.get(&status_endpoints, None).await?;

    let servers = logicals
        .logical_servers
        .iter()
        .map(|server| server.logical.clone())
        .collect::<Vec<_>>();

    let mut loads = Vec::new();
    loads.resize(logicals.logical_servers.len(), super::Load::default());
    compute_loads(
        &mut loads,
        &servers,
        &status,
        &Some(Location {
            latitude: USER_LATITUDE,
            longitude: USER_LONGITUDE,
        }),
        &Some(Country::try_from(USER_COUNTRY)?),
    )?;

    let mut logicals_v1 = super::v1::Logicals {
        logical_servers: Vec::new(),
    };

    for (server, load) in logicals.logical_servers.iter().zip(loads) {
        logicals_v1.logical_servers.push(super::v1::Server {
            name: server.name.clone(),
            status: if load.is_enabled { 1 } else { 0 },
            load: load.load,
            score: load.score,
            score_jitter_bps: 0.0,
            #[cfg(feature = "debug")]
            debug: load.debug.clone(),
        });
    }

    logicals_v1
        .logical_servers
        .sort_by(|a, b| a.name.cmp(&b.name));

    Ok(logicals_v1)
}
