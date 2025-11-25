use super::Result;

#[derive(Debug, Clone)]
pub enum Endpoints {
    Client((muon::Client, std::path::PathBuf)),
    Cache(std::path::PathBuf),
}

impl Endpoints {
    pub async fn new(default_cache_dir: &str) -> Result<Self> {
        let cache = std::path::PathBuf::from(
            std::env::var("PROTON_VPN_BINARY_STATUS_TEST_CACHE")
                .unwrap_or(default_cache_dir.into()),
        );

        let user = std::env::var("PROTON_VPN_BINARY_STATUS_TEST_USER");
        if user.is_ok() {
            let password =
                std::env::var("PROTON_VPN_BINARY_STATUS_TEST_PASSWORD")?;
            let two_fa = std::env::var("PROTON_VPN_BINARY_STATUS_TEST_2FA")?;
            Ok(Self::Client((
                Self::login(&user?, &password, &two_fa).await?,
                cache,
            )))
        } else {
            Ok(Self::Cache(cache))
        }
    }

    pub async fn new_from_params(
        user: &str,
        password: &str,
        two_fa: &str,
    ) -> Result<Self> {
        Ok(Self::Client((
            Self::login(user, password, two_fa).await?,
            std::path::PathBuf::new(),
        )))
    }

    pub async fn login(
        user: &str,
        password: &str,
        two_fa: &str,
    ) -> Result<muon::Client> {
        // First, define which app is using the client.
        let app = muon::App::new("windows-vpn@4.1.0")?; // TODO: replace with something generic

        let store = Storage::default();
        let client = muon::Client::new(app, store)?;
        let auth = client.auth();

        let extra_info = muon::client::flow::LoginExtraInfo::builder().build();

        // We can use the auth flow to login.
        let client =
            match auth.login_with_extra(user, password, extra_info).await {
                muon::client::flow::LoginFlow::Ok(client, _) => client,
                muon::client::flow::LoginFlow::TwoFactor(client, _) => {
                    if client.has_totp() {
                        client.totp(&two_fa).await?
                    } else if client.has_fido() {
                        unimplemented!()
                    } else {
                        panic!("no 2FA available");
                    }
                }

                muon::client::flow::LoginFlow::Failed { reason, client } => {
                    log::error!(
                        "Login failure: {reason}, client is staying un-logged."
                    );
                    client
                }
            };

        Ok(client)
    }

    fn create_dir_all(path: &std::path::Path) -> Result<()> {
        if !path.exists() {
            if let Some(parent) = path.parent() {
                Self::create_dir_all(parent)?;
            }
            std::fs::create_dir(path)?;
        }
        Ok(())
    }

    pub async fn get(
        &mut self,
        endpoint: &str,
        header: Option<(&str, &str)>,
    ) -> Result<Vec<u8>> {
        match &self {
            Self::Client((client, path)) => {
                let mut http_request = muon::GET!("/{}", endpoint)
                    .query(("SecureCoreFilter", "all"))
                    .query(("WithEntryLocation", "true"));

                if let Some(header) = header {
                    http_request = http_request.header(header);
                }

                let request = client.send(http_request).await?.body().to_vec();

                log::info!("Test cache {}", path.display());

                if path.components().count() != 0 {
                    let endpoint_path = path.join(endpoint);

                    log::info!(
                        "Creating cache folder {}",
                        endpoint_path.display()
                    );

                    Self::create_dir_all(endpoint_path.parent().ok_or_else(
                        || {
                            anyhow::anyhow!(
                                "Invalid endpoint path: {}",
                                endpoint_path.display()
                            )
                        },
                    )?)?;

                    use std::io::Write;
                    let mut file = std::fs::File::create(endpoint_path)?;
                    file.write_all(&request)?;
                }

                Ok(request)
            }
            Self::Cache(path) => {
                let mut buffer = Vec::new();
                if path.components().count() != 0 {
                    use std::io::Read;
                    log::info!("Reading cache file {:?}", path.join(endpoint));
                    let mut file = std::fs::File::open(path.join(endpoint))?;
                    file.read_to_end(&mut buffer)?;
                }
                Ok(buffer)
            }
        }
    }

    pub async fn get_deserialized<T>(
        &mut self,
        endpoint: &str,
        header: Option<(&str, &str)>,
    ) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.get(endpoint, header).await?;
        Ok(serde_json::from_slice(&data)?)
    }
}

#[derive(Debug)]
pub struct Storage(muon::env::EnvId, muon::client::Auth);

impl Default for Storage {
    fn default() -> Self {
        Self(muon::env::EnvId::new_prod(), Default::default())
    }
}

#[async_trait::async_trait]
impl muon::store::Store for Storage {
    // retrieve the env
    fn env(&self) -> muon::env::EnvId {
        self.0.clone()
    }
    // retrieve the auth
    async fn get_auth(&self) -> muon::client::Auth {
        self.1.clone()
    }
    // set the auth
    async fn set_auth(
        &mut self,
        auth: muon::client::Auth,
    ) -> Result<muon::client::Auth, muon::store::StoreError> {
        self.1 = auth;
        // retrieve the auth that is currently stored
        Ok(self.get_auth().await)
    }
}
