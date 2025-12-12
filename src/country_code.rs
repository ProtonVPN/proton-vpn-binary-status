// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

// The UniFFI bindings require errors to implement std::error::Error trait.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CountryCodeConversionError {
    #[error("Country code must be ascii letters")]
    InvalidFormat,
    #[error("Country code must be exactly 2 bytes in size")]
    InvalidLength,
}

// Represents a country code in the format of two uppercase ASCII letters,
// in the ISO 3166-1 alpha-2 format.
//
// The country code is stored as a 2-byte array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "cffi", repr(C))]
pub struct CountryCode([u8; 2]);

impl CountryCode {
    pub fn as_bytes(&self) -> &[u8; 2] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        // This should never panic because `Country` is always ASCII
        str::from_utf8(&self.0).expect("invalid country code")
    }
}

impl TryFrom<&[u8; 2]> for CountryCode {
    type Error = CountryCodeConversionError;

    fn try_from(
        value: &[u8; 2],
    ) -> std::result::Result<Self, CountryCodeConversionError> {
        // Should probably validate against `ISO 3166-1 alpha-2` format more
        // strictly in the future.
        if value.iter().any(|&c| !c.is_ascii()) {
            return Err(CountryCodeConversionError::InvalidFormat);
        }

        let uppercase_value =
            [value[0].to_ascii_uppercase(), value[1].to_ascii_uppercase()];
        Ok(Self(uppercase_value))
    }
}

impl TryFrom<&str> for CountryCode {
    type Error = CountryCodeConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = &<[u8; 2]>::try_from(value.as_bytes())
            .map_err(|_| CountryCodeConversionError::InvalidLength)?;

        CountryCode::try_from(bytes)
    }
}

impl AsRef<[u8; 2]> for CountryCode {
    fn as_ref(&self) -> &[u8; 2] {
        self.as_bytes()
    }
}

impl AsRef<str> for CountryCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> From<&'a CountryCode> for &'a str {
    fn from(value: &'a CountryCode) -> Self {
        // This should never panic because `Country` is always ASCII
        value.as_str()
    }
}

impl TryFrom<String> for CountryCode {
    type Error = CountryCodeConversionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

// This also implements `to_string()`
impl std::fmt::Display for CountryCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(self.as_str())
    }
}

// Needed by `uniffi`
impl From<CountryCode> for String {
    fn from(value: CountryCode) -> Self {
        String::from_utf8_lossy(&value.0).into_owned()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for CountryCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.into())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for CountryCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.try_into().map_err(|e: CountryCodeConversionError| {
            serde::de::Error::custom(e.to_string())
        })
    }
}

#[cfg(feature = "uniffi")]
uniffi::custom_type!(CountryCode, String);

#[cfg(test)]
mod tests {
    use super::*;

    type AnyResult = Result<(), Box<dyn std::error::Error>>;

    #[test_log::test]
    fn test_new() -> AnyResult {
        for nz_variant in [b"NZ", b"nz", b"Nz", b"nZ"] {
            assert_eq!(&CountryCode::try_from(nz_variant)?.0, b"NZ");
        }

        const INVALID_CHARACTERS: &[u8; 2] = b"\xc3\xa9"; // e with accent;
        assert_eq!(
            CountryCode::try_from(INVALID_CHARACTERS).unwrap_err(),
            CountryCodeConversionError::InvalidFormat
        );

        Ok(())
    }

    #[test_log::test]
    fn test_try_from() -> AnyResult {
        assert_eq!(CountryCode::try_from("US")?.as_bytes(), b"US");
        assert_eq!(CountryCode::try_from("US".to_string())?.as_bytes(), b"US");
        assert_eq!(
            CountryCode::try_from("USA").unwrap_err(),
            CountryCodeConversionError::InvalidLength
        );

        assert_eq!(CountryCode::try_from("US")?.as_str(), "US");
        assert_eq!(CountryCode::try_from("US")?.to_string(), "US");
        assert_eq!(String::from(CountryCode::try_from("US")?), "US");
        assert_eq!(CountryCode::try_from("us")?.as_str(), "US");
        assert_eq!(CountryCode::try_from("us")?.to_string(), "US");
        assert_eq!(String::from(CountryCode::try_from("Us")?), "US");

        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test_log::test]
    fn test_serialization() -> AnyResult {
        assert_eq!(
            serde_json::to_string(&CountryCode::try_from("US")?)?,
            "\"US\""
        );
        assert_eq!(
            serde_json::from_str::<CountryCode>("\"US\"")?,
            CountryCode::try_from(b"US")?
        );

        Ok(())
    }
}
