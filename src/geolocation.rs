use crate::cache;
use crate::error::{GeolocationError, NetworkError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const IPINFO_URL: &str = "https://ipinfo.io/json";
const NOMINATIM_URL: &str = "https://nominatim.openstreetmap.org/reverse";
const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 500;

#[derive(Deserialize, Debug)]
struct IpInfoResponse {
    loc: String,
    city: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub city: Option<String>,
}

pub async fn detect_location() -> Result<GeoLocation, GeolocationError> {
    if let Some(cached) = cache::load_cached_location().await {
        return Ok(cached);
    }

    detect_location_with_retry().await
}

async fn detect_location_with_retry() -> Result<GeoLocation, GeolocationError> {
    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        match fetch_location().await {
            Ok(location) => return Ok(location),
            Err(e) => {
                let should_retry = matches!(
                    e,
                    GeolocationError::Unreachable(ref net_err) if net_err.is_retryable()
                );

                if !should_retry || attempt == MAX_RETRIES {
                    return Err(e);
                }

                let delay_ms = INITIAL_RETRY_DELAY_MS * 2_u64.pow(attempt - 1);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                last_error = Some(e);
            }
        }
    }

    Err(
        last_error.unwrap_or_else(|| GeolocationError::RetriesExhausted {
            attempts: MAX_RETRIES,
        }),
    )
}

async fn fetch_location() -> Result<GeoLocation, GeolocationError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| GeolocationError::Unreachable(NetworkError::ClientCreation(e)))?;

    let response = client.get(IPINFO_URL).send().await.map_err(|e| {
        GeolocationError::Unreachable(NetworkError::from_reqwest(e, IPINFO_URL, 10))
    })?;

    let ip_info: IpInfoResponse = response.json().await.map_err(|e| {
        GeolocationError::Unreachable(NetworkError::from_reqwest(e, IPINFO_URL, 10))
    })?;

    let coords: Vec<&str> = ip_info.loc.split(',').collect();
    if coords.len() != 2 {
        return Err(GeolocationError::ParseError(
            "Invalid location format from ipinfo.io".to_string(),
        ));
    }

    let latitude = coords[0]
        .parse::<f64>()
        .map_err(|_| GeolocationError::ParseError("Invalid latitude format".to_string()))?;

    let longitude = coords[1]
        .parse::<f64>()
        .map_err(|_| GeolocationError::ParseError("Invalid longitude format".to_string()))?;

    let location = GeoLocation {
        latitude,
        longitude,
        city: ip_info.city,
    };

    cache::save_location_cache(&location);

    Ok(location)
}

#[derive(Deserialize, Debug)]
struct NominatimAddress {
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NominatimResponse {
    address: Option<NominatimAddress>,
}

/// Best-effort reverse geocode: returns a city/town/village name for the given
/// coordinates, or `None` if the lookup fails or the location doesn't map to a
/// meaningful settlement (e.g. open sea, administrative-only regions).
pub async fn reverse_geocode(latitude: f64, longitude: f64, language: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .build()
        .ok()?;

    let url = format!(
        "{}?lat={}&lon={}&format=json&zoom=10",
        NOMINATIM_URL, latitude, longitude
    );

    let mut req = client.get(&url).header(
        "User-Agent",
        format!("weathr/{}", env!("CARGO_PKG_VERSION")),
    );

    if language != "auto" {
        req = req.header("Accept-Language", language);
    }

    let resp = req.send().await.ok()?;

    let data: NominatimResponse = resp.json().await.ok()?;

    let addr = data.address?;
    addr.city.or(addr.town).or(addr.village)
}
