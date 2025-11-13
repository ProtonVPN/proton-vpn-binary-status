// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ServerStatusError(pub(crate) String);
pub type ServerStatusResult<T> = std::result::Result<T, ServerStatusError>;

pub fn validate_server(
    server: ServerStatus,
) -> ServerStatusResult<ServerStatus> {
    if server.load > 100 {
        return Err(ServerStatusError(
            "Server load must be between 0 and 100".into(),
        ));
    }

    if server.partial_score < 0.0 || server.partial_score > 1.0 {
        return Err(ServerStatusError(
            "Server partial score must be between 0.0 and 1.0".into(),
        ));
    }

    Ok(server)
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServerStatus {
    pub status: u8,
    pub load: u8,
    pub partial_score: f32, // This is a 32-bit float to match the binary format
                            // of binary status files.
                            // We keep it as a 32-bit float to keep the size
                            // small.
}

impl From<&[u8; 6]> for ServerStatus {
    fn from(src: &[u8; 6]) -> Self {
        Self {
            status: src[0],
            load: src[1],
            partial_score: f32::from_le_bytes([src[2], src[3], src[4], src[5]]),
        }
    }
}

impl TryFrom<&[u8]> for ServerStatus {
    type Error = ServerStatusError;
    fn try_from(src: &[u8]) -> ServerStatusResult<Self> {
        let bytes: &[u8; 6] = src.try_into().map_err(|err| {
            ServerStatusError(format!("Not enough bytes to parse ServerStatus: {}, original size is {}", err, src.len()))
        })?;

        validate_server(Self::from(bytes))
    }
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            status: 0,
            load: 0,
            partial_score: 0.0,
        }
    }
}
