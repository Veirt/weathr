use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ConfigError;
use crate::weather::types::WeatherUnits;

pub const ENV_LATITUDE: &str = "WEATHR_LATITUDE";
pub const ENV_LONGITUDE: &str = "WEATHR_LONGITUDE";

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocationDisplay {
    #[default]
    Coordinates,
    City,
    Mixed,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    #[serde(default = "default_latitude")]
    pub latitude: f64,
    #[serde(default = "default_longitude")]
    pub longitude: f64,
    #[serde(default)]
    pub auto: bool,
    #[serde(default)]
    pub hide: bool,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub display: LocationDisplay,
    #[serde(default = "default_city_name_language")]
    pub city_name_language: String,
}

fn default_city_name_language() -> String {
    "auto".to_string()
}

pub fn default_latitude() -> f64 {
    52.52
}

pub fn default_longitude() -> f64 {
    13.41
}

impl Default for Location {
    fn default() -> Self {
        Self {
            latitude: default_latitude(),
            longitude: default_longitude(),
            auto: true,
            hide: false,
            city: None,
            display: LocationDisplay::default(),
            city_name_language: default_city_name_language(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            eprintln!(
                "Warning: Config file not found. Create one at {:?} to customize settings.",
                config_path
            );
            let default = Self::default();
            if default.location.auto {
                eprintln!(
                    "Tip: Set latitude and longitude in config.toml for more accurate weather."
                );
            }
            let mut config = default;
            config.apply_env_overrides()?;
            return Ok(config);
        }

        let mut config = Self::load_from_path(&config_path)?;
        config.apply_env_overrides()?;
        config.validate()?;
        Ok(config)
    }

    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        if let Ok(val) = env::var(ENV_LATITUDE) {
            let lat = val
                .trim()
                .parse::<f64>()
                .map_err(|_| ConfigError::InvalidEnvVar {
                    name: ENV_LATITUDE,
                    value: val.clone(),
                })?;
            self.location.latitude = lat;
            self.location.auto = false;
        }

        if let Ok(val) = env::var(ENV_LONGITUDE) {
            let lon = val
                .trim()
                .parse::<f64>()
                .map_err(|_| ConfigError::InvalidEnvVar {
                    name: ENV_LONGITUDE,
                    value: val.clone(),
                })?;
            self.location.longitude = lon;
            self.location.auto = false;
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.location.latitude < -90.0 || self.location.latitude > 90.0 {
            return Err(ConfigError::InvalidLatitude(self.location.latitude));
        }

        if self.location.longitude < -180.0 || self.location.longitude > 180.0 {
            return Err(ConfigError::InvalidLongitude(self.location.longitude));
        }

        Ok(())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
            path: path.display().to_string(),
            source: e,
        })?;

        let value: toml::Value = toml::from_str(&content).map_err(ConfigError::ParseError)?;

        if let Some(loc) = value.get("location") {
            let has_lat = loc.get("latitude").is_some();
            let has_lon = loc.get("longitude").is_some();
            if has_lat && !has_lon {
                eprintln!(
                    "Warning: latitude is set but longitude is missing, defaulting longitude to 13.41 (Berlin)."
                );
            } else if has_lon && !has_lat {
                eprintln!(
                    "Warning: longitude is set but latitude is missing, defaulting latitude to 52.52 (Berlin)."
                );
            }
        }

        toml::Value::try_into(value).map_err(ConfigError::ParseError)
    }

    pub fn get_config_dir() -> Result<PathBuf, ConfigError> {
        let config_dir = if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config)
        } else {
            dirs::config_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
                .ok_or(ConfigError::NoConfigDir)?
        };

        Ok(config_dir.join("weathr"))
    }

    pub fn get_config_path() -> Result<PathBuf, ConfigError> {
        Ok(Self::get_config_dir()?.join("config.toml"))
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self).map_err(ConfigError::SerializeError)?;
        fs::write(path, content).map_err(|e| ConfigError::WriteError {
            path: path.display().to_string(),
            source: e,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

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
        let toml_content = r#"
[location]
latitude = 40.7128
longitude = -74.0060
"#;
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("weathr_test_config.toml");
        fs::write(&test_config_path, toml_content).unwrap();

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
        let toml_content = "this is not valid toml {{{{";
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("weathr_test_invalid.toml");
        fs::write(&test_config_path, toml_content).unwrap();

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
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("weathr_test_missing_lat.toml");
        fs::write(&test_config_path, toml_content).unwrap();

        let config = Config::load_from_path(&test_config_path).unwrap();
        assert_eq!(config.location.latitude, default_latitude());
        assert_eq!(config.location.longitude, 13.41);

        fs::remove_file(test_config_path).ok();
    }

    #[test]
    fn test_config_missing_longitude() {
        let toml_content = r#"
[location]
latitude = 52.52
"#;
        let temp_dir = std::env::temp_dir();
        let test_config_path = temp_dir.join("weathr_test_missing_lon.toml");
        fs::write(&test_config_path, toml_content).unwrap();

        let config = Config::load_from_path(&test_config_path).unwrap();
        assert_eq!(config.location.latitude, 52.52);
        assert_eq!(config.location.longitude, default_longitude());

        fs::remove_file(test_config_path).ok();
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
                city: None,
                display: LocationDisplay::default(),
                city_name_language: "auto".to_string(),
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
                city: None,
                display: LocationDisplay::default(),
                city_name_language: "auto".to_string(),
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
                city: None,
                display: LocationDisplay::default(),
                city_name_language: "auto".to_string(),
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
                city: None,
                display: LocationDisplay::default(),
                city_name_language: "auto".to_string(),
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
                city: None,
                display: LocationDisplay::default(),
                city_name_language: "auto".to_string(),
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

    #[test]
    fn test_location_display_default() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.display, LocationDisplay::Coordinates);
    }

    #[test]
    fn test_location_display_coordinates() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
display = "coordinates"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.display, LocationDisplay::Coordinates);
    }

    #[test]
    fn test_location_display_city() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
display = "city"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.display, LocationDisplay::City);
    }

    #[test]
    fn test_location_display_mixed() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
display = "mixed"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.display, LocationDisplay::Mixed);
    }

    #[test]
    fn test_location_city_field() {
        let toml_content = r#"
[location]
latitude = 53.9
longitude = 27.5667
city = "Minsk"
display = "city"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.city, Some("Minsk".to_string()));
        assert_eq!(config.location.display, LocationDisplay::City);
    }

    #[test]
    fn test_location_city_field_default_none() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.city, None);
    }

    #[test]
    fn test_city_name_language_default() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.city_name_language, "auto");
    }

    #[test]
    fn test_city_name_language_explicit_auto() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
city_name_language = "auto"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.city_name_language, "auto");
    }

    #[test]
    fn test_city_name_language_explicit_en() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
city_name_language = "en"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.city_name_language, "en");
    }

    #[test]
    fn test_city_name_language_explicit_ru() {
        let toml_content = r#"
[location]
latitude = 0.0
longitude = 0.0
city_name_language = "ru"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.location.city_name_language, "ru");
    }

    #[test]
    fn test_env_var_latitude_override() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::set_var("WEATHR_LATITUDE", "48.8566");
            env::remove_var("WEATHR_LONGITUDE");
        }
        let mut config = Config::default();
        config.apply_env_overrides().unwrap();
        assert_eq!(config.location.latitude, 48.8566);
        assert!(!config.location.auto);
        unsafe { env::remove_var("WEATHR_LATITUDE") };
    }

    #[test]
    fn test_env_var_longitude_override() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::remove_var("WEATHR_LATITUDE");
            env::set_var("WEATHR_LONGITUDE", "2.3522");
        }
        let mut config = Config::default();
        config.apply_env_overrides().unwrap();
        assert_eq!(config.location.longitude, 2.3522);
        assert!(!config.location.auto);
        unsafe { env::remove_var("WEATHR_LONGITUDE") };
    }

    #[test]
    fn test_env_var_both_override() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::set_var("WEATHR_LATITUDE", "35.6762");
            env::set_var("WEATHR_LONGITUDE", "139.6503");
        }
        let mut config = Config::default();
        config.apply_env_overrides().unwrap();
        assert_eq!(config.location.latitude, 35.6762);
        assert_eq!(config.location.longitude, 139.6503);
        assert!(!config.location.auto);
        unsafe {
            env::remove_var("WEATHR_LATITUDE");
            env::remove_var("WEATHR_LONGITUDE");
        }
    }

    #[test]
    fn test_env_var_invalid_latitude() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::set_var("WEATHR_LATITUDE", "not-a-number");
            env::remove_var("WEATHR_LONGITUDE");
        }
        let mut config = Config::default();
        let result = config.apply_env_overrides();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "InvalidEnvVar");
        unsafe { env::remove_var("WEATHR_LATITUDE") };
    }

    #[test]
    fn test_env_var_invalid_longitude() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::remove_var("WEATHR_LATITUDE");
            env::set_var("WEATHR_LONGITUDE", "abc");
        }
        let mut config = Config::default();
        let result = config.apply_env_overrides();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "InvalidEnvVar");
        unsafe { env::remove_var("WEATHR_LONGITUDE") };
    }

    #[test]
    fn test_env_var_overrides_config_file_values() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let toml_content = r#"
[location]
latitude = 52.52
longitude = 13.41
auto = false
"#;
        unsafe {
            env::set_var("WEATHR_LATITUDE", "-33.8688");
            env::set_var("WEATHR_LONGITUDE", "151.2093");
        }
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("weathr_test_env_override.toml");
        fs::write(&path, toml_content).unwrap();
        let mut config = Config::load_from_path(&path).unwrap();
        config.apply_env_overrides().unwrap();
        assert_eq!(config.location.latitude, -33.8688);
        assert_eq!(config.location.longitude, 151.2093);
        assert!(!config.location.auto);
        fs::remove_file(path).ok();
        unsafe {
            env::remove_var("WEATHR_LATITUDE");
            env::remove_var("WEATHR_LONGITUDE");
        }
    }

    #[test]
    fn test_config_save_round_trip() {
        let config = Config {
            location: Location {
                latitude: 40.7128,
                longitude: -74.0060,
                auto: false,
                hide: true,
                ..Default::default()
            },
            hide_hud: true,
            units: WeatherUnits {
                temperature: crate::weather::types::TemperatureUnit::Fahrenheit,
                wind_speed: crate::weather::types::WindSpeedUnit::Mph,
                precipitation: crate::weather::types::PrecipitationUnit::Inch,
            },
            silent: true,
        };

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("weathr_test_save_roundtrip.toml");

        config.save(&path).unwrap();
        let loaded = Config::load_from_path(&path).unwrap();

        assert_eq!(loaded.location.latitude, 40.7128);
        assert_eq!(loaded.location.longitude, -74.0060);
        assert!(!loaded.location.auto);
        assert!(loaded.location.hide);
        assert!(loaded.hide_hud);
        assert!(loaded.silent);
        assert_eq!(
            loaded.units.temperature,
            crate::weather::types::TemperatureUnit::Fahrenheit
        );
        assert_eq!(
            loaded.units.wind_speed,
            crate::weather::types::WindSpeedUnit::Mph
        );
        assert_eq!(
            loaded.units.precipitation,
            crate::weather::types::PrecipitationUnit::Inch
        );

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_config_save_default_round_trip() {
        let config = Config::default();

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("weathr_test_save_default.toml");

        config.save(&path).unwrap();
        let loaded = Config::load_from_path(&path).unwrap();

        assert_eq!(loaded.location.latitude, config.location.latitude);
        assert_eq!(loaded.location.longitude, config.location.longitude);
        assert_eq!(loaded.location.auto, config.location.auto);
        assert_eq!(loaded.hide_hud, config.hide_hud);
        assert_eq!(loaded.silent, config.silent);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_config_save_to_invalid_path() {
        let config = Config::default();
        let path = PathBuf::from("/nonexistent_dir_12345/config.toml");
        let result = config.save(&path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "WriteError");
    }
}
