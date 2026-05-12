use crate::weather::WeatherData;
use crate::{config::Provider, geolocation::GeoLocation};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::mpsc;

const LOCATION_CACHE_DURATION_SECS: u64 = 86400;
const WEATHER_CACHE_DURATION_SECS: u64 = 300;
const CACHE_WRITER_BUFFER_SIZE: usize = 32;

#[derive(Debug)]
enum CacheWriteTask {
    Location(GeoLocation),
    Weather(WeatherData, f64, f64, Provider),
    Geocode(String, f64, f64, String),
}

static CACHE_WRITER: OnceLock<mpsc::Sender<CacheWriteTask>> = OnceLock::new();

fn get_cache_writer() -> &'static mpsc::Sender<CacheWriteTask> {
    CACHE_WRITER.get_or_init(|| {
        let (tx, mut rx) = mpsc::channel::<CacheWriteTask>(CACHE_WRITER_BUFFER_SIZE);

        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                match task {
                    CacheWriteTask::Location(location) => {
                        let _ = write_location_cache(&location).await;
                    }
                    CacheWriteTask::Weather(data, lat, lon, provider) => {
                        let _ = write_weather_cache(&data, lat, lon, provider).await;
                    }
                    CacheWriteTask::Geocode(city, lat, lon, lang) => {
                        let _ = write_geocode_cache(&city, lat, lon, &lang).await;
                    }
                }
            }
        });

        tx
    })
}

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

async fn write_location_cache(location: &GeoLocation) -> Result<(), std::io::Error> {
    if let Some(cache_dir) = get_cache_dir() {
        fs::create_dir_all(&cache_dir).await?;

        let cache = LocationCache {
            location: location.clone(),
            cached_at: current_timestamp(),
        };

        let json = serde_json::to_string(&cache)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(cache_dir.join("location.json"), json).await?;
    }
    Ok(())
}

pub fn save_location_cache(location: &GeoLocation) {
    let location = location.clone();
    let writer = get_cache_writer();
    let _ = writer.try_send(CacheWriteTask::Location(location));
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

async fn write_geocode_cache(
    city_name: &str,
    latitude: f64,
    longitude: f64,
    language: &str,
) -> Result<(), std::io::Error> {
    if let Some(cache_dir) = get_cache_dir() {
        fs::create_dir_all(&cache_dir).await?;

        let cache = GeocodeCache {
            city_name: city_name.to_string(),
            cached_at: current_timestamp(),
            location_key: make_location_key(latitude, longitude),
            language: language.to_string(),
        };

        let json = serde_json::to_string(&cache)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(cache_dir.join("geocode.json"), json).await?;
    }
    Ok(())
}

pub fn save_geocode_cache(city_name: &str, latitude: f64, longitude: f64, language: &str) {
    let city_name = city_name.to_string();
    let language = language.to_string();
    let writer = get_cache_writer();
    let _ = writer.try_send(CacheWriteTask::Geocode(
        city_name, latitude, longitude, language,
    ));
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

async fn write_weather_cache(
    weather: &WeatherData,
    latitude: f64,
    longitude: f64,
    provider: Provider,
) -> Result<(), std::io::Error> {
    if let Some(cache_dir) = get_cache_dir() {
        fs::create_dir_all(&cache_dir).await?;

        let cache = WeatherCache {
            data: weather.clone(),
            cached_at: current_timestamp(),
            location_key: make_location_key(latitude, longitude),
            provider,
        };

        let json = serde_json::to_string(&cache)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(cache_dir.join("weather.json"), json).await?;
    }
    Ok(())
}

pub fn save_weather_cache(
    weather: &WeatherData,
    latitude: f64,
    longitude: f64,
    provider: Provider,
) {
    let weather = weather.clone();
    let writer = get_cache_writer();
    let _ = writer.try_send(CacheWriteTask::Weather(
        weather, latitude, longitude, provider,
    ));
}
