use serde::ser::Error as _;
use weathr::error::{ConfigError, GeolocationError, NetworkError, OnboardError, TerminalError};

#[test]
fn test_config_error_kind() {
    let error = ConfigError::InvalidLatitude(91.0);
    assert_eq!(error.kind(), "InvalidLatitude");

    let error = ConfigError::InvalidLongitude(181.0);
    assert_eq!(error.kind(), "InvalidLongitude");

    let error = ConfigError::NoConfigDir;
    assert_eq!(error.kind(), "NoConfigDir");
}

#[test]
fn test_config_error_display() {
    let error = ConfigError::InvalidLatitude(91.0);
    assert!(error.to_string().contains("91"));
    assert!(error.to_string().contains("latitude"));

    let error = ConfigError::InvalidLongitude(-181.0);
    assert!(error.to_string().contains("-181"));
    assert!(error.to_string().contains("longitude"));

    let error = ConfigError::NoConfigDir;
    assert!(error.to_string().contains("config directory"));
}

#[test]
fn test_network_error_is_retryable() {
    let timeout_error = NetworkError::Timeout {
        url: "https://example.com".to_string(),
        timeout_secs: 10,
    };
    assert!(timeout_error.is_retryable());

    let connection_refused = NetworkError::ConnectionRefused {
        url: "https://example.com".to_string(),
    };
    assert!(connection_refused.is_retryable());
}

#[test]
fn test_network_error_user_friendly_message() {
    let timeout_error = NetworkError::Timeout {
        url: "https://api.example.com".to_string(),
        timeout_secs: 5,
    };
    let msg = timeout_error.user_friendly_message();
    assert!(msg.contains("timed out"));
    assert!(msg.contains("5s"));
    assert!(msg.contains("api.example.com"));

    let connection_refused = NetworkError::ConnectionRefused {
        url: "https://localhost:9999".to_string(),
    };
    let msg = connection_refused.user_friendly_message();
    assert!(msg.contains("Cannot connect"));
    assert!(msg.contains("localhost:9999"));
}

#[test]
fn test_terminal_error_user_friendly_message() {
    let too_small = TerminalError::TooSmall {
        width: 40,
        height: 10,
        min_width: 70,
        min_height: 20,
    };
    let msg = too_small.user_friendly_message();
    assert!(msg.contains("40x10"));
    assert!(msg.contains("70x20"));
    assert!(msg.contains("too small"));

    let not_a_tty = TerminalError::NotATty;
    let msg = not_a_tty.user_friendly_message();
    assert!(msg.contains("terminal"));
    assert!(msg.contains("redirected") || msg.contains("piped"));
}

#[test]
fn test_geolocation_error_user_friendly_message() {
    let network_error = NetworkError::Timeout {
        url: "https://ipinfo.io/json".to_string(),
        timeout_secs: 10,
    };
    let geo_error = GeolocationError::Unreachable(network_error);
    let msg = geo_error.user_friendly_message();
    assert!(msg.contains("timed out"));
    assert!(msg.contains("10s"));
    assert!(msg.contains("configured/default"));
    assert!(!msg.contains("Cannot auto-detect location:"));

    let parse_error = GeolocationError::ParseError("Invalid coords".to_string());
    let msg = parse_error.user_friendly_message();
    assert!(msg.contains("invalid") || msg.contains("Invalid"));
    assert!(msg.contains("configured/default"));

    let retries_exhausted = GeolocationError::RetriesExhausted { attempts: 3 };
    let msg = retries_exhausted.user_friendly_message();
    assert!(msg.contains("3 attempts"));
    assert!(msg.contains("configured/default"));
}

#[test]
fn test_geolocation_error_all_network_variants() {
    let connection_refused = GeolocationError::Unreachable(NetworkError::ConnectionRefused {
        url: "https://ipinfo.io".to_string(),
    });
    let msg = connection_refused.user_friendly_message();
    assert!(msg.contains("unavailable"));
    assert!(!msg.contains("Network error:"));
    assert!(!msg.contains("Cannot auto-detect location:"));

    let timeout = GeolocationError::Unreachable(NetworkError::Timeout {
        url: "https://ipinfo.io".to_string(),
        timeout_secs: 5,
    });
    let msg = timeout.user_friendly_message();
    assert!(msg.contains("timed out"));
    assert!(msg.contains("5s"));
    assert!(!msg.contains("Network error:"));
    assert!(!msg.contains("Cannot auto-detect location:"));
}

#[test]
fn test_config_error_new_variants_kind() {
    let error = ConfigError::SerializeError(toml::ser::Error::custom("test"));
    assert_eq!(error.kind(), "SerializeError");

    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let error = ConfigError::WriteError {
        path: "/tmp/test".to_string(),
        source: io_err,
    };
    assert_eq!(error.kind(), "WriteError");
    assert!(error.to_string().contains("/tmp/test"));
}

#[test]
fn test_onboard_error_display() {
    let error = OnboardError::Cancelled;
    assert!(error.to_string().contains("cancelled"));

    let error = OnboardError::NoGeocodingResults("Atlantis".to_string());
    assert!(error.to_string().contains("Atlantis"));

    let error = OnboardError::PromptError("stdin closed".to_string());
    assert!(error.to_string().contains("stdin closed"));

    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let error = OnboardError::CreateDirError {
        path: "/root/weathr".to_string(),
        source: io_err,
    };
    assert!(error.to_string().contains("/root/weathr"));
}

#[test]
fn test_onboard_error_from_config_error() {
    let config_err = ConfigError::NoConfigDir;
    let onboard_err: OnboardError = config_err.into();
    assert!(onboard_err.to_string().contains("config directory"));
}
