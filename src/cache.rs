use crate::geolocation::GeoLocation;
use crate::weather::WeatherData;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

const LOCATION_CACHE_DURATION_SECS: u64 = 86400;
const WEATHER_CACHE_DURATION_SECS: u64 = 300;

#[derive(Serialize, Deserialize)]
struct LocationCache {
    location: GeoLocation,
    cached_at: u64,
}

#[derive(Serialize, Deserialize)]
struct WeatherCache {
    data: WeatherData,
    cached_at: u64,
    location_key: String,
}

fn get_cache_dir() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join("weathr"))
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn make_location_key(latitude: f64, longitude: f64) -> String {
    format!("{:.2},{:.2}", latitude, longitude)
}

pub async fn load_cached_location() -> Option<GeoLocation> {
    let cache_path = get_cache_dir()?.join("location.json");
    let contents = fs::read_to_string(&cache_path).await.ok()?;
    let cache: LocationCache = serde_json::from_str(&contents).ok()?;

    let now = current_timestamp();
    if now - cache.cached_at < LOCATION_CACHE_DURATION_SECS {
        Some(cache.location)
    } else {
        None
    }
}

pub fn save_location_cache(location: &GeoLocation) {
    let location = location.clone();
    tokio::spawn(async move {
        if let Some(cache_dir) = get_cache_dir() {
            let _ = fs::create_dir_all(&cache_dir).await;

            let cache = LocationCache {
                location,
                cached_at: current_timestamp(),
            };

            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = fs::write(cache_dir.join("location.json"), json).await;
            }
        }
    });
}

pub async fn load_cached_weather(latitude: f64, longitude: f64) -> Option<WeatherData> {
    let cache_path = get_cache_dir()?.join("weather.json");
    let contents = fs::read_to_string(&cache_path).await.ok()?;
    let cache: WeatherCache = serde_json::from_str(&contents).ok()?;

    let location_key = make_location_key(latitude, longitude);
    if cache.location_key != location_key {
        return None;
    }

    let now = current_timestamp();
    if now - cache.cached_at < WEATHER_CACHE_DURATION_SECS {
        Some(cache.data)
    } else {
        None
    }
}

pub fn save_weather_cache(weather: &WeatherData, latitude: f64, longitude: f64) {
    let weather = weather.clone();
    tokio::spawn(async move {
        if let Some(cache_dir) = get_cache_dir() {
            let _ = fs::create_dir_all(&cache_dir).await;

            let cache = WeatherCache {
                data: weather,
                cached_at: current_timestamp(),
                location_key: make_location_key(latitude, longitude),
            };

            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = fs::write(cache_dir.join("weather.json"), json).await;
            }
        }
    });
}
