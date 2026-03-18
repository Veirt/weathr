use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherCondition {
    Clear,
    PartlyCloudy,
    Cloudy,
    Overcast,
    Fog,
    Drizzle,
    Rain,
    FreezingRain,
    Snow,
    SnowGrains,
    RainShowers,
    SnowShowers,
    Thunderstorm,
    ThunderstormHail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RainIntensity {
    Drizzle,
    Light,
    Heavy,
    Storm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnowIntensity {
    Light,
    Medium,
    Heavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FogIntensity {
    Light,
    Medium,
    Heavy,
}

impl WeatherCondition {
    pub const ALL: &'static [WeatherCondition] = &[
        Self::Clear,
        Self::PartlyCloudy,
        Self::Cloudy,
        Self::Overcast,
        Self::Fog,
        Self::Drizzle,
        Self::Rain,
        Self::FreezingRain,
        Self::RainShowers,
        Self::Snow,
        Self::SnowGrains,
        Self::SnowShowers,
        Self::Thunderstorm,
        Self::ThunderstormHail,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::PartlyCloudy => "partly-cloudy",
            Self::Cloudy => "cloudy",
            Self::Overcast => "overcast",
            Self::Fog => "fog",
            Self::Drizzle => "drizzle",
            Self::Rain => "rain",
            Self::FreezingRain => "freezing-rain",
            Self::RainShowers => "rain-showers",
            Self::Snow => "snow",
            Self::SnowGrains => "snow-grains",
            Self::SnowShowers => "snow-showers",
            Self::Thunderstorm => "thunderstorm",
            Self::ThunderstormHail => "thunderstorm-hail",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Clear => "Clear sunny sky",
            Self::PartlyCloudy => "Partial cloud coverage",
            Self::Cloudy => "Cloudy sky",
            Self::Overcast => "Overcast sky",
            Self::Fog => "Foggy conditions",
            Self::Drizzle => "Light drizzle",
            Self::Rain => "Rain",
            Self::FreezingRain => "Freezing rain",
            Self::RainShowers => "Rain showers",
            Self::Snow => "Snow",
            Self::SnowGrains => "Snow grains",
            Self::SnowShowers => "Snow showers",
            Self::Thunderstorm => "Thunderstorm",
            Self::ThunderstormHail => "Thunderstorm with hail",
        }
    }

    pub fn group(&self) -> &'static str {
        match self {
            Self::Clear | Self::PartlyCloudy | Self::Cloudy | Self::Overcast => "Clear Skies",
            Self::Fog | Self::Drizzle | Self::Rain | Self::FreezingRain | Self::RainShowers => {
                "Precipitation"
            }
            Self::Snow | Self::SnowGrains | Self::SnowShowers => "Snow",
            Self::Thunderstorm | Self::ThunderstormHail => "Storms",
        }
    }

    pub fn rain_intensity(&self) -> RainIntensity {
        match self {
            Self::Drizzle => RainIntensity::Drizzle,
            Self::Rain | Self::RainShowers => RainIntensity::Light,
            Self::FreezingRain => RainIntensity::Heavy,
            Self::Thunderstorm => RainIntensity::Heavy,
            Self::ThunderstormHail => RainIntensity::Storm,
            _ => RainIntensity::Light,
        }
    }

    pub fn snow_intensity(&self) -> SnowIntensity {
        match self {
            Self::SnowGrains => SnowIntensity::Light,
            Self::SnowShowers => SnowIntensity::Medium,
            Self::Snow => SnowIntensity::Heavy,
            _ => SnowIntensity::Light,
        }
    }

    pub fn fog_intensity(&self) -> FogIntensity {
        match self {
            Self::Fog => FogIntensity::Medium,
            _ => FogIntensity::Light,
        }
    }

    pub fn is_raining(&self) -> bool {
        matches!(
            self,
            Self::Drizzle
                | Self::Rain
                | Self::RainShowers
                | Self::FreezingRain
                | Self::Thunderstorm
                | Self::ThunderstormHail
        )
    }

    pub fn is_snowing(&self) -> bool {
        matches!(self, Self::Snow | Self::SnowGrains | Self::SnowShowers)
    }

    pub fn is_thunderstorm(&self) -> bool {
        matches!(self, Self::Thunderstorm | Self::ThunderstormHail)
    }

    pub fn is_cloudy(&self) -> bool {
        matches!(self, Self::PartlyCloudy | Self::Cloudy | Self::Overcast)
    }

    pub fn is_foggy(&self) -> bool {
        matches!(self, Self::Fog)
    }
}

impl std::str::FromStr for WeatherCondition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.to_lowercase().replace('_', "-");
        Self::ALL
            .iter()
            .find(|c| c.as_str() == normalized)
            .copied()
            .ok_or_else(|| format!("Unknown weather condition: '{}'", s))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindSpeedUnit {
    Kmh,
    Ms,
    Mph,
    Kn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrecipitationUnit {
    Mm,
    Inch,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeatherData {
    pub condition: WeatherCondition,
    pub temperature: f64,
    pub precipitation: f64,
    pub wind_speed: f64,
    pub wind_direction: f64,
    pub sun: CelestialEvents,
    pub moon_phase: Option<f64>,
    pub timestamp: String,
    pub attribution: String,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(default)]
pub struct WeatherUnits {
    pub temperature: TemperatureUnit,
    pub wind_speed: WindSpeedUnit,
    pub precipitation: PrecipitationUnit,
}

impl WeatherUnits {
    pub fn imperial() -> Self {
        Self {
            temperature: TemperatureUnit::Fahrenheit,
            wind_speed: WindSpeedUnit::Mph,
            precipitation: PrecipitationUnit::Inch,
        }
    }

    pub fn metric() -> Self {
        Self::default()
    }
}

impl Default for WeatherUnits {
    fn default() -> Self {
        Self {
            temperature: TemperatureUnit::Celsius,
            wind_speed: WindSpeedUnit::Kmh,
            precipitation: PrecipitationUnit::Mm,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct WeatherLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherConditions {
    pub is_raining: bool,
    pub is_snowing: bool,
    pub is_thunderstorm: bool,
    pub is_cloudy: bool,
    pub is_foggy: bool,
    pub sun: CelestialEvents,
}

impl Default for WeatherConditions {
    fn default() -> Self {
        Self {
            is_raining: false,
            is_snowing: false,
            is_thunderstorm: false,
            is_cloudy: false,
            is_foggy: false,
            sun: CelestialEvents::from_bool(true),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CelestialEvents {
    pub is_day: bool,
    pub begin_twilight: Option<NaiveTime>,
    pub rise: Option<NaiveTime>,
    pub upper_transit: Option<NaiveTime>,
    pub set: Option<NaiveTime>,
    pub end_twilight: Option<NaiveTime>,
}

impl CelestialEvents {
    pub fn only_day(is_day: i32) -> Self {
        Self {
            is_day: is_day == 1,
            begin_twilight: None,
            rise: None,
            upper_transit: None,
            set: None,
            end_twilight: None,
        }
    }

    pub fn from_bool(is_day: bool) -> Self {
        Self {
            is_day,
            begin_twilight: None,
            rise: None,
            upper_transit: None,
            set: None,
            end_twilight: None,
        }
    }
}
