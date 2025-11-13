// -----------------------------------------------------------------------------
// Copyright (c) 2025 Proton AG
// -----------------------------------------------------------------------------
use std::str::Utf8Error;

// The UniFFI bindings require errors to implement std::error::Error trait.
#[derive(Debug, PartialEq, Eq)]
pub struct CountryConversionError(pub &'static str);

// Represents a country code in the format of two uppercase ASCII letters,
// in the ISO 3166-1 alpha-2 format.
//
// The country code is stored as a 2-byte array.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "cffi", repr(C))]
pub struct Country([u8; 2]);

impl Country {
    pub fn new(
        value: &[u8; 2],
    ) -> std::result::Result<Self, CountryConversionError> {
        if !(value[0].is_ascii_uppercase() && value[1].is_ascii_uppercase()) {
            return Err(CountryConversionError(
                "Country must be uppercase ascii letters",
            ));
        }

        Ok(Self(*value))
    }
}

impl std::fmt::Display for CountryConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for CountryConversionError {}

impl TryFrom<&str> for Country {
    type Error = CountryConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        if bytes.len() != 2 {
            return Err(CountryConversionError(
                "Country must be exactly 2 bytes in size",
            ));
        }

        Country::new(&[bytes[0], bytes[1]])
    }
}

impl<'a> TryFrom<&'a Country> for &'a str {
    type Error = Utf8Error;

    fn try_from(value: &'a Country) -> Result<Self, Self::Error> {
        str::from_utf8(&value.0)
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
        let string = self
            .try_into()
            .map_err(|e: Utf8Error| serde::ser::Error::custom(e.to_string()))?;
        serializer.serialize_str(string)
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
        assert_eq!(&Country::new(b"NZ")?.0, b"NZ");

        const E: &[u8; 2] = b"\xc3\xa9"; // e with accent;

        for error_case in [b"nz", b"Nz", b"nZ", E] {
            assert_eq!(
                &Country::new(error_case).unwrap_err(),
                &CountryConversionError(
                    "Country must be uppercase ascii letters"
                )
            );
        }

        Ok(())
    }

    #[test_log::test]
    fn test_try_from() -> AnyResult {
        assert_eq!(Country::try_from("US")?.0, *b"US");
        assert_eq!(Country::try_from("US".to_string())?.0, *b"US");
        assert_eq!(
            Country::try_from("USA").unwrap_err(),
            CountryConversionError("Country must be exactly 2 bytes in size")
        );

        assert_eq!(<&str>::try_from(&Country::try_from("US")?)?, "US");

        assert_eq!(<String>::from(Country::try_from("US")?), "US".to_string());

        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test_log::test]
    fn test_serialization() -> AnyResult {
        assert_eq!(serde_json::to_string(&Country::try_from("US")?)?, "\"US\"");
        assert_eq!(
            serde_json::from_str::<Country>("\"US\"")?,
            Country::new(b"US")?
        );

        Ok(())
    }
}
