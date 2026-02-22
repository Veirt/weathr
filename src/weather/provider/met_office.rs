use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{error::{ConfigError, NetworkError, WeatherError}, weather::{WeatherLocation, WeatherUnits, provider::{WeatherProvider, WeatherProviderResponse}, units::{fahrenheit_to_celsius, normalize_temperature}}};

const BASE_URL: &str = "https://data.hub.api.metoffice.gov.uk/sitespecific/v0";

pub struct MetOfficeProvider {
    client: reqwest::Client,
    config: MetOfficeProviderConfig,
    last_weather_results: Mutex<Option<MetOfficeResponse>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeProviderConfig {
    #[serde(default)]
    pub include_location_name: bool,

    pub api_key: String,
    
    #[serde(default)]
    pub data_source: String,
}

impl Default for MetOfficeProviderConfig {
    fn default() -> Self {
        Self { include_location_name: true, data_source: "BD1".to_owned(), api_key: String::new()}
    }
}

impl MetOfficeProvider {
    pub fn new(mut config: MetOfficeProviderConfig) -> Result<Self, WeatherError> {

        if config.data_source.is_empty() { config.data_source = MetOfficeProviderConfig::default().data_source; }

        if config.api_key.is_empty() { panic!("API key is empty for Met Office Provider"); }

        let client = reqwest::ClientBuilder::new();

        let mut headers = header::HeaderMap::new();

        let mut auth_value = header::HeaderValue::from_str(&config.api_key)
            .map_err(|_e|{ WeatherError::Config(ConfigError::InvalidAPIKey("Only visible ASCII characters (32-127) are permitted".to_owned())) })?;

        auth_value.set_sensitive(true);
        headers.insert("apikey", auth_value);

        let client = client.default_headers(headers);
        let client = client.build().map_err(|e| { WeatherError::Network(NetworkError::Other(e)) })?;

        Ok(Self {
            client,
            config,
            last_weather_results: Mutex::new(None),
        })
    }

    fn build_url(&self, location: &WeatherLocation) -> String {
        format!("{BASE_URL}/point/hourly?latitude={}&longitude={}&includeLocationName={}&dataSource={}",
            location.latitude,
            location.longitude,
            self.config.include_location_name,
            self.config.data_source
        )
    }

    async fn do_api_req(&self, location: &WeatherLocation) -> Result<MetOfficeResponse, WeatherError> {
        let url = self.build_url(location);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;


        let data: MetOfficeResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::Network(NetworkError::from_reqwest(e, &url, 30)))?;

        Ok(data)
    }

    fn get_current_time_series(data: &MetOfficeResponse) -> Option<MetOfficeTimeSeries> {
        
        if let Some(feature) = data.features.first() {

            let item = feature.properties.time_series.clone().into_iter().find(|item| {
                let time = item.time.replace("Z", ":00Z"); // The Met Office returns the time in a loose format
                let start: DateTime<Utc> = time.parse().unwrap();
                let end = start + chrono::Duration::hours(1);
                Utc::now() >= start && Utc::now() <= end
            });

            return item;
        }

        None
    }
}

#[async_trait]
impl WeatherProvider for MetOfficeProvider {

    fn get_attribution(&self) -> &'static str {
        // Required by Met-Office
        // See: https://www.metoffice.gov.uk/binaries/content/assets/metofficegovuk/pdf/data/met-office-weatherdatahub-terms-and-conditions.pdf
        "Data supplied by the Met Office"
    }

    async fn get_current_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
    ) -> Result<WeatherProviderResponse, WeatherError> {

        let data = if let Ok(mut previous_data_lock) = self.last_weather_results.try_lock() {
            match previous_data_lock.clone() {
                Some(data) => {data},
                None => {
                    let data = self.do_api_req(location).await?;
                    *previous_data_lock = Some(data.clone());
                    data
                },
            }
        } else {
            self.do_api_req(location).await? // Failsafe to ensure data is always available
        };

        let moon_phase = Some(0.5);

        let Some(current_weather) = MetOfficeProvider::get_current_time_series(&data) else {
            return Err(WeatherError::Network(NetworkError::ConnectionRefused { url: self.build_url(location) })) // Don't have a error type for this
            // this is likely never to occur
        };
        
        Ok(
            WeatherProviderResponse {
                weather_code: current_weather.significant_weather_code,
                temperature: current_weather.normalize_screen_temperature(units, &data.parameters),
                apparent_temperature: current_weather.feels_like_temperature,
                humidity: current_weather.screen_relative_humidity,
                precipitation: current_weather.percipitation_rate,
                wind_speed: current_weather.wind_gust_speed_10m,
                wind_direction: current_weather.wind_direction_from_10m as f64,
                cloud_cover: current_weather.uv_index as f64, // Unsure if this is correct
                pressure: current_weather.mslp as f64,
                visibility: Some(current_weather.visibility as f64),
                is_day: current_weather.uv_index as i32, // TODO - The MetOffice doesn't have a day/night indicator, either another API call or a calculation
                moon_phase,
                timestamp: current_weather.time,
                attribution: self.get_attribution().to_string(),
            }
        )
    }
}

pub type MetOfficeParameters = Vec<HashMap<String, MetOfficeParameter>>;

#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeResponse {
    pub features: Vec<MetOfficeFeatures>,
    pub parameters: MetOfficeParameters // This contains the definitions to convert unclean to clean
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeParameter {
    #[allow(unused)]
    pub description: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub unit: MetOfficeParameterUnit
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeParameterUnit {
    pub label: String,
    #[allow(unused)]
    pub symbol: HashMap<String, String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeFeatures {
    #[allow(unused)]
    pub geometry: MetOfficeGeometry,
    pub properties: MetOfficeProperties
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeProperties {
    /// Contains human readable information about the location, also includes license information
    /// TODO: Solves - https://github.com/Veirt/weathr/issues/12
    #[allow(unused)]
    pub location: Option<HashMap<String, String>>, // This is sometimes omitted
    #[serde(rename = "modelRunDate")]
    pub _model_run_date: String,
    #[serde(rename = "requestPointDistance")]
    pub _request_point_distance: f64,
    #[serde(rename = "timeSeries")]
    pub time_series: Vec<MetOfficeTimeSeries> // This contains unclean weather
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeTimeSeries { // Weather event Per Hour
    #[serde(rename = "feelsLikeTemperature")]
    pub feels_like_temperature: f64,
    
    /// Mean Sea Level Pressure
    pub mslp: usize,
    #[serde(rename = "precipitationRate")]
    pub percipitation_rate: f64,

    #[serde(rename = "probOfPrecipitation")]
    pub _probability_of_precipitation: f64,

    #[serde(rename = "screenDewPointTemperature")]
    pub _screen_dew_point_temp: f64,

    #[serde(rename = "screenRelativeHumidity")]
    pub screen_relative_humidity: f64,

    #[serde(rename = "screenTemperature")]
    pub screen_temperature: f64,

    #[serde(rename = "significantWeatherCode")]
    pub significant_weather_code: i32,

    pub time: String,

    #[serde(rename = "uvIndex")]
    pub uv_index: usize,

    #[serde(rename = "visibility")]
    pub visibility: usize,

    #[serde(rename = "windDirectionFrom10m")]
    pub wind_direction_from_10m: usize,
    #[serde(rename = "windGustSpeed10m")]
    pub wind_gust_speed_10m: f64,

    #[serde(rename = "windSpeed10m")]
    pub _wind_speed_10m: f64

}

impl MetOfficeTimeSeries {
    /// This function will attempt to normalize the data
    /// If the Met Office doesn't response with the unit of the field, assume its C per Weights and Measures Act 1985
    pub fn normalize_screen_temperature(&self, units: &WeatherUnits, param: &MetOfficeParameters) -> f64 {
        
        if let Some(param) = Self::find_param(param, "screenTemperature") && param.type_ == "Parameter" {
            if param.unit.label == "degrees Celsius" {
                normalize_temperature(self.screen_temperature, units.temperature)
            } else {
                normalize_temperature(fahrenheit_to_celsius(self.screen_temperature), units.temperature)
            }

        } else {
            normalize_temperature(self.screen_temperature, units.temperature)
        }
        
    }

    fn find_param(param: &MetOfficeParameters, name: &str) -> Option<MetOfficeParameter> {
        for p in param {
            for (k, v) in p {
                if k == name {
                    return Some(v.clone());
                }
            }
        }
        None
    }
}

#[allow(unused)] // TODO: Display this on the UI
#[derive(Debug, Clone, Deserialize)]
pub struct MetOfficeGeometry {
    pub coordinates: Vec<f32>,
    #[serde(rename = "type")]
    pub type_: String
}

#[cfg(test)]
mod tests {
    use std::env;

    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn test_response_parse() {
        let api_key = env::var("MET_OFFICE_API_KEY").unwrap();

        let location = WeatherLocation {
            latitude: 52.52,
            longitude: 13.41,
            elevation: None,
        };

        let provider_cfg = MetOfficeProviderConfig {
            include_location_name: true,
            api_key,
            ..Default::default()
        };

        let provider = MetOfficeProvider::new(provider_cfg).unwrap();
        let url = provider.build_url(&location);

        let response = provider
            .client
            .get(&url)
            .send()
            .await.unwrap();

        let data: Value = response
            .json()
            .await
            .unwrap();

        println!("{data:#?}");

        let _: MetOfficeResponse = serde_json::from_value(data).unwrap();

    }

    #[tokio::test]
    async fn test_met_office_provider() {
        let api_key = env::var("MET_OFFICE_API_KEY").unwrap();
        let provider_cfg = MetOfficeProviderConfig {
            include_location_name: true,
            api_key,
            ..Default::default()
        };

        let provider = MetOfficeProvider::new(provider_cfg).unwrap();

        let location = WeatherLocation {
            latitude: 52.52,
            longitude: 13.41,
            elevation: None,
        };

        let response = provider.get_current_weather(&location, &WeatherUnits::default()).await.unwrap();
        println!("{response:#?}");
    }

}