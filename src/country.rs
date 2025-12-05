// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------

// The UniFFI bindings require errors to implement std::error::Error trait.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CountryConversionError {
    #[error("Country must be uppercase ascii letters")]
    InvalidFormat,
    #[error("Country must be exactly 2 bytes in size")]
    InvalidLength,
}

// Represents a country code in the format of two uppercase ASCII letters,
// in the ISO 3166-1 alpha-2 format.
//
// The country code is stored as a 2-byte array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "cffi", repr(C))]
pub struct Country([u8; 2]);

impl Country {
    pub fn as_bytes(&self) -> &[u8; 2] {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        // This should never panic because `Country` is always ASCII
        str::from_utf8(&self.0).expect("invalid country code")
    }
}

impl TryFrom<&[u8; 2]> for Country {
    type Error = CountryConversionError;

    fn try_from(
        value: &[u8; 2],
    ) -> std::result::Result<Self, CountryConversionError> {
        // Should probably validate against `ISO 3166-1 alpha-2` format more
        // strictly in the future.
        if !(value[0].is_ascii_uppercase() && value[1].is_ascii_uppercase()) {
            return Err(CountryConversionError::InvalidFormat);
        }

        Ok(Self(*value))
    }
}

impl TryFrom<&str> for Country {
    type Error = CountryConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = &<[u8; 2]>::try_from(value.as_bytes())
            .map_err(|_| CountryConversionError::InvalidLength)?;

        Country::try_from(bytes)
    }
}

impl AsRef<[u8; 2]> for Country {
    fn as_ref(&self) -> &[u8; 2] {
        self.as_bytes()
    }
}

impl AsRef<str> for Country {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> From<&'a Country> for &'a str {
    fn from(value: &'a Country) -> Self {
        // This should never panic because `Country` is always ASCII
        value.as_str()
    }
}

impl TryFrom<String> for Country {
    type Error = CountryConversionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl From<Country> for String {
    fn from(value: Country) -> Self {
        String::from_utf8_lossy(&value.0).into_owned()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Country {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.into())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Country {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.try_into().map_err(|e: CountryConversionError| {
            serde::de::Error::custom(e.to_string())
        })
    }
}

#[cfg(feature = "uniffi")]
uniffi::custom_type!(Country, String);

#[cfg(test)]
mod tests {
    use super::*;

    type AnyResult = Result<(), Box<dyn std::error::Error>>;

    #[test_log::test]
    fn test_new() -> AnyResult {
        assert_eq!(&Country::try_from(b"NZ")?.0, b"NZ");

        const E: &[u8; 2] = b"\xc3\xa9"; // e with accent;

        for error_case in [b"nz", b"Nz", b"nZ", E] {
            assert_eq!(
                Country::try_from(error_case).unwrap_err(),
                CountryConversionError::InvalidFormat
            );
        }

        Ok(())
    }

    #[test_log::test]
    fn test_try_from() -> AnyResult {
        assert_eq!(Country::try_from("US")?.as_bytes(), b"US");
        assert_eq!(Country::try_from("US".to_string())?.as_bytes(), b"US");
        assert_eq!(
            Country::try_from("USA").unwrap_err(),
            CountryConversionError::InvalidLength
        );

        assert_eq!(Country::try_from("US")?.as_str(), "US");

        assert_eq!(<String>::from(Country::try_from("US")?), "US".to_string());

        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test_log::test]
    fn test_serialization() -> AnyResult {
        assert_eq!(serde_json::to_string(&Country::try_from("US")?)?, "\"US\"");
        assert_eq!(
            serde_json::from_str::<Country>("\"US\"")?,
            Country::try_from(b"US")?
        );

        Ok(())
    }
}
