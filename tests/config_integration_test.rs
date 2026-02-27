use std::fs;
use std::io::Write;
use weathr::config::{Config, Location, LocationDisplay};
use weathr::weather::types::{PrecipitationUnit, TemperatureUnit, WeatherUnits, WindSpeedUnit};

#[test]
fn test_config_integration_load_valid_file() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_integration_test.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[location]").unwrap();
    writeln!(file, "latitude = 35.6762").unwrap();
    writeln!(file, "longitude = 139.6503").unwrap();
    drop(file);

    let config = Config::load_from_path(&test_config_path).expect("Failed to load config");

    assert_eq!(config.location.latitude, 35.6762);
    assert_eq!(config.location.longitude, 139.6503);

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_realistic_coordinates() {
    let test_cases = vec![
        (52.52, 13.41, "Berlin"),
        (40.7128, -74.0060, "New York"),
        (-33.8688, 151.2093, "Sydney"),
        (35.6762, 139.6503, "Tokyo"),
        (51.5074, -0.1278, "London"),
    ];

    for (lat, lon, city_name) in test_cases {
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join(format!("weathr_test_{}.toml", city_name));

        let mut file = fs::File::create(&test_config_path).unwrap();
        writeln!(file, "[location]").unwrap();
        writeln!(file, "latitude = {}", lat).unwrap();
        writeln!(file, "longitude = {}", lon).unwrap();
        drop(file);

        let config = Config::load_from_path(&test_config_path)
            .unwrap_or_else(|_| panic!("Failed to load config for {}", city_name));

        assert_eq!(config.location.latitude, lat);
        assert_eq!(config.location.longitude, lon);

        fs::remove_file(test_config_path).ok();
    }
}

#[test]
fn test_config_integration_malformed_toml() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_malformed.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[[[[invalid toml").unwrap();
    drop(file);

    let result = Config::load_from_path(&test_config_path);
    assert!(result.is_err());

    use weathr::error::ConfigError;
    match result.unwrap_err() {
        ConfigError::ParseError(_) => {}
        other => panic!("Expected ParseError, got: {:?}", other),
    }

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_missing_fields() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_missing_fields.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[location]").unwrap();
    writeln!(
        file,
        "# Missing latitude and longitude - should use defaults"
    )
    .unwrap();
    drop(file);

    let config =
        Config::load_from_path(&test_config_path).expect("Should use defaults for missing fields");
    assert_eq!(config.location.latitude, 52.52);
    assert_eq!(config.location.longitude, 13.41);

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_extra_whitespace() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_whitespace.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file).unwrap();
    writeln!(file, "  [location]  ").unwrap();
    writeln!(file).unwrap();
    writeln!(file, "  latitude  =  48.8566  ").unwrap();
    writeln!(file, "  longitude  =  2.3522  ").unwrap();
    writeln!(file).unwrap();
    drop(file);

    let config = Config::load_from_path(&test_config_path).expect("Should handle extra whitespace");

    assert_eq!(config.location.latitude, 48.8566);
    assert_eq!(config.location.longitude, 2.3522);

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_city_and_display() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_city_display.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[location]").unwrap();
    writeln!(file, "latitude = 53.9").unwrap();
    writeln!(file, "longitude = 27.5667").unwrap();
    writeln!(file, "auto = false").unwrap();
    writeln!(file, r#"city = "Minsk""#).unwrap();
    writeln!(file, r#"display = "city""#).unwrap();
    drop(file);

    let config = Config::load_from_path(&test_config_path)
        .expect("Should load config with city and display");

    assert_eq!(config.location.city, Some("Minsk".to_string()));
    assert_eq!(config.location.display, LocationDisplay::City);
    assert!(!config.location.auto);

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_display_mixed() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_display_mixed.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[location]").unwrap();
    writeln!(file, "latitude = 34.0754").unwrap();
    writeln!(file, "longitude = -84.2941").unwrap();
    writeln!(file, r#"display = "mixed""#).unwrap();
    drop(file);

    let config =
        Config::load_from_path(&test_config_path).expect("Should load config with mixed display");

    assert_eq!(config.location.display, LocationDisplay::Mixed);
    assert_eq!(config.location.city, None);

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_display_defaults_to_coordinates() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_display_default.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[location]").unwrap();
    writeln!(file, "latitude = 52.52").unwrap();
    writeln!(file, "longitude = 13.41").unwrap();
    drop(file);

    let config =
        Config::load_from_path(&test_config_path).expect("Should default display to coordinates");

    assert_eq!(config.location.display, LocationDisplay::Coordinates);
    assert_eq!(config.location.city, None);

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_city_name_language() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_city_name_lang.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[location]").unwrap();
    writeln!(file, "latitude = 55.7558").unwrap();
    writeln!(file, "longitude = 37.6173").unwrap();
    writeln!(file, r#"display = "city""#).unwrap();
    writeln!(file, r#"city_name_language = "ru""#).unwrap();
    drop(file);

    let config = Config::load_from_path(&test_config_path)
        .expect("Should load config with city_name_language");

    assert_eq!(config.location.city_name_language, "ru");
    assert_eq!(config.location.display, LocationDisplay::City);

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_city_name_language_defaults_to_auto() {
    let temp_dir = std::env::temp_dir();
    let test_config_path = temp_dir.join("weathr_city_name_lang_default.toml");

    let mut file = fs::File::create(&test_config_path).unwrap();
    writeln!(file, "[location]").unwrap();
    writeln!(file, "latitude = 52.52").unwrap();
    writeln!(file, "longitude = 13.41").unwrap();
    drop(file);

    let config = Config::load_from_path(&test_config_path)
        .expect("Should default city_name_language to auto");

    assert_eq!(config.location.city_name_language, "auto");

    fs::remove_file(test_config_path).ok();
}

#[test]
fn test_config_integration_save_and_reload() {
    let config = Config {
        location: Location {
            latitude: -33.8688,
            longitude: 151.2093,
            auto: true,
            hide: false,
            ..Default::default()
        },
        hide_hud: false,
        units: WeatherUnits {
            temperature: TemperatureUnit::Celsius,
            wind_speed: WindSpeedUnit::Ms,
            precipitation: PrecipitationUnit::Mm,
        },
        silent: false,
    };

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("weathr_integration_save.toml");

    config.save(&path).expect("Failed to save config");

    let loaded = Config::load_from_path(&path).expect("Failed to reload saved config");
    assert_eq!(loaded.location.latitude, -33.8688);
    assert_eq!(loaded.location.longitude, 151.2093);
    assert!(loaded.location.auto);
    assert_eq!(loaded.units.wind_speed, WindSpeedUnit::Ms);

    fs::remove_file(path).ok();
}

#[test]
fn test_config_integration_save_imperial_units() {
    let config = Config {
        location: Location {
            latitude: 40.7128,
            longitude: -74.0060,
            auto: false,
            hide: true,
            ..Default::default()
        },
        hide_hud: true,
        units: WeatherUnits::imperial(),
        silent: true,
    };

    let temp_dir = std::env::temp_dir();
    let path = temp_dir.join("weathr_integration_save_imperial.toml");

    config.save(&path).expect("Failed to save config");

    let loaded = Config::load_from_path(&path).expect("Failed to reload saved config");
    assert_eq!(loaded.units.temperature, TemperatureUnit::Fahrenheit);
    assert_eq!(loaded.units.wind_speed, WindSpeedUnit::Mph);
    assert_eq!(loaded.units.precipitation, PrecipitationUnit::Inch);
    assert!(loaded.hide_hud);
    assert!(loaded.silent);
    assert!(loaded.location.hide);

    fs::remove_file(path).ok();
}
