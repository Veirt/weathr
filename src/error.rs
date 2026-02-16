use std::io;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum WeatherError {
    #[error("{0}")]
    Network(#[from] NetworkError),

    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("{0}")]
    Terminal(#[from] TerminalError),

    #[error("{0}")]
    Geolocation(#[from] GeolocationError),
}

#[derive(ThisError, Debug)]
pub enum NetworkError {
    #[error("failed to create HTTP client: {0}")]
    ClientCreation(#[source] reqwest::Error),

    #[error("DNS resolution failed for {url}")]
    DnsFailure {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("connection timeout after {timeout_secs}s for {url}")]
    Timeout { url: String, timeout_secs: u64 },

    #[error("connection refused for {url}")]
    ConnectionRefused { url: String },

    #[error("HTTP request failed for {url}: {status}")]
    HttpError {
        url: String,
        status: u16,
        #[source]
        source: reqwest::Error,
    },

    #[error("failed to parse JSON response from {url}")]
    JsonParse {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("network error: {0}")]
    Other(#[from] reqwest::Error),
}

impl NetworkError {
    pub fn from_reqwest(error: reqwest::Error, url: &str, timeout_secs: u64) -> Self {
        if error.is_timeout() {
            NetworkError::Timeout {
                url: url.to_string(),
                timeout_secs,
            }
        } else if error.is_connect() {
            let error_msg = error.to_string();
            if error_msg.contains("dns") || error_msg.contains("DNS") {
                return NetworkError::DnsFailure {
                    url: url.to_string(),
                    source: error,
                };
            }
            if error_msg.contains("Connection refused") || error_msg.contains("refused") {
                return NetworkError::ConnectionRefused {
                    url: url.to_string(),
                };
            }
            NetworkError::Other(error)
        } else if error.is_status() {
            if let Some(status) = error.status() {
                return NetworkError::HttpError {
                    url: url.to_string(),
                    status: status.as_u16(),
                    source: error,
                };
            }
            NetworkError::Other(error)
        } else if error.is_decode() {
            NetworkError::JsonParse {
                url: url.to_string(),
                source: error,
            }
        } else {
            NetworkError::Other(error)
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            NetworkError::Timeout { .. }
                | NetworkError::ConnectionRefused { .. }
                | NetworkError::DnsFailure { .. }
        )
    }

    pub fn user_friendly_message(&self) -> String {
        match self {
            NetworkError::DnsFailure { url, .. } => {
                format!("Cannot reach {url}. Check your internet connection or DNS settings.")
            }
            NetworkError::Timeout { url, timeout_secs } => {
                format!(
                    "Request to {url} timed out after {timeout_secs}s. Check your internet connection."
                )
            }
            NetworkError::ConnectionRefused { url } => {
                format!("Cannot connect to {url}. The service may be down.")
            }
            NetworkError::HttpError { url, status, .. } => {
                format!("Server error from {url}: HTTP {status}")
            }
            NetworkError::JsonParse { url, .. } => {
                format!("Received invalid data from {url}")
            }
            NetworkError::ClientCreation(_) => "Failed to initialize HTTP client".to_string(),
            NetworkError::Other(e) => format!("Network error: {e}"),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum ConfigError {
    #[error("failed to read config file at {path}")]
    ReadError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("failed to write config file at {path}")]
    WriteError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("invalid TOML syntax in config file")]
    ParseError(#[from] toml::de::Error),

    #[error("failed to serialize config")]
    SerializeError(#[from] toml::ser::Error),

    #[error("could not determine config directory (check $XDG_CONFIG_HOME or $HOME)")]
    NoConfigDir,

    #[error("invalid latitude {0} (must be between -90 and 90)")]
    InvalidLatitude(f64),

    #[error("invalid longitude {0} (must be between -180 and 180)")]
    InvalidLongitude(f64),

    #[error("invalid value for ${name} (expected a float, got {value:?})")]
    InvalidEnvVar { name: &'static str, value: String },
}

impl ConfigError {
    #[allow(dead_code)]
    pub fn kind(&self) -> &str {
        match self {
            ConfigError::ReadError { .. } => "ReadError",
            ConfigError::WriteError { .. } => "WriteError",
            ConfigError::ParseError(_) => "ParseError",
            ConfigError::SerializeError(_) => "SerializeError",
            ConfigError::NoConfigDir => "NoConfigDir",
            ConfigError::InvalidLatitude(_) => "InvalidLatitude",
            ConfigError::InvalidLongitude(_) => "InvalidLongitude",
            ConfigError::InvalidEnvVar { .. } => "InvalidEnvVar",
        }
    }
}

#[derive(ThisError, Debug)]
pub enum TerminalError {
    #[error("terminal is too small (minimum: {min_width}x{min_height}, current: {width}x{height})")]
    TooSmall {
        width: u16,
        height: u16,
        min_width: u16,
        min_height: u16,
    },

    #[error("not running in a terminal (output is redirected or piped)")]
    NotATty,

    #[error("failed to enable raw mode")]
    RawModeError(#[source] io::Error),

    #[error("failed to get terminal size")]
    SizeError(#[source] io::Error),

    #[error("failed to initialize terminal")]
    InitError(#[source] io::Error),

    #[error("terminal I/O error")]
    IoError(#[from] io::Error),
}

impl TerminalError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            TerminalError::TooSmall {
                width,
                height,
                min_width,
                min_height,
            } => {
                format!(
                    "Terminal window is too small ({width}x{height}).\n\
                     Please resize to at least {min_width}x{min_height} characters."
                )
            }
            TerminalError::NotATty => "This application must be run in a terminal.\n\
                 It cannot work when output is redirected or piped."
                .to_string(),
            TerminalError::RawModeError(_) => "Failed to initialize terminal raw mode.\n\
                 You may need to run this in a proper terminal emulator."
                .to_string(),
            TerminalError::SizeError(_) => "Cannot detect terminal size.\n\
                 Make sure you're running in a standard terminal."
                .to_string(),
            _ => self.to_string(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum GeolocationError {
    #[error("cannot reach geolocation service")]
    Unreachable(#[source] NetworkError),

    #[error("failed to parse location data: {0}")]
    ParseError(String),

    #[error("failed after {attempts} retry attempts")]
    RetriesExhausted { attempts: u32 },
}

impl GeolocationError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            GeolocationError::Unreachable(net_err) => match net_err {
                NetworkError::Timeout { timeout_secs, .. } => {
                    format!(
                        "Location detection timed out after {timeout_secs}s. Check your internet connection.\n\
                         Using configured/default location."
                    )
                }
                NetworkError::DnsFailure { .. } => {
                    "Cannot reach location service. Check your DNS settings.\n\
                     Using configured/default location."
                        .to_string()
                }
                NetworkError::ConnectionRefused { .. } => {
                    "Location service is unavailable. Try again later.\n\
                     Using configured/default location."
                        .to_string()
                }
                NetworkError::HttpError { status, .. } => {
                    format!(
                        "Location service returned error (HTTP {status}).\n\
                         Using configured/default location."
                    )
                }
                NetworkError::JsonParse { .. } => "Received invalid data from location service.\n\
                     Using configured/default location."
                    .to_string(),
                NetworkError::ClientCreation(_) => "Failed to initialize network client.\n\
                     Using configured/default location."
                    .to_string(),
                NetworkError::Other(_) => {
                    "Cannot auto-detect location. Check your internet connection.\n\
                     Using configured/default location."
                        .to_string()
                }
            },
            GeolocationError::ParseError(_) => "Received invalid location data.\n\
                 Using configured/default location."
                .to_string(),
            GeolocationError::RetriesExhausted { attempts } => {
                format!(
                    "Failed to detect location after {attempts} attempts.\n\
                     Using configured/default location."
                )
            }
        }
    }
}

#[derive(ThisError, Debug)]
pub enum OnboardError {
    #[error("failed to create config directory: {path}")]
    CreateDirError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("{0}")]
    Config(#[from] ConfigError),

    #[error("geocoding search failed")]
    GeocodingError(#[source] NetworkError),

    #[error("no results found for \"{0}\"")]
    NoGeocodingResults(String),

    #[error("user cancelled the setup")]
    Cancelled,

    #[error("interactive prompt error: {0}")]
    PromptError(String),
}
