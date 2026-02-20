pub mod client;
pub mod normalizer;
pub mod provider;
pub mod types;
pub mod units;

pub use client::WeatherClient;
pub use provider::open_meteo::OpenMeteoProvider;
pub use types::{
    FogIntensity, RainIntensity, SnowIntensity, WeatherCondition, WeatherConditions, WeatherData,
    WeatherLocation, WeatherUnits,
};
pub use units::{format_precipitation, format_temperature, format_wind_speed};
