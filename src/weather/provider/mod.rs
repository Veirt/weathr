use crate::error::WeatherError;
use crate::weather::types::{WeatherLocation, WeatherUnits};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod open_meteo;
pub mod met_office;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherProviderResponse {
    pub weather_code: i32,
    pub temperature: f64,
    pub apparent_temperature: f64,
    pub humidity: f64,
    pub precipitation: f64,
    pub wind_speed: f64,
    pub wind_direction: f64,
    pub cloud_cover: f64,
    pub pressure: f64,
    pub visibility: Option<f64>,
    pub is_day: i32,
    pub moon_phase: Option<f64>,
    pub timestamp: String,
    pub attribution: String,
}

#[async_trait]
pub trait WeatherProvider: Send + Sync {
    async fn get_current_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
    ) -> Result<WeatherProviderResponse, WeatherError>;

    fn get_attribution(&self) -> &'static str;
}
