use std::fmt;

#[derive(Debug)]
pub enum TwitterError {
    ConfigError(ConfigError),
    ApiError(ApiError),
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    TomlDeserializeError(toml::de::Error),
    TomlSerializeError(toml::ser::Error),
    ReqwestError(reqwest::Error),
}

#[derive(Debug)]
pub enum ConfigError {
    HomeDirNotFound,
    ReadFailed {
        path: String,
        source: std::io::Error,
    },
    WriteFailed {
        path: String,
        source: std::io::Error,
    },
    InvalidFormat {
        source: toml::de::Error,
    },
    MissingField {
        field: String,
    },
    EditorFailed {
        editor: String,
        source: std::io::Error,
    },
}

#[derive(Debug)]
pub enum ApiError {
    InvalidCredentials,
    RateLimited,
    TweetTooLong,
    NetworkError(reqwest::Error),
    InvalidResponse { status: u16, body: String },
}

impl fmt::Display for TwitterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TwitterError::ConfigError(e) => write!(f, "Configuration error: {e}"),
            TwitterError::ApiError(e) => write!(f, "API error: {e}"),
            TwitterError::IoError(e) => write!(f, "I/O error: {e}"),
            TwitterError::SerializationError(e) => write!(f, "Serialization error: {e}"),
            TwitterError::TomlDeserializeError(e) => write!(f, "TOML parsing error: {e}"),
            TwitterError::TomlSerializeError(e) => write!(f, "TOML serialization error: {e}"),
            TwitterError::ReqwestError(e) => write!(f, "HTTP error: {e}"),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::HomeDirNotFound => write!(f, "Home directory not found"),
            ConfigError::ReadFailed { path, source } => {
                write!(f, "Failed to read config file '{path}': {source}")
            }
            ConfigError::WriteFailed { path, source } => {
                write!(f, "Failed to write config file '{path}': {source}")
            }
            ConfigError::InvalidFormat { source } => {
                write!(f, "Invalid config file format: {source}")
            }
            ConfigError::MissingField { field } => {
                write!(f, "Missing required field in config: {field}")
            }
            ConfigError::EditorFailed { editor, source } => {
                write!(f, "Failed to open editor '{editor}': {source}")
            }
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::InvalidCredentials => write!(f, "Invalid API credentials"),
            ApiError::RateLimited => write!(f, "Rate limited by Twitter API"),
            ApiError::TweetTooLong => write!(f, "Tweet exceeds character limit"),
            ApiError::NetworkError(e) => write!(f, "Network error: {e}"),
            ApiError::InvalidResponse { status, body } => {
                write!(f, "Invalid API response (status {status}): {body}")
            }
        }
    }
}

impl std::error::Error for TwitterError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for ApiError {}

// Conversion implementations
impl From<std::io::Error> for TwitterError {
    fn from(err: std::io::Error) -> Self {
        TwitterError::IoError(err)
    }
}

impl From<serde_json::Error> for TwitterError {
    fn from(err: serde_json::Error) -> Self {
        TwitterError::SerializationError(err)
    }
}

impl From<toml::de::Error> for TwitterError {
    fn from(err: toml::de::Error) -> Self {
        TwitterError::TomlDeserializeError(err)
    }
}

impl From<toml::ser::Error> for TwitterError {
    fn from(err: toml::ser::Error) -> Self {
        TwitterError::TomlSerializeError(err)
    }
}

impl From<reqwest::Error> for TwitterError {
    fn from(err: reqwest::Error) -> Self {
        TwitterError::ReqwestError(err)
    }
}

impl From<ConfigError> for TwitterError {
    fn from(err: ConfigError) -> Self {
        TwitterError::ConfigError(err)
    }
}

impl From<ApiError> for TwitterError {
    fn from(err: ApiError) -> Self {
        TwitterError::ApiError(err)
    }
}

pub type Result<T> = std::result::Result<T, TwitterError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::HomeDirNotFound;
        assert_eq!(format!("{error}"), "Home directory not found");

        let error = ConfigError::MissingField {
            field: "test_field".to_string(),
        };
        assert_eq!(
            format!("{error}"),
            "Missing required field in config: test_field"
        );
    }

    #[test]
    fn test_api_error_display() {
        let error = ApiError::InvalidCredentials;
        assert_eq!(format!("{error}"), "Invalid API credentials");

        let error = ApiError::RateLimited;
        assert_eq!(format!("{error}"), "Rate limited by Twitter API");
    }

    #[test]
    fn test_twitter_error_display() {
        let error = TwitterError::ConfigError(ConfigError::HomeDirNotFound);
        assert_eq!(
            format!("{error}"),
            "Configuration error: Home directory not found"
        );

        let error = TwitterError::ApiError(ApiError::InvalidCredentials);
        assert_eq!(format!("{error}"), "API error: Invalid API credentials");
    }

    #[test]
    fn test_error_conversions() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let twitter_error: TwitterError = io_error.into();
        assert!(matches!(twitter_error, TwitterError::IoError(_)));

        let config_error = ConfigError::HomeDirNotFound;
        let twitter_error: TwitterError = config_error.into();
        assert!(matches!(twitter_error, TwitterError::ConfigError(_)));
    }
}
