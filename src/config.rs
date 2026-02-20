use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::error::ConfigError;
use crate::weather::types::WeatherUnits;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub location: Location,
    #[serde(default)]
    pub hide_hud: bool,
    #[serde(default)]
    pub units: WeatherUnits,
    #[serde(default)]
    pub silent: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Location {
    #[serde(default = "default_latitude")]
    pub latitude: f64,
    #[serde(default = "default_longitude")]
    pub longitude: f64,
    #[serde(default)]
    pub auto: bool,
    #[serde(default)]
    pub hide: bool,
}

fn default_latitude() -> f64 {
    52.52
}

fn default_longitude() -> f64 {
    13.41
}

impl Default for Location {
    fn default() -> Self {
        Self {
            latitude: default_latitude(),
            longitude: default_longitude(),
            auto: true,
            hide: false,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            eprintln!("Config file not found at {:?}", config_path);
            eprintln!("Auto-detecting location via IP...");
            eprintln!("(Set auto = false in config to use Berlin as default)");
            return Ok(Self::default());
        }

        let config = Self::load_from_path(&config_path)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.location.latitude < -90.0 || self.location.latitude > 90.0 {
            return Err(ConfigError::InvalidLatitude(self.location.latitude));
        }

        if self.location.longitude < -180.0 || self.location.longitude > 180.0 {
            return Err(ConfigError::InvalidLongitude(self.location.longitude));
        }

        Ok(())
    }

    pub fn load_from_path(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
            path: path.display().to_string(),
            source: e,
        })?;

        toml::from_str(&content).map_err(ConfigError::ParseError)
    }

    fn get_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(config_dir.join("weathr").join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_config_deserialize_valid() {
        let toml_content = r#"
[location]
latitude = 52.52
longitude = 13.41
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.latitude, 52.52);
        assert_eq!(config.location.longitude, 13.41);
    }

    #[test]
    fn test_config_deserialize_negative_coordinates() {
        let toml_content = r#"
[location]
latitude = -33.8688
longitude = 151.2093
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.latitude, -33.8688);
        assert_eq!(config.location.longitude, 151.2093);
    }

    #[test]
    fn test_config_load_from_path_success() {
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("weathr_test_config.toml");

        let mut file = fs::File::create(&test_config_path).unwrap();
        writeln!(file, "[location]").unwrap();
        writeln!(file, "latitude = 40.7128").unwrap();
        writeln!(file, "longitude = -74.0060").unwrap();

        let config = Config::load_from_path(&test_config_path).unwrap();
        assert_eq!(config.location.latitude, 40.7128);
        assert_eq!(config.location.longitude, -74.0060);

        fs::remove_file(test_config_path).ok();
    }

    #[test]
    fn test_config_load_from_path_file_not_found() {
        let nonexistent_path = PathBuf::from("/tmp/nonexistent_weathr_config_12345.toml");
        let result = Config::load_from_path(&nonexistent_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "ReadError");
    }

    #[test]
    fn test_config_load_from_path_invalid_toml() {
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("weathr_test_invalid.toml");

        let mut file = fs::File::create(&test_config_path).unwrap();
        writeln!(file, "this is not valid toml {{{{").unwrap();

        let result = Config::load_from_path(&test_config_path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "ParseError");

        fs::remove_file(test_config_path).ok();
    }

    #[test]
    fn test_config_missing_latitude() {
        let toml_content = r#"
[location]
longitude = 13.41
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.latitude, 52.52);
        assert_eq!(config.location.longitude, 13.41);
    }

    #[test]
    fn test_config_missing_longitude() {
        let toml_content = r#"
[location]
latitude = 52.52
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.latitude, 52.52);
        assert_eq!(config.location.longitude, 13.41);
    }

    #[test]
    fn test_location_boundary_values() {
        let toml_content = r#"
[location]
latitude = 90.0
longitude = 180.0
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.latitude, 90.0);
        assert_eq!(config.location.longitude, 180.0);
    }

    #[test]
    fn test_location_zero_coordinates() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.latitude, 0.0);
        assert_eq!(config.location.longitude, 0.0);
    }

    #[test]
    fn test_validation_invalid_latitude_high() {
        let config = Config {
            location: Location {
                latitude: 91.0,
                longitude: 0.0,
                auto: false,
                hide: false,
            },
            hide_hud: false,
            units: WeatherUnits::default(),
            silent: false,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "InvalidLatitude");
    }

    #[test]
    fn test_validation_invalid_latitude_low() {
        let config = Config {
            location: Location {
                latitude: -91.0,
                longitude: 0.0,
                auto: false,
                hide: false,
            },
            hide_hud: false,
            units: WeatherUnits::default(),
            silent: false,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "InvalidLatitude");
    }

    #[test]
    fn test_validation_invalid_longitude_high() {
        let config = Config {
            location: Location {
                latitude: 0.0,
                longitude: 181.0,
                auto: false,
                hide: false,
            },
            hide_hud: false,
            units: WeatherUnits::default(),
            silent: false,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "InvalidLongitude");
    }

    #[test]
    fn test_validation_invalid_longitude_low() {
        let config = Config {
            location: Location {
                latitude: 0.0,
                longitude: -181.0,
                auto: false,
                hide: false,
            },
            hide_hud: false,
            units: WeatherUnits::default(),
            silent: false,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "InvalidLongitude");
    }

    #[test]
    fn test_validation_valid_config() {
        let config = Config {
            location: Location {
                latitude: 52.52,
                longitude: 13.41,
                auto: false,
                hide: false,
            },
            hide_hud: false,
            units: WeatherUnits::default(),
            silent: false,
        };
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_units_default() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(
            config.units.temperature,
            crate::weather::types::TemperatureUnit::Celsius
        );
        assert_eq!(
            config.units.wind_speed,
            crate::weather::types::WindSpeedUnit::Kmh
        );
        assert_eq!(
            config.units.precipitation,
            crate::weather::types::PrecipitationUnit::Mm
        );
    }

    #[test]
    fn test_config_units_custom() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0

[units]
temperature = "fahrenheit"
wind_speed = "mph"
precipitation = "inch"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(
            config.units.temperature,
            crate::weather::types::TemperatureUnit::Fahrenheit
        );
        assert_eq!(
            config.units.wind_speed,
            crate::weather::types::WindSpeedUnit::Mph
        );
        assert_eq!(
            config.units.precipitation,
            crate::weather::types::PrecipitationUnit::Inch
        );
    }
}
