// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------
use super::server_status::ServerStatus;
use crate::{error::*, status::server_status::ServerStatusResult};

pub const VERSION_HEADER: [u8; 4] = [1_u8, 0_u8, 0_u8, 0_u8];
const SERVER_SIZE: usize = 6; // 1 byte status, 1 byte load, 4 bytes partial score

fn handle_errors(
    index: usize,
    result: ServerStatusResult<ServerStatus>,
    default: &ServerStatus,
    log_errors: &mut impl FnMut(usize, usize, &str),
) -> ServerStatus {
    match result {
        Ok(server_status) => server_status,
        Err(error) => {
            log_errors(index, index * SERVER_SIZE, &error.0);
            default.clone()
        }
    }
}
/// Interpretes a byte stream as a status file containing multiple servers.
#[derive(Debug)]
pub struct Parser<'a>(&'a [u8]);

impl Parser<'_> {
    // Returns the server at the given index.
    //
    // - If the index is out of bounds, it returns a copy of the default status
    //   provided.
    // - If there is an error parsing the server status, it logs the error
    //   (only once) and returns the default status.
    //
    pub fn get(
        &self,
        i: usize,
        default: &ServerStatus,
        log_errors: &mut impl FnMut(usize, usize, &str),
    ) -> ServerStatus {
        if i >= self.len() {
            return default.clone();
        }

        let lower = i * SERVER_SIZE;
        let upper = lower + SERVER_SIZE;

        handle_errors(
            i,
            ServerStatus::try_from(&self.0[lower..upper]),
            default,
            log_errors,
        )
    }

    /// Returns the number of servers in the status file.
    ///
    pub fn len(&self) -> usize {
        self.0.len() / SERVER_SIZE
    }

    /// Returns a bool indicating whether the status file contains
    /// any servers.
    ///
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the servers in the status file.
    ///
    pub fn iter(&self) -> impl Iterator<Item = ServerStatus> + '_ {
        // Waiting on Iterator::array_chunks to stabilize.
        // https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.array_chunks
        // before we can remove this expect call
        self.0.chunks_exact(SERVER_SIZE).map(|chunk| {
            let array: &[u8; 6] =
                chunk.try_into().expect("Chunk size is not correct");
            ServerStatus::from(array)
        })
    }
}

impl<'a> std::convert::TryFrom<&'a [u8]> for Parser<'a> {
    type Error = Error;

    fn try_from(value: &'a [u8]) -> Result<Self> {
        if value.len() < 4 {
            return Err(Error::ParserError(
                "Failed to read first 4 bytes in magic number".to_string(),
            ));
        }

        let version: &[u8; 4] = value[0..4].try_into().map_err(|_| {
            Error::ParserError(
                "Failed to convert first 4 bytes in magic number".to_string(),
            )
        })?;

        if version != &VERSION_HEADER {
            return Err(Error::ParserError("Invalid magic number".to_string()));
        }

        if ((value.len() - 4) % SERVER_SIZE) != 0 {
            return Err(Error::ParserError(
                "Status file is corrupt".to_string(),
            ));
        }

        Ok(Self(&value[4..]))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Parser<'_> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq as _;

        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for i in self.iter() {
            seq.serialize_element(&i)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn make_server(server_status: ServerStatus) -> [u8; 6] {
        [
            server_status.status,
            server_status.load,
            server_status.partial_score.to_le_bytes()[0],
            server_status.partial_score.to_le_bytes()[1],
            server_status.partial_score.to_le_bytes()[2],
            server_status.partial_score.to_le_bytes()[3],
        ]
    }

    fn make_status_file(servers: &[[u8; 6]]) -> Vec<u8> {
        let mut result = vec![1_u8, 0_u8, 0_u8, 0_u8];
        for server in servers {
            result.extend_from_slice(server);
        }
        result
    }

    #[test_log::test]
    fn test_simple_valid_status_file() {
        let simple_status = make_status_file(&[
            make_server(ServerStatus {
                status: 1,
                load: 57,
                partial_score: 0.97,
            }),
            make_server(ServerStatus {
                status: 1,
                load: 75,
                partial_score: 0.99,
            }),
            make_server(ServerStatus {
                status: 1,
                load: 23,
                partial_score: 0.43,
            }),
        ]);

        let status = Parser::try_from(&simple_status[..])
            .expect("Failed to parse status file");

        let mut errors = move |_index: usize, _offset: usize, error: &str| {
            panic!("Should not have any errors {}", error);
        };

        let default = ServerStatus::default();

        assert_eq!(status.len(), 3);
        assert_eq!(status.get(0, &default, &mut errors).status, 1);
        assert_eq!(status.get(0, &default, &mut errors).load, 57);
        assert_eq!(status.get(0, &default, &mut errors).partial_score, 0.97);
        assert_eq!(status.get(1, &default, &mut errors).status, 1);
        assert_eq!(status.get(1, &default, &mut errors).load, 75);
        assert_eq!(status.get(1, &default, &mut errors).partial_score, 0.99);
        assert_eq!(status.get(2, &default, &mut errors).status, 1);
        assert_eq!(status.get(2, &default, &mut errors).load, 23);
        assert_eq!(status.get(2, &default, &mut errors).partial_score, 0.43);
    }

    #[test_log::test]
    fn test_simple_empty() {
        let simple_status = [8_u8];

        let error = Parser::try_from(&simple_status[..]).unwrap_err();
        match error {
            Error::ParserError(error) => assert_eq!(
                error,
                "Failed to read first 4 bytes in magic number"
            ),
            _ => panic!("Expected ParserError"),
        };
    }

    #[test_log::test]
    fn test_simple_invalid_status_file() {
        let simple_status = [8_u8, 7_u8, 6_u8, 5_u8];

        let error = Parser::try_from(&simple_status[..]).unwrap_err();
        match error {
            Error::ParserError(error) => {
                assert_eq!(error, "Invalid magic number")
            }
            _ => panic!("Expected Invalid magic number error"),
        };
    }

    #[test_log::test]
    fn test_simple_corrupt_file() {
        let simple_status = [1_u8, 0_u8, 0_u8, 0_u8, 0_u8];

        let error = Parser::try_from(&simple_status[..]).unwrap_err();
        match error {
            Error::ParserError(error) => {
                assert_eq!(error, "Status file is corrupt")
            }
            _ => panic!("Expected Status file is corrupt error"),
        };
    }

    #[test_log::test]
    fn test_out_of_range_index() {
        let server_zero = ServerStatus {
            status: 1,
            load: 57,
            partial_score: 0.97,
        };
        let simple_status =
            make_status_file(&[make_server(server_zero.clone())]);

        let status = Parser::try_from(&simple_status[..])
            .expect("Failed to parse status file");

        let mut error_reported = false;
        let mut handle_errors =
            |_index: usize, _offset: usize, _error: &str| {
                error_reported = true;
            };

        let default = ServerStatus::default();
        assert_eq!(status.get(0, &default, &mut handle_errors), server_zero);
        assert_eq!(
            status.get(1, &default, &mut handle_errors),
            ServerStatus::default()
        );
        // Its not an error to request an out-of-range index
        assert!(!error_reported);
    }

    #[test_log::test]
    fn test_invalid_load() {
        let simple_status = make_status_file(&[
            make_server(ServerStatus {
                status: 1,
                load: 150, // Invalid load
                partial_score: 1.0,
            }),
            make_server(ServerStatus {
                status: 1,
                load: 200, // Invalid load
                partial_score: 1.0,
            }),
        ]);

        let status = Parser::try_from(&simple_status[..])
            .expect("Failed to parse status file");

        let mut error_reported = 0;
        let mut handle_errors =
            |_index: usize, _offset: usize, _error: &str| {
                error_reported += 1;
            };

        let default = ServerStatus::default();
        assert_eq!(status.get(0, &default, &mut handle_errors), default);
        assert_eq!(status.get(1, &default, &mut handle_errors), default);
        // This error should be reported twice, once for each invalid server
        assert_eq!(error_reported, 2);
    }
    #[test_log::test]
    fn test_invalid_partial_score() {
        let simple_status = make_status_file(&[
            make_server(ServerStatus {
                status: 1,
                load: 0,
                partial_score: 100.0, // Invalid partial score
            }),
            make_server(ServerStatus {
                status: 1,
                load: 0,
                partial_score: -100.0, // Invalid partial score
            }),
        ]);

        let status = Parser::try_from(&simple_status[..])
            .expect("Failed to parse status file");

        let mut error_reported = 0;
        let mut handle_errors =
            |_index: usize, _offset: usize, _error: &str| {
                error_reported += 1;
            };

        let default = ServerStatus::default();
        assert_eq!(status.get(0, &default, &mut handle_errors), default);
        assert_eq!(status.get(1, &default, &mut handle_errors), default);
        // This error should be reported twice, once for each invalid server
        assert_eq!(error_reported, 2);
    }
}
