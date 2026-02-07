use std::sync::Arc;
use std::time::Duration;
use weathr::weather::{
    OpenMeteoProvider, PrecipitationUnit, TemperatureUnit, WeatherClient, WeatherLocation,
    WeatherUnits, WindSpeedUnit,
};

#[tokio::test]
async fn test_open_meteo_provider_integration_basic_fetch() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let result = client.get_current_weather(&location, &units).await;
    assert!(result.is_ok(), "Should successfully fetch weather data");

    let weather = result.unwrap();
    assert!(weather.temperature > -100.0 && weather.temperature < 60.0);
    assert!(weather.humidity >= 0.0 && weather.humidity <= 100.0);
    assert!(weather.cloud_cover >= 0.0 && weather.cloud_cover <= 100.0);
}

#[tokio::test]
async fn test_open_meteo_provider_integration_multiple_locations() {
    let test_cases = vec![
        (52.52, 13.41, "Berlin"),
        (40.7128, -74.0060, "New York"),
        (-33.8688, 151.2093, "Sydney"),
        (35.6762, 139.6503, "Tokyo"),
        (51.5074, -0.1278, "London"),
    ];

    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));
    let units = WeatherUnits::default();

    for (lat, lon, city_name) in test_cases {
        let location = WeatherLocation {
            latitude: lat,
            longitude: lon,
            elevation: None,
        };

        let result = client.get_current_weather(&location, &units).await;
        assert!(
            result.is_ok(),
            "Should successfully fetch weather for {}",
            city_name
        );

        let weather = result.unwrap();
        assert!(
            weather.temperature > -100.0 && weather.temperature < 60.0,
            "Temperature should be realistic for {}",
            city_name
        );
        assert!(
            weather.humidity >= 0.0 && weather.humidity <= 100.0,
            "Humidity should be valid for {}",
            city_name
        );
    }
}

#[tokio::test]
async fn test_open_meteo_provider_integration_celsius_units() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let units = WeatherUnits {
        temperature: TemperatureUnit::Celsius,
        wind_speed: WindSpeedUnit::Kmh,
        precipitation: PrecipitationUnit::Mm,
    };

    let weather = client
        .get_current_weather(&location, &units)
        .await
        .expect("Celsius fetch should succeed");

    assert!(
        weather.temperature >= -50.0 && weather.temperature <= 50.0,
        "Celsius temperature should be in typical range"
    );
}

#[tokio::test]
async fn test_open_meteo_provider_integration_fahrenheit_units() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let units = WeatherUnits {
        temperature: TemperatureUnit::Fahrenheit,
        wind_speed: WindSpeedUnit::Mph,
        precipitation: PrecipitationUnit::Inch,
    };

    let weather = client
        .get_current_weather(&location, &units)
        .await
        .expect("Fahrenheit fetch should succeed");

    assert!(
        weather.temperature >= -100.0 && weather.temperature <= 150.0,
        "Fahrenheit temperature should be realistic"
    );
}

#[tokio::test]
async fn test_open_meteo_provider_integration_extreme_locations() {
    let test_cases = vec![
        (90.0, 0.0, "North Pole"),
        (-90.0, 0.0, "South Pole"),
        (0.0, 0.0, "Null Island"),
        (0.0, 180.0, "Date Line"),
    ];

    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));
    let units = WeatherUnits::default();

    for (lat, lon, location_name) in test_cases {
        let location = WeatherLocation {
            latitude: lat,
            longitude: lon,
            elevation: None,
        };

        let result = client.get_current_weather(&location, &units).await;
        assert!(
            result.is_ok(),
            "Should handle extreme location: {}",
            location_name
        );
    }
}

#[tokio::test]
async fn test_open_meteo_provider_integration_with_elevation() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 46.5197,
        longitude: 6.6323,
        elevation: Some(2042.0),
    };

    let units = WeatherUnits::default();

    let result = client.get_current_weather(&location, &units).await;
    assert!(
        result.is_ok(),
        "Should successfully fetch weather with elevation"
    );
}

#[tokio::test]
async fn test_open_meteo_provider_integration_wind_speed_units() {
    let provider = Arc::new(OpenMeteoProvider::new());

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let test_units = vec![
        (WindSpeedUnit::Kmh, "km/h"),
        (WindSpeedUnit::Ms, "m/s"),
        (WindSpeedUnit::Mph, "mph"),
        (WindSpeedUnit::Kn, "knots"),
    ];

    for (wind_unit, unit_name) in test_units {
        let units = WeatherUnits {
            temperature: TemperatureUnit::Celsius,
            wind_speed: wind_unit,
            precipitation: PrecipitationUnit::Mm,
        };

        let client = WeatherClient::new(provider.clone(), Duration::from_secs(1));
        let result = client.get_current_weather(&location, &units).await;

        assert!(
            result.is_ok(),
            "Should successfully fetch with wind speed unit: {}",
            unit_name
        );

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::test]
async fn test_open_meteo_provider_integration_tropical_location() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 1.3521,
        longitude: 103.8198,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let weather = client
        .get_current_weather(&location, &units)
        .await
        .expect("Should fetch tropical weather");

    assert!(
        weather.temperature >= 20.0 && weather.temperature <= 40.0,
        "Tropical temperature should be warm"
    );
    assert!(weather.humidity >= 50.0, "Tropical humidity should be high");
}

#[tokio::test]
async fn test_open_meteo_provider_integration_arctic_location() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 78.2232,
        longitude: 15.6267,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let weather = client
        .get_current_weather(&location, &units)
        .await
        .expect("Should fetch arctic weather");

    assert!(
        weather.temperature >= -60.0 && weather.temperature <= 20.0,
        "Arctic temperature should be cold to moderate"
    );
}
