use crate::error::{GeocodingError, NetworkError};
use serde::Deserialize;
use std::time::Duration;

const GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";
const MAX_RETRIES: u32 = 3;
const INITIAL_RETRY_DELAY_MS: u64 = 500;

#[derive(Deserialize, Debug)]
struct GeocodingResponse {
    results: Option<Vec<GeocodingResult>>,
}

#[derive(Deserialize, Debug)]
struct GeocodingResult {
    latitude: f64,
    longitude: f64,
    name: String,
    country: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeocodedLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub name: String,
    pub country: Option<String>,
}

pub async fn geocode_city(city: &str) -> Result<GeocodedLocation, GeocodingError> {
    geocode_city_with_retry(city).await
}

async fn geocode_city_with_retry(city: &str) -> Result<GeocodedLocation, GeocodingError> {
    let mut last_error = None;
    let mut delay = Duration::from_millis(INITIAL_RETRY_DELAY_MS);

    for attempt in 1..=MAX_RETRIES {
        match fetch_geocoding(city).await {
            Ok(location) => return Ok(location),
            Err(GeocodingError::CityNotFound(_)) => {
                return Err(GeocodingError::CityNotFound(city.to_string()));
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < MAX_RETRIES {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }
    }

    Err(last_error.unwrap_or(GeocodingError::RetriesExhausted {
        attempts: MAX_RETRIES,
    }))
}

fn url_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                String::from(b as char)
            }
            b' ' => String::from("+"),
            _ => format!("%{:02X}", b),
        })
        .collect()
}

async fn fetch_geocoding(city: &str) -> Result<GeocodedLocation, GeocodingError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| GeocodingError::Unreachable(NetworkError::ClientCreation(e)))?;

    let url = format!(
        "{}?name={}&count=1&language=en",
        GEOCODING_URL,
        url_encode(city)
    );

    let response = client.get(&url).send().await.map_err(|e| {
        GeocodingError::Unreachable(NetworkError::from_reqwest(e, &url, 10))
    })?;

    let geo_response: GeocodingResponse = response.json().await.map_err(|e| {
        GeocodingError::Unreachable(NetworkError::from_reqwest(e, &url, 10))
    })?;

    let result = geo_response
        .results
        .and_then(|r: Vec<GeocodingResult>| r.into_iter().next())
        .ok_or_else(|| GeocodingError::CityNotFound(city.to_string()))?;

    Ok(GeocodedLocation {
        latitude: result.latitude,
        longitude: result.longitude,
        name: result.name,
        country: result.country,
    })
}
