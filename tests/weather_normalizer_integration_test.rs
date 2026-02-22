use weathr::weather::WeatherCondition;
use weathr::weather::normalizer::WeatherNormalizer;
use weathr::weather::provider::WeatherProviderResponse;

#[test]
fn test_weather_normalizer_integration_all_wmo_codes() {
    let wmo_codes = vec![
        (0, WeatherCondition::Clear),
        (1, WeatherCondition::PartlyCloudy),
        (2, WeatherCondition::PartlyCloudy),
        (3, WeatherCondition::Overcast),
        (45, WeatherCondition::Fog),
        (48, WeatherCondition::Fog),
        (51, WeatherCondition::Drizzle),
        (53, WeatherCondition::Drizzle),
        (55, WeatherCondition::Drizzle),
        (56, WeatherCondition::FreezingRain),
        (57, WeatherCondition::FreezingRain),
        (61, WeatherCondition::Rain),
        (63, WeatherCondition::Rain),
        (65, WeatherCondition::Rain),
        (66, WeatherCondition::FreezingRain),
        (67, WeatherCondition::FreezingRain),
        (71, WeatherCondition::Snow),
        (73, WeatherCondition::Snow),
        (75, WeatherCondition::Snow),
        (77, WeatherCondition::SnowGrains),
        (80, WeatherCondition::RainShowers),
        (81, WeatherCondition::RainShowers),
        (82, WeatherCondition::RainShowers),
        (85, WeatherCondition::SnowShowers),
        (86, WeatherCondition::SnowShowers),
        (95, WeatherCondition::Thunderstorm),
        (96, WeatherCondition::ThunderstormHail),
        (99, WeatherCondition::ThunderstormHail),
    ];

    for (code, expected_condition) in wmo_codes {
        let response = WeatherProviderResponse {
            weather_code: code,
            temperature: 20.0,
            apparent_temperature: 19.0,
            humidity: 75.0,
            precipitation: 0.0,
            wind_speed: 10.0,
            wind_direction: 180.0,
            cloud_cover: 50.0,
            pressure: 1013.0,
            visibility: Some(10000.0),
            is_day: 1,
            moon_phase: None,
            timestamp: "2024-01-01T12:00".to_string(),
            attribution: "".to_string(),
        };

        let weather = WeatherNormalizer::normalize(response);
        assert_eq!(
            weather.condition, expected_condition,
            "WMO code {} should map to {:?}",
            code, expected_condition
        );
    }
}

#[test]
fn test_weather_normalizer_integration_day_night() {
    let response_day = WeatherProviderResponse {
        weather_code: 0,
        temperature: 20.0,
        apparent_temperature: 19.0,
        humidity: 75.0,
        precipitation: 0.0,
        wind_speed: 10.0,
        wind_direction: 180.0,
        cloud_cover: 0.0,
        pressure: 1013.0,
        visibility: Some(10000.0),
        is_day: 1,
        moon_phase: None,
        timestamp: "2024-01-01T12:00".to_string(),
        attribution: "".to_string(),
    };

    let response_night = WeatherProviderResponse {
        weather_code: 0,
        temperature: 15.0,
        apparent_temperature: 14.0,
        humidity: 80.0,
        precipitation: 0.0,
        wind_speed: 5.0,
        wind_direction: 180.0,
        cloud_cover: 0.0,
        pressure: 1013.0,
        visibility: Some(10000.0),
        is_day: 0,
        moon_phase: None,
        timestamp: "2024-01-01T00:00".to_string(),
        attribution: "".to_string(),
    };

    let weather_day = WeatherNormalizer::normalize(response_day);
    let weather_night = WeatherNormalizer::normalize(response_night);

    assert!(weather_day.is_day, "Should correctly identify day");
    assert!(!weather_night.is_day, "Should correctly identify night");
}

#[test]
fn test_weather_normalizer_integration_clear_conditions() {
    let response = WeatherProviderResponse {
        weather_code: 0,
        temperature: 22.5,
        apparent_temperature: 21.0,
        humidity: 60.0,
        precipitation: 0.0,
        wind_speed: 5.0,
        wind_direction: 90.0,
        cloud_cover: 10.0,
        pressure: 1015.0,
        visibility: Some(15000.0),
        is_day: 1,
        moon_phase: None,
        timestamp: "2024-06-15T14:00".to_string(),
        attribution: "".to_string(),
    };

    let weather = WeatherNormalizer::normalize(response);

    assert_eq!(weather.condition, WeatherCondition::Clear);
    assert_eq!(weather.temperature, 22.5);
    assert_eq!(weather.apparent_temperature, 21.0);
    assert_eq!(weather.humidity, 60.0);
    assert_eq!(weather.precipitation, 0.0);
    assert!(weather.is_day);
}

#[test]
fn test_weather_normalizer_integration_rainy_conditions() {
    let response = WeatherProviderResponse {
        weather_code: 61,
        temperature: 15.0,
        apparent_temperature: 13.5,
        humidity: 85.0,
        precipitation: 5.2,
        wind_speed: 12.0,
        wind_direction: 270.0,
        cloud_cover: 95.0,
        pressure: 1005.0,
        visibility: Some(3000.0),
        is_day: 1,
        moon_phase: None,
        timestamp: "2024-03-20T10:00".to_string(),
        attribution: "".to_string(),
    };

    let weather = WeatherNormalizer::normalize(response);

    assert_eq!(weather.condition, WeatherCondition::Rain);
    assert_eq!(weather.precipitation, 5.2);
    assert_eq!(weather.cloud_cover, 95.0);
}

#[test]
fn test_weather_normalizer_integration_snowy_conditions() {
    let response = WeatherProviderResponse {
        weather_code: 71,
        temperature: -2.0,
        apparent_temperature: -5.0,
        humidity: 90.0,
        precipitation: 3.5,
        wind_speed: 8.0,
        wind_direction: 0.0,
        cloud_cover: 100.0,
        pressure: 1010.0,
        visibility: Some(1000.0),
        is_day: 0,
        moon_phase: None,
        timestamp: "2024-01-10T22:00".to_string(),
        attribution: "".to_string(),
    };

    let weather = WeatherNormalizer::normalize(response);

    assert_eq!(weather.condition, WeatherCondition::Snow);
    assert!(weather.temperature < 0.0);
    assert!(!weather.is_day);
}
