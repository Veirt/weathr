use crate::error::{NetworkError, WeatherError};
use crate::weather::provider::{WeatherProvider, WeatherProviderResponse};
use crate::weather::types::{
    PrecipitationUnit, TemperatureUnit, WeatherLocation, WeatherUnits, WindSpeedUnit,
};
use crate::weather::units::{normalize_precipitation, normalize_temperature, normalize_wind_speed};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;

const OPEN_METEO_BASE_URL: &str = "https://api.open-meteo.com/v1/forecast";

pub struct OpenMeteoProvider {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    time: String,
    temperature_2m: f64,
    relative_humidity_2m: f64,
    apparent_temperature: f64,
    is_day: i32,
    precipitation: f64,
    weather_code: i32,
    cloud_cover: f64,
    surface_pressure: f64,
    wind_speed_10m: f64,
    wind_direction_10m: f64,
    #[serde(default)]
    visibility: Option<f64>,
}

impl OpenMeteoProvider {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to create custom HTTP client: {}", e);
                eprintln!("Using default client with standard timeout settings.");
                reqwest::Client::new()
            });

        Self {
            client,
            base_url: OPEN_METEO_BASE_URL.to_string(),
        }
    }

    fn temperature_unit_param(unit: &TemperatureUnit) -> &'static str {
        match unit {
            TemperatureUnit::Celsius => "celsius",
            TemperatureUnit::Fahrenheit => "fahrenheit",
        }
    }

    fn wind_speed_unit_param(unit: &WindSpeedUnit) -> &'static str {
        match unit {
            WindSpeedUnit::Kmh => "kmh",
            WindSpeedUnit::Ms => "ms",
            WindSpeedUnit::Mph => "mph",
            WindSpeedUnit::Kn => "kn",
        }
    }

    fn precipitation_unit_param(unit: &PrecipitationUnit) -> &'static str {
        match unit {
            PrecipitationUnit::Mm => "mm",
            PrecipitationUnit::Inch => "inch",
        }
    }

    fn build_url(&self, location: &WeatherLocation, units: &WeatherUnits) -> String {
        format!(
            "{}?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,weather_code,cloud_cover,surface_pressure,wind_speed_10m,wind_direction_10m,visibility&temperature_unit={}&wind_speed_unit={}&precipitation_unit={}&timezone=auto",
            self.base_url,
            location.latitude,
            location.longitude,
            Self::temperature_unit_param(&units.temperature),
            Self::wind_speed_unit_param(&units.wind_speed),
            Self::precipitation_unit_param(&units.precipitation)
        )
    }
}

impl Default for OpenMeteoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WeatherProvider for OpenMeteoProvider {
    async fn get_current_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
    ) -> Result<WeatherProviderResponse, WeatherError> {
        let url = self.build_url(location, units);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;

        let data: OpenMeteoResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;

        let moon_phase = Some(0.5);

        Ok(WeatherProviderResponse {
            weather_code: data.current.weather_code,
            temperature: normalize_temperature(data.current.temperature_2m, units.temperature),
            apparent_temperature: normalize_temperature(
                data.current.apparent_temperature,
                units.temperature,
            ),
            humidity: data.current.relative_humidity_2m,
            precipitation: normalize_precipitation(data.current.precipitation, units.precipitation),
            wind_speed: normalize_wind_speed(data.current.wind_speed_10m, units.wind_speed),
            wind_direction: data.current.wind_direction_10m,
            cloud_cover: data.current.cloud_cover,
            pressure: data.current.surface_pressure,
            visibility: data.current.visibility,
            is_day: data.current.is_day,
            moon_phase,
            timestamp: data.current.time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversion_params() {
        assert_eq!(
            OpenMeteoProvider::temperature_unit_param(&TemperatureUnit::Celsius),
            "celsius"
        );
        assert_eq!(
            OpenMeteoProvider::temperature_unit_param(&TemperatureUnit::Fahrenheit),
            "fahrenheit"
        );
        assert_eq!(
            OpenMeteoProvider::wind_speed_unit_param(&WindSpeedUnit::Kmh),
            "kmh"
        );
        assert_eq!(
            OpenMeteoProvider::wind_speed_unit_param(&WindSpeedUnit::Ms),
            "ms"
        );
        assert_eq!(
            OpenMeteoProvider::precipitation_unit_param(&PrecipitationUnit::Mm),
            "mm"
        );
    }
}
