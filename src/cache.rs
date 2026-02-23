use crate::weather::WeatherData;
use crate::{config::Provider, geolocation::GeoLocation};
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
    provider: Provider,
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

#[derive(Serialize, Deserialize)]
struct GeocodeCache {
    city_name: String,
    cached_at: u64,
    location_key: String,
    language: String,
}

pub async fn load_cached_geocode(latitude: f64, longitude: f64, language: &str) -> Option<String> {
    let cache_path = get_cache_dir()?.join("geocode.json");
    let contents = fs::read_to_string(&cache_path).await.ok()?;
    let cache: GeocodeCache = serde_json::from_str(&contents).ok()?;

    let location_key = make_location_key(latitude, longitude);
    if cache.location_key != location_key || cache.language != language {
        return None;
    }

    let now = current_timestamp();
    if now - cache.cached_at < LOCATION_CACHE_DURATION_SECS {
        Some(cache.city_name)
    } else {
        None
    }
}

pub fn save_geocode_cache(city_name: &str, latitude: f64, longitude: f64, language: &str) {
    let city_name = city_name.to_string();
    let language = language.to_string();
    tokio::spawn(async move {
        if let Some(cache_dir) = get_cache_dir() {
            let _ = fs::create_dir_all(&cache_dir).await;

            let cache = GeocodeCache {
                city_name,
                cached_at: current_timestamp(),
                location_key: make_location_key(latitude, longitude),
                language,
            };

            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = fs::write(cache_dir.join("geocode.json"), json).await;
            }
        }
    });
}

pub async fn load_cached_weather(
    latitude: f64,
    longitude: f64,
    provider: Provider,
) -> Option<WeatherData> {
    let cache_path = get_cache_dir()?.join("weather.json");
    let contents = fs::read_to_string(&cache_path).await.ok()?;
    let cache: WeatherCache = serde_json::from_str(&contents).ok()?;

    let location_key = make_location_key(latitude, longitude);
    if cache.location_key != location_key || cache.provider != provider {
        return None;
    }

    let now = current_timestamp();
    if now - cache.cached_at < WEATHER_CACHE_DURATION_SECS {
        Some(cache.data)
    } else {
        None
    }
}

pub fn save_weather_cache(
    weather: &WeatherData,
    latitude: f64,
    longitude: f64,
    provider: Provider,
) {
    let weather = weather.clone();
    tokio::spawn(async move {
        if let Some(cache_dir) = get_cache_dir() {
            let _ = fs::create_dir_all(&cache_dir).await;

            let cache = WeatherCache {
                data: weather,
                cached_at: current_timestamp(),
                location_key: make_location_key(latitude, longitude),
                provider,
            };

            if let Ok(json) = serde_json::to_string(&cache) {
                let _ = fs::write(cache_dir.join("weather.json"), json).await;
            }
        }
    });
}
