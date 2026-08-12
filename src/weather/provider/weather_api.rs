use crate::error::{NetworkError, WeatherError};
use crate::weather::provider::{WeatherProvider, WeatherProviderResponse};
use crate::weather::types::{
    CelestialEvents, PrecipitationUnit, TemperatureUnit, WeatherLocation, WeatherUnits,
    WindSpeedUnit,
};
use crate::weather::units::{normalize_precipitation, normalize_temperature, normalize_wind_speed};
use async_trait::async_trait;
use serde::Deserialize;
use serde::de::{self, Deserializer};
use std::time::Duration;

const WEATHERAPI_BASE_URL: &str = "https://api.weatherapi.com/v1";

pub struct WeatherApiProvider {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct WeatherApiResponse {
    current: CurrentWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    last_updated: String,
    temp_c: f64,
    temp_f: f64,
    is_day: i32,
    precip_mm: f64,
    precip_in: f64,
    condition: WeatherCondition,
    wind_kph: f64,
    wind_mph: f64,
    wind_degree: f64,
}

#[derive(Debug, Deserialize)]
struct WeatherCondition {
    code: i32,
}

fn deserialize_i32_from_number<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Number {
        Integer(i32),
        Float(f64),
    }

    match Number::deserialize(deserializer)? {
        Number::Integer(value) => Ok(value),
        Number::Float(value) => {
            if !value.is_finite() {
                return Err(de::Error::custom("expected a finite numeric value"));
            }
            Ok(value.round() as i32)
        }
    }
}

impl WeatherApiProvider {
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
            base_url: WEATHERAPI_BASE_URL.to_string(),
        }
    }

    fn build_url(&self, location: &WeatherLocation) -> String {
        format!(
            "{}/current.json?q={},{}&aqi=no",
            self.base_url, location.latitude, location.longitude
        )
    }

    fn extract_temperature(&self, current: &CurrentWeather, unit: TemperatureUnit) -> f64 {
        match unit {
            TemperatureUnit::Celsius => current.temp_c,
            TemperatureUnit::Fahrenheit => current.temp_f,
        }
    }

    fn extract_precipitation(&self, current: &CurrentWeather, unit: PrecipitationUnit) -> f64 {
        match unit {
            PrecipitationUnit::Mm => current.precip_mm,
            PrecipitationUnit::Inch => current.precip_in,
        }
    }

    fn extract_wind_speed(&self, current: &CurrentWeather, unit: WindSpeedUnit) -> f64 {
        match unit {
            WindSpeedUnit::Kmh => current.wind_kph * 1.609,  // mph to kmh
            WindSpeedUnit::Ms => current.wind_mph * 0.44704, // mph to m/s
            WindSpeedUnit::Mph => current.wind_mph,
            WindSpeedUnit::Kn => current.wind_mph * 0.868976, // mph to knots
        }
    }
}

impl Default for WeatherApiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WeatherProvider for WeatherApiProvider {
    fn get_attribution(&self) -> &'static str {
        "Weather data provided by WeatherAPI.com"
    }

    async fn get_current_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
    ) -> Result<WeatherProviderResponse, WeatherError> {
        let url = self.build_url(location);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .and_then(|resp| resp.error_for_status())
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;

        let data: WeatherApiResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;

        Ok(WeatherProviderResponse {
            weather_code: data.current.condition.code,
            temperature: self.extract_temperature(&data.current, units.temperature),
            precipitation: self.extract_precipitation(&data.current, units.precipitation),
            wind_speed: self.extract_wind_speed(&data.current, units.wind_speed),
            wind_direction: data.current.wind_degree,
            sun: CelestialEvents::only_day(data.current.is_day),
            moon_phase: None,
            timestamp: data.current.last_updated,
            attribution: self.get_attribution().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        let provider = WeatherApiProvider::new();
        let location = WeatherLocation {
            latitude: 51.5074,
            longitude: -0.1278,
            name: "London".to_string(),
        };
        let url = provider.build_url(&location);
        assert!(url.contains("51.5074"));
        assert!(url.contains("-0.1278"));
        assert!(url.contains("current.json"));
    }
}