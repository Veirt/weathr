use std::sync::Arc;
use std::time::Duration;
use weathr::weather::{OpenMeteoProvider, WeatherClient, WeatherLocation, WeatherUnits};

#[tokio::test]
async fn test_weather_client_integration_cache_behavior() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let weather1 = client
        .get_current_weather(&location, &units)
        .await
        .expect("First fetch should succeed");

    let weather2 = client
        .get_current_weather(&location, &units)
        .await
        .expect("Second fetch should succeed");

    assert_eq!(
        weather1.timestamp, weather2.timestamp,
        "Second fetch should return cached data"
    );
}

#[tokio::test]
async fn test_weather_client_integration_cache_invalidation() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let _weather1 = client
        .get_current_weather(&location, &units)
        .await
        .expect("First fetch should succeed");

    client.invalidate_cache().await;

    let weather2 = client
        .get_current_weather(&location, &units)
        .await
        .expect("Fetch after invalidation should succeed");

    assert!(
        weather2.temperature >= -90.0 && weather2.temperature <= 60.0,
        "Weather data should still be valid after cache invalidation"
    );
}

#[tokio::test]
async fn test_weather_client_integration_provider_name() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    assert_eq!(
        client.get_provider_name(),
        "Open-Meteo",
        "Provider name should be correct"
    );
}

#[tokio::test]
async fn test_weather_client_integration_realistic_weather_ranges() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 0.0,
        longitude: 0.0,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let weather = client
        .get_current_weather(&location, &units)
        .await
        .expect("Should fetch weather");

    assert!(
        weather.temperature >= -90.0 && weather.temperature <= 60.0,
        "Temperature should be within realistic range"
    );
    assert!(
        weather.humidity >= 0.0 && weather.humidity <= 100.0,
        "Humidity should be 0-100%"
    );
    assert!(
        weather.wind_speed >= 0.0 && weather.wind_speed <= 500.0,
        "Wind speed should be realistic"
    );
    assert!(
        weather.wind_direction >= 0.0 && weather.wind_direction <= 360.0,
        "Wind direction should be 0-360 degrees"
    );
    assert!(
        weather.cloud_cover >= 0.0 && weather.cloud_cover <= 100.0,
        "Cloud cover should be 0-100%"
    );
    assert!(
        weather.pressure >= 800.0 && weather.pressure <= 1100.0,
        "Pressure should be realistic (hPa)"
    );
    assert!(
        weather.precipitation >= 0.0,
        "Precipitation should be non-negative"
    );
}
